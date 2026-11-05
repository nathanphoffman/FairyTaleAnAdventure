using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Creatures;

/// Placing new monsters while the hero walks about.
public static class Spawning
{
    private const short ENEMY_SLOT = 2;
    private const short FIRST_ENEMY_ACTOR = 3;
    private const short LAST_ENEMY_ACTOR = 7;
    private const int PLACE_TRIES = 10;

    // Waiting encounters are placed on every 16th tick, and a new random one is rolled every 32nd.
    private const ushort PLACE_PERIOD_MASK = 15;
    private const ushort ROLL_PERIOD_MASK = 31;

    // The danger of an encounter is the extent type plus this base (higher indoors); the roll to
    // beat is 0-63.
    private const ushort DANGER_BASE_OUTDOORS = 2;
    private const ushort DANGER_BASE_INDOORS = 5;

    // Encounter kinds chosen by rule: the dark knight extent turns a kind 2 into 4, the spider pit
    // always gives 6 and the final extent 2.
    private const ushort KIND_DKNIGHT_FROM = 2;
    private const ushort KIND_DKNIGHT_TO = 4;
    private const ushort KIND_SPIDER_PIT = 6;
    private const ushort KIND_FINAL = 2;

    // Extents whose type is a multiple of 4 never mix races.
    private const ushort NO_MIX_MASK = 3;
    private const ushort SPREAD = 63;
    private const int PLAIN_TERRAIN = 0;
    private const byte WRAITH = 2;

    // Extent type values: above the first no mixed groups; the limit for random encounters;
    // special zones.
    private const ushort NO_MIX_XTYPE = 49;
    private const ushort NO_RANDOM_XTYPE = 50;
    private const ushort SPIDER_PIT_XTYPE = 8;
    private const ushort DKNIGHT_XTYPE = 7;
    private const ushort FINAL_XTYPE = 49;
    private const ushort INDOOR_REGION = 7;

    /// Says where the hero is, finishes loading monsters, places waiting ones and now and then
    /// starts a new random encounter. (Disk reads finish at once now, so a loading monster file
    /// is always ready.)
    public static void SpawnEncounters(Data.Game game, IOutput output)
    {
        Places.FindPlace(game, output, 2);

        if (game.Actors.Loading == 1)
        {
            output.FinishSpriteLoad(ENEMY_SLOT);
            game.Actors.Loading = 0;
            game.Actors.Active = FIRST_ENEMY_ACTOR;
        }

        if ((game.World.DayClock & PLACE_PERIOD_MASK) == 0 && game.Encounter.Number != 0 && game.Actors.Loading == 0)
        {
            PlaceWaitingEncounters(game);
        }

        if (RandomEncounterDue(game.Actors, game.World))
        {
            RollRandomEncounter(game, output);
        }
    }

    // A new random encounter is rolled now and then, when nothing else is going on.
    private static bool RandomEncounterDue(Actors actors, World world) =>
        (world.DayClock & ROLL_PERIOD_MASK) == 0
        && actors.OnScreen == 0
        && actors.Loading == 0
        && actors.ActiveCarrier == 0
        && world.ExtentType < NO_RANDOM_XTYPE;

    // Rolls the danger of the area against a random number; if it is beaten, picks the kind of the
    // encounter (a few extents decide it by rule) and starts loading the monsters' sprites.
    private static void RollRandomEncounter(Data.Game game, IOutput output)
    {
        ushort danger = game.World.Region > INDOOR_REGION ? DANGER_BASE_INDOORS : DANGER_BASE_OUTDOORS;

        game.Encounter.DangerLevel = (byte)(danger + game.World.ExtentType);

        if (game.Rng.Rand64() > game.Encounter.DangerLevel)
        {
            return;
        }

        game.Encounter.Kind = (ushort)game.Rng.Rand4();

        if (game.World.ExtentType == DKNIGHT_XTYPE && game.Encounter.Kind == KIND_DKNIGHT_FROM)
        {
            game.Encounter.Kind = KIND_DKNIGHT_TO;
        }

        if (game.World.ExtentType == SPIDER_PIT_XTYPE)
        {
            game.Encounter.Kind = KIND_SPIDER_PIT;
            game.Encounter.MixFlag = 0;
        }

        if (game.World.ExtentType == FINAL_XTYPE)
        {
            game.Encounter.Kind = KIND_FINAL;
            game.Encounter.MixFlag = 0;
        }

        Encounters.LoadEnemySprites(game, output);
    }

    // Tries to find a clear spot near the hero and puts the waiting encounter there, into free
    // actor slots first and then into dead ones.
    private static void PlaceWaitingEncounters(Data.Game game)
    {
        game.Encounter.MixFlag = (short)game.Rng.Next();

        if (game.World.ExtentType > NO_MIX_XTYPE)
        {
            game.Encounter.MixFlag = 0;
        }

        game.Encounter.WeaponType = (short)game.Rng.Rand4();

        if ((game.World.ExtentType & NO_MIX_MASK) == 0)
        {
            game.Encounter.MixFlag = 0;
        }

        for (int attempt = 1; attempt < PLACE_TRIES; attempt++)
        {
            Extents.PickEncounterSpot(game);

            if (MapLookup.TerrainAt(game, game.Encounter.X, game.Encounter.Y) != PLAIN_TERRAIN)
            {
                continue;
            }

            PlaceInFreeSlots(game);
            PlaceInDeadSlots(game);

            break;
        }
    }

    // Puts the waiting monsters into the actor slots that are still free.
    private static void PlaceInFreeSlots(Data.Game game)
    {
        while (game.Encounter.Number != 0 && game.Actors.Active < LAST_ENEMY_ACTOR)
        {
            if (Encounters.PlaceEncounter(game, (ushort)game.Actors.Active, SPREAD) != 0)
            {
                game.Actors.Active++;
            }

            game.Encounter.Number--;
        }
    }

    // Puts the monsters still waiting in place of dead ones that are no longer on screen (a wraith
    // can be replaced even while its body is seen).
    private static void PlaceInDeadSlots(Data.Game game)
    {
        for (int index = FIRST_ENEMY_ACTOR; index < LAST_ENEMY_ACTOR && game.Encounter.Number != 0; index++)
        {
            Shape actor = game.Actors.List[index];

            if (actor.State == ActorState.DEAD && (actor.Visible == 0 || actor.Race == WRAITH))
            {
                Encounters.PlaceEncounter(game, (ushort)index, SPREAD);
                game.Encounter.Number--;
            }
        }
    }
}
