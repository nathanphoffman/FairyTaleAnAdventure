using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures;

/// Placing monsters and carriers, and what happens when an actor dies.
public static class Encounters
{
    /// What a new death needs shown or said.
    internal readonly record struct Death(bool DarkKnight);

    /// A weapon with this bit set is a bow, so the monster is an archer.
    private const sbyte BOW_BIT = 4;

    /// Tries this many random spots before giving up.
    private const int MAX_TRY = 15;

    // The encounter's MixFlag bits: 2 picks one of a pair of races, 4 picks the weapon at random.
    private const short MIX_RANDOM_WEAPON = 4;
    private const short MIX_RACE_PAIR = 2;

    // The enemy kind that never mixes races, and the mask that rounds a race down to the first of
    // its pair.
    private const ushort KIND_NO_MIX = 4;
    private const ushort PAIR_MASK = 0xfffe;

    /// Each chart row has four weapon choices.
    private const int WEAPON_CHOICES = 4;

    // An enemy starts this far off from the screen's corner (its sprite's anchor, in pixels).
    private const ushort ENEMY_ANCHOR_X = 8;
    private const ushort ENEMY_ANCHOR_Y = 26;

    // A carrier appears this far from its extent's corner.
    private const ushort CARRIER_OFFSET_X = 250;
    private const ushort CARRIER_OFFSET_Y = 200;

    /// Actors active once a carrier is out: the hero, the raft, the first person, and the carrier.
    private const short ACTORS_WITH_CARRIER = 4;

    // The extent value (V3) of the special encounter that always appears at a fixed place.
    private const byte FIXED_PLACE_ENCOUNTER = 7;
    private const ushort FIXED_PLACE_X = 21635;
    private const ushort FIXED_PLACE_Y = 25762;

    // The spider pit extent type, where monsters may stand on terrain 7.
    private const ushort SPIDER_PIT = 52;
    private const int TERRAIN_SWAMP = 7;

    // The monster that is the dark knight and the kinds of people that matter in CheckDeath.
    private const byte RACE_DARK_KNIGHT = 7;
    private const byte RACE_LASSO_KEEPER = 0x89;
    private const int SPEAK_DARK_KNIGHT_DIES = 42;

    // Penalties and rewards for a death.
    private const short KINDNESS_LOSS = 3;
    private const short LUCK_LOSS = 5;
    internal const sbyte DYING_COUNTDOWN = 7;

    // Carrier numbers.
    private const short CARRIER_DRAGON = 10;
    private const short CARRIER_TURTLE = 5;
    private const short VITALITY_CARRIER = 50;
    private const byte SEQ_ENEMY = 2;
    private const int FIRST_CARRIER_SLOT = 3;
    private const short FIRST_ENEMY_SLOT = 3;

    /// Places a new monster in slot position of the actor list somewhere within spread pixels of
    /// the encounter spot, on a free square. Its race is the encounter kind (mixed with the one
    /// next to it when MixFlag bit 1 is set). Returns 1 if it was placed and 0 if no free spot was
    /// found in 15 tries.
    public static int PlaceEncounter(Data.Game game, ushort position, ushort spread)
    {
        if (FindOpenSpot(game, position, spread) is not { } spot)
        {
            return 0;
        }

        Shape actor = game.Actors.List[position];

        actor.AbsX = spot.X;
        actor.AbsY = spot.Y;
        actor.Kind = ActorKind.ENEMY;
        int race = EncounterRace(game);
        actor.Race = (byte)race;
        ArmEnemy(game, actor, race);

        actor.RelX = (ushort)(actor.AbsX - game.World.MapX - ENEMY_ANCHOR_X);
        actor.RelY = (ushort)(actor.AbsY - game.World.MapY - ENEMY_ANCHOR_Y);

        return 1;
    }

    /// Starts dying if the actor at position is out of vitality (and is not already dying or
    /// dead): it becomes DYING, the hero gains bravery for a kill, and killing a person (not the
    /// lasso keeper) costs 3 kindness. If it is the hero, dtype is the death message, luck drops
    /// by 5 and the music changes. The hero's stats are always refreshed on screen.
    public static void CheckDeath(Data.Game game, IOutput output, int position, int dtype)
    {
        if (StartDying(position, game.Actors.List[position], game.Hero) is { } death)
        {
            if (death.DarkKnight)
            {
                TextRoutines.Speak(game, output, SPEAK_DARK_KNIGHT_DIES);
            }

            if (position == 0)
            {
                TextRoutines.ShowEvent(game, output, dtype);
                Music.SetMood(game, 1);
            }

            TextRoutines.QueuePrint(game.Ui, PrintRequest.STATS);
        }

        if (position == 0)
        {
            TextRoutines.QueuePrint(game.Ui, PrintRequest.VITALITY);
        }
    }

