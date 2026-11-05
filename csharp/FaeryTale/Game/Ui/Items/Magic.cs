using FaeryTale.Game.Creatures;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Ui;

/// The Magic menu actions: casting the spells the hero has stones, vials and so on for. Each use
/// takes one from the stack. Also what the spells do once cast: the map, the stone jump, healing
/// and scaring enemies.
public static class Magic
{
    /// What stands between the hero and a spell.
    internal enum SpellGate
    {
        /// No such spell, or none of it left: the "no spell" event.
        NoSpell,

        /// In a quiet place: the hero says it does not work here.
        QuietZone,

        /// The spell cannot be cast now (freeze while riding, map indoors); nothing is used up.
        Refused,

        Proceed,
    }

    /// Where the stone ring spell takes the hero.
    internal abstract record StoneJump
    {
        /// Not standing in the middle of a stone circle: the spell fails.
        internal sealed record Refused : StoneJump;

        /// In the right sector but at no circle of the list: the stone is used up, nothing happens.
        internal sealed record NoRing : StoneJump;

        /// Jump to this position (the same spot inside the sector of the target circle).
        internal sealed record To(ushort X, ushort Y) : StoneJump;
    }

    private const short SPELL_STONE = 5;
    private const short SPELL_LIGHT = 6;
    private const short SPELL_HEAL = 7;
    private const short SPELL_SECRET = 8;
    private const short SPELL_MAP = 9;
    private const short SPELL_FREEZE = 10;
    private const short SPELL_FEAR = 11;
    private const short FIRST_SPELL = 5;
    private const int SPELL_SLOT_BASE = 4;
    private const short LIGHT_TIME = 760;
    private const short SECRET_TIME = 360;
    private const short FREEZE_TIME = 100;

    // Where the spells cannot be used: the extent with this V3 (a quiet place).
    private const byte NO_MAGIC_ZONE = 9;
    private const int NO_MAGIC_SPEECH = 59;
    private const int NO_SPELL_EVENT = 21;

    /// Regions above this are indoors (no overview map there).
    private const ushort LAST_OUTDOOR_REGION = 7;

    /// Riding something bigger than the raft stops the freeze spell.
    private const short RIDING_BIG = 1;

    // A sector is 256 pixels wide and high, and a tile 16 (shifts of 8 and 4).
    private const int SECTOR_SHIFT = 8;
    private const ushort SECTOR_MASK = 255;
    private const int TILE_SHIFT = 4;

    /// The heal cap grows by a quarter of the bravery.
    private const short BRAVERY_DIVISOR = 4;
    private const int BATTLE_EVENT = 34;

    // The stone circle: sector, and where in the sector the hero must stand.
    private const short STONE_SECTOR = 144;
    private const ushort STONE_COLUMN_WIDTH = 85;
    private const ushort STONE_ROW_HEIGHT = 64;
    private const int STONE_CIRCLES = 11;

    // The overview map and where the hero's position is marked.
    private const long MARK_X_OFFSET = 4;
    private const long MARK_Y_OFFSET = 3;
    private const uint MARK_MAX_X = 320;
    private const uint MARK_MAX_Y = 143;
    private const sbyte VIEW_MAP = 1;
    private const int PRQ_VITALITY_LINE = 4;
    private const int PRQ_MAP = 5;

    /// Healing is capped at this plus a quarter of bravery.
    internal const short HEAL_CAP_BASE = 15;
    private const int HEAL_MIN = 4;

    /// The fear spell only hits enemies of races below this.
    private const byte FEAR_RACE_LIMIT = 7;

    /// Handles an entry of the Magic menu. Returns false if UpdateMenuAvailability should not
    /// follow (the spell did not work, so the stone is not used up).
    public static bool MagicOption(Data.Game game, IOutput output, short chosenEntry)
    {
        if (RefuseSpell(game, output, chosenEntry) is { } result)
        {
            return result;
        }

        if (!CastSpell(game, output, chosenEntry))
        {
            return false;
        }

        UseUpOne(game, chosenEntry);

        return true;
    }

