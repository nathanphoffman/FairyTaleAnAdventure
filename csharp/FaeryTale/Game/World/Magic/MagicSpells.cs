using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Spells;

/// Magic: the spells that move or heal the hero.
public static partial class Magic
{
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
        int sectorY = py >> SECTOR_SHIFT; // same shift for the row

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
            ushort positionX = unchecked((ushort)((list[target * 2] << SECTOR_SHIFT) + (px & SECTOR_MASK)));
            ushort positionY = unchecked((ushort)((list[target * 2 + 1] << SECTOR_SHIFT) + (py & SECTOR_MASK)));  // same as x, for the row

            return new StoneJump.To(positionX, positionY);
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

    // Checks that the spell can be cast here and now. Gives a result for MagicOption to return
    // when it cannot, and null when the spell goes ahead.
    private static bool? RefuseSpell(Data.Game game, IOutput output, short chosenEntry)
    {
        byte count = chosenEntry < FIRST_SPELL ? (byte)0 : game.Hero.Stuff[SPELL_SLOT_BASE + chosenEntry];
        SpellGate gate = GateFor(chosenEntry, count, game.Extents[game.World.Extent].V3, game.Hero.Riding, game.Ui.CheatEnabled, GateRegion(game));

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

    // The stone ring spell also heals, but only when the jump worked.
    private static bool JumpAndHeal(Data.Game game, IOutput output)
    {
        if (!StoneJumpSpell(game, output))
        {
            return false;
        }

        Heal(game, output);

        return true;
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
}