    /// Puts carrier n (5 turtle, 10 dragon, otherwise the swan) in actor slot 3, loading its
    /// sprites if they are not the ones already loaded, in the middle of its extent.
    public static void LoadCarrier(Data.Game game, IOutput output, short n)
    {
        Shape actor = game.Actors.List[FIRST_CARRIER_SLOT];

        actor.Kind = n == CARRIER_DRAGON ? ActorKind.DRAGON : ActorKind.CARRIER;
        int extent = n == CARRIER_DRAGON ? 2 : n == CARRIER_TURTLE ? 1 : 0;

        if (game.Actors.LoadedActorFile != n)
        {
            output.RewindSpritesTo(SEQ_ENEMY);
            output.LoadSpriteFile(n);
            output.FinishSpriteLoad(actor.Kind);
        }

        Extent e = game.Extents[extent];

        actor.AbsX = (ushort)(e.X1 + CARRIER_OFFSET_X);
        actor.AbsY = (ushort)(e.Y1 + CARRIER_OFFSET_Y);
        actor.Index = 0;
        actor.Weapon = 0;
        actor.Environ = 0;
        actor.State = ActorState.STILL;
        actor.Vitality = VITALITY_CARRIER;
        game.Actors.Active = ACTORS_WITH_CARRIER;
        game.Actors.ActiveCarrier = n;
        game.Actors.LoadedActorFile = n;
        actor.Race = (byte)n;
    }

    /// The actors' sprite file the monsters of this encounter need: sets how many appear and, if
    /// that file is not the one loaded, loads it (the carrier is dropped then).
    public static void LoadEnemySprites(Data.Game game, IOutput output)
    {
        Extent extent = game.Extents[game.World.Extent];

        game.Encounter.Number = (byte)(extent.V1 + game.Rng.Rnd(extent.V2));
        short file = Tables.Entry(Tables.EncounterChart, game.Encounter.Kind).FileId;

        if (game.Actors.LoadedActorFile != file)
        {
            game.Actors.LoadedActorFile = file;
            game.Actors.Active = FIRST_ENEMY_SLOT;
            output.RewindSpritesTo((byte)ActorKind.ENEMY);
            output.LoadSpriteFile(game.Actors.LoadedActorFile);
            game.Actors.Loading = 1;
            game.Actors.ActiveCarrier = 0;
        }
    }

    /// Starts the actor dying if it is out of vitality and not already dying or dead: it goes
    /// into the dying state with a countdown. Killing a person (except the lasso keeper) costs the
    /// hero kindness; killing anything else gives bravery; the hero's own death costs luck.
    /// Returns null if nothing happened.
    internal static Death? StartDying(int position, Shape actor, Hero hero)
    {
        if (actor.Vitality >= 1 || actor.State == ActorState.DYING || actor.State == ActorState.DEAD)
        {
            return null;
        }

        actor.Vitality = 0;
        actor.Tactic = DYING_COUNTDOWN;
        actor.Goal = ActorGoal.DEATH;
        actor.State = ActorState.DYING;
        bool darkKnight = actor.Race == RACE_DARK_KNIGHT;

        if (!darkKnight && actor.Kind == ActorKind.SETFIG && actor.Race != RACE_LASSO_KEEPER)
        {
            hero.Kindness -= KINDNESS_LOSS;
        }

        if (position != 0)
        {
            hero.Brave++;
        }
        else
        {
            hero.Luck -= LUCK_LOSS;
        }

        if (hero.Kindness < 0)
        {
            hero.Kindness = 0;
        }

        return new Death(darkKnight);
    }

    // Gives the new enemy its weapon (picked from the race's chart, at random for a mixed group),
    // its goal (archers shoot, the rest attack, both as clever as the race is) and its hit points.
    private static void ArmEnemy(Data.Game game, Shape actor, int race)
    {
        if ((game.Encounter.MixFlag & MIX_RANDOM_WEAPON) != 0)
        {
            game.Encounter.WeaponType = (short)game.Rng.Rand4();
        }

        Encounter chart = Tables.Entry(Tables.EncounterChart, race);
        int weapon = chart.Arms * WEAPON_CHOICES + game.Encounter.WeaponType;

        actor.Weapon = ObjectTables.WeaponProbs[weapon];
        actor.State = ActorState.STILL;
        actor.Environ = 0;
        actor.Facing = 0;
        actor.Goal = (sbyte)((actor.Weapon & BOW_BIT) != 0 ? ActorGoal.ARCHER1 + chart.Cleverness : ActorGoal.ATTACK1 + chart.Cleverness);
        actor.Vitality = chart.Hitpoints;
    }

    // Finds a place near the encounter point for the actor, trying up to MAX_TRY random spots
    // within spread pixels. Some encounters always use one fixed place. Returns null if every spot
    // was blocked.
    private static (ushort X, ushort Y)? FindOpenSpot(Data.Game game, ushort position, ushort spread)
    {
        if (game.Extents[game.World.Extent].V3 == FIXED_PLACE_ENCOUNTER)
        {
            return (FIXED_PLACE_X, FIXED_PLACE_Y);
        }

        for (int attempt = 0; attempt < MAX_TRY; attempt++)
        {
            ushort x = (ushort)(game.Encounter.X + game.Rng.BitRand(spread) - spread / 2);
            ushort y = (ushort)(game.Encounter.Y + game.Rng.BitRand(spread) - spread / 2);

            if (Blocking.BlockedAt(game, (short)x, (short)y, (short)position) == 0)
            {
                return (x, y);
            }

            if (game.World.ExtentType == SPIDER_PIT && MapLookup.TerrainAt(game, x, y) == TERRAIN_SWAMP)
            {
                return (x, y);
            }
        }

        return null;
    }

    // The race of the next enemy: for mixed groups the two races of the pair are picked at random
    // (except encounter type 4).
    private static int EncounterRace(Data.Game game) =>
        (game.Encounter.MixFlag & MIX_RACE_PAIR) != 0 && game.Encounter.Kind != KIND_NO_MIX
            ? (game.Encounter.Kind & PAIR_MASK) + game.Rng.Rand2()
            : game.Encounter.Kind;
}