    /// The checks before a spell is cast: count is how many of the spell's item the hero has and
    /// zone the V3 of the current extent.
    internal static SpellGate GateFor(short entry, byte count, byte zone, short riding, short cheat, ushort region)
    {
        if (entry < FIRST_SPELL || count == 0)
        {
            return SpellGate.NoSpell;
        }

        if (zone == NO_MAGIC_ZONE)
        {
            return SpellGate.QuietZone;
        }

        if (entry == SPELL_FREEZE && riding > RIDING_BIG)
        {
            return SpellGate.Refused;
        }

        if (entry == SPELL_MAP && cheat == 0 && region > LAST_OUTDOOR_REGION)
        {
            return SpellGate.Refused;
        }

        return SpellGate.Proceed;
    }

    /// The pure part of the stone jump: the hero goes facing + 1 circles further along the list,
    /// wrapping around after the last one.
    internal static StoneJump StoneTarget(short sector, ushort px, ushort py, sbyte facing, byte[] list)
    {
        if (sector != STONE_SECTOR)
        {
            return new StoneJump.Refused();
        }

        if ((px & SECTOR_MASK) / STONE_COLUMN_WIDTH != 1 || (py & SECTOR_MASK) / STONE_ROW_HEIGHT != 1)
        {
            return new StoneJump.Refused();
        }

        // Which sector the hero is in: pixels divided by the sector size via a shift.
        int sectorX = px >> SECTOR_SHIFT;
        int sectorY = py >> SECTOR_SHIFT;

        for (int ring = 0; ring < STONE_CIRCLES; ring++)
        {
            if (list[ring * 2] != sectorX || list[ring * 2 + 1] != sectorY)
            {
                continue;
            }

            int target = ring + facing + 1;

            if (target > STONE_CIRCLES - 1)
            {
                target -= STONE_CIRCLES;
            }

            // Back to pixels: the target sector, then the same spot inside it.
            ushort x = unchecked((ushort)((list[target * 2] << SECTOR_SHIFT) + (px & SECTOR_MASK)));
            ushort y = unchecked((ushort)((list[target * 2 + 1] << SECTOR_SHIFT) + (py & SECTOR_MASK)));

            return new StoneJump.To(x, y);
        }

        return new StoneJump.NoRing();
    }

    /// Vitality after the heal spell: it adds 4 plus a roll (0-7), up to a limit that grows with
    /// bravery. Also says whether the limit was hit (then the hero was already nearly full).
    internal static (short Vitality, bool Capped) HealedVitality(short vitality, short brave, int roll)
    {
        short cap = (short)(HEAL_CAP_BASE + brave / BRAVERY_DIVISOR);
        short healed = unchecked((short)(vitality + (short)(roll + HEAL_MIN)));

        return healed > cap ? (cap, true) : (healed, false);
    }

    /// Whether the fear spell kills this actor: a living enemy of one of the weaker races.
    internal static bool IsScaredByFearSpell(Shape actor) =>
        actor.Vitality != 0 && actor.Kind == ActorKind.ENEMY && actor.Race < FEAR_RACE_LIMIT;

    // Checks that the spell can be cast here and now. Gives a result for MagicOption to return
    // when it cannot, and null when the spell goes ahead.
    private static bool? RefuseSpell(Data.Game game, IOutput output, short chosenEntry)
    {
        byte count = chosenEntry < FIRST_SPELL ? (byte)0 : game.Hero.Stuff[SPELL_SLOT_BASE + chosenEntry];
        SpellGate gate = GateFor(chosenEntry, count, game.Extents[game.World.Extent].V3, game.Hero.Riding, game.Ui.CheatEnabled, game.World.Region);

        switch (gate)
        {
            case SpellGate.NoSpell:
                TextRoutines.ShowEvent(game, output, NO_SPELL_EVENT);

                return true;

            case SpellGate.QuietZone:
                TextRoutines.Speak(game, output, NO_MAGIC_SPEECH);

                return true;

            case SpellGate.Refused:
                return false;

            default:
                return null;
        }
    }

