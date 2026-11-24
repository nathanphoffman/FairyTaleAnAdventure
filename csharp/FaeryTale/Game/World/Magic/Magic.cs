using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Reimagined;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Spells;

/// The Magic menu actions: casting the spells the hero has stones, vials and so on for. Each use
/// takes one from the stack. Also what the spells do once cast: the map, the stone jump, healing
/// and scaring enemies.
public static partial class Magic
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

    // The region the map spell's gate looks at: the map is only for outdoors, but the skull that takes
    // its place in Reimagined burns anywhere.
    private static ushort GateRegion(Data.Game game) => game.Experience == Experience.Reimagined ? (ushort)0 : game.World.Region;

    // Casts the spell. Returns false if it did not work (the stone ring only works in a circle).
    private static bool CastSpell(Data.Game game, IOutput output, short chosenEntry)
    {
        if (chosenEntry == SPELL_STONE)
        {
            return JumpAndHeal(game, output);
        }

        CastOtherSpell(game, output, chosenEntry);

        return true;
    }

    // The spells that cannot fail: the timers, the map (or the skull), healing and fear.
    private static void CastOtherSpell(Data.Game game, IOutput output, short chosenEntry)
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

            case SPELL_MAP when game.Experience == Experience.Reimagined:
                InfernoSkull.Cast(game);
                break;

            case SPELL_MAP:
                ShowMap(game, output);
                break;

            case SPELL_HEAL:
                Heal(game, output);
                break;

            case SPELL_FEAR:
                ScareEnemies(game, output);
                break;
        }
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
}