    // Casts the spell. Returns false if it did not work (the stone ring only works in a circle).
    private static bool CastSpell(Data.Game game, IOutput output, short chosenEntry)
    {
        switch (chosenEntry)
        {
            case SPELL_LIGHT:
                game.World.LightTimer += LIGHT_TIME;
                break;

            case SPELL_SECRET:
                game.World.SecretTimer += SECRET_TIME;
                break;

            case SPELL_FREEZE:
                game.World.FreezeTimer += FREEZE_TIME;
                break;

            case SPELL_MAP:
                ShowMap(game, output);
                break;

            case SPELL_STONE:
                if (!StoneJumpSpell(game, output))
                {
                    return false;
                }

                Heal(game, output);
                break;

            case SPELL_HEAL:
                Heal(game, output);
                break;

            case SPELL_FEAR:
                ScareEnemies(game, output);
                break;
        }

        return true;
    }

    // Takes one of the spell's stones from the stack, and updates the menu when the last is gone.
    private static void UseUpOne(Data.Game game, short chosenEntry)
    {
        int slot = SPELL_SLOT_BASE + chosenEntry;

        game.Hero.Stuff[slot] = unchecked((byte)(game.Hero.Stuff[slot] - 1));

        if (game.Hero.Stuff[slot] == 0)
        {
            Menus.UpdateMenuAvailability(game);
        }
    }

    // Draws the overview map with a "+" where the hero is, and shows it.
    private static void ShowMap(Data.Game game, IOutput output)
    {
        output.DrawOverviewMap(game.Map, game.World, game.World.MapX, game.World.MapY);

        // The hero on the overview map: his tile column minus the left edge of the shown sectors,
        // and his position in 16 pixel steps minus the top edge.
        uint x = unchecked((uint)((game.Hero.X >> TILE_SHIFT) - ((game.World.SectorX + (long)game.World.OriginX) << TILE_SHIFT) - MARK_X_OFFSET));
        uint y = unchecked((uint)((game.Hero.Y >> TILE_SHIFT) - ((game.World.SectorY + (long)game.World.OriginY) << TILE_SHIFT) + MARK_Y_OFFSET));
        bool onMap = x > 0 && x < MARK_MAX_X && y > 0 && y < MARK_MAX_Y;

        output.MarkOverviewMap(onMap ? ((int)x, (int)y) : null);

        game.Ui.ViewStatus = VIEW_MAP;
        MessagePage.ShowStillScreen(game, output);
        TextRoutines.QueuePrint(game.Ui, PRQ_MAP);
    }

    // The stone ring spell: only works standing in the middle of a stone circle. Takes the hero to
    // the next circle in the direction faced. Returns false if the spell did not work.
    private static bool StoneJumpSpell(Data.Game game, IOutput output)
    {
        StoneJump jump = StoneTarget(game.Hero.Sector, game.Hero.X, game.Hero.Y, game.Actors.List[0].Facing, Tables.StoneRingSectors);

        switch (jump)
        {
            case StoneJump.Refused:
                return false;

            case StoneJump.To to:
                Fading.FlickerColors(game, output);
                Places.TransferHero(game, output, to.X, to.Y, 1);

                if (game.Hero.Riding != 0)
                {
                    game.Actors.List[game.Actors.CarrierSlot].AbsX = game.Actors.List[0].AbsX;
                    game.Actors.List[game.Actors.CarrierSlot].AbsY = game.Actors.List[0].AbsY;
                }

                return true;

            default:
                return true;
        }
    }

    // Heals the hero a little, up to a limit set by bravery.
    private static void Heal(Data.Game game, IOutput output)
    {
        Shape hero = game.Actors.List[0];
        (short vitality, bool capped) = HealedVitality(hero.Vitality, game.Hero.Brave, game.Rng.Rand8());

        hero.Vitality = vitality;

        if (!capped)
        {
            output.Print("That feels a lot better!");
        }

        TextRoutines.QueuePrint(game.Ui, PRQ_VITALITY_LINE);
    }

    // Kills every weak enemy on the screen (and costs a little bravery for each).
    private static void ScareEnemies(Data.Game game, IOutput output)
    {
        for (int counter = 1; counter < game.Actors.Active; counter++)
        {
            Shape actor = game.Actors.List[counter];

            if (IsScaredByFearSpell(actor))
            {
                actor.Vitality = 0;
                Encounters.CheckDeath(game, output, counter, 0);
                game.Hero.Brave--;
            }
        }

        if (game.Actors.BattleFlag != 0)
        {
            TextRoutines.ShowEvent(game, output, BATTLE_EVENT);
        }
    }
}
