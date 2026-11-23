using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Render;

/// ScreenEffects: debug coordinates and the stat numbers.
public static partial class ScreenEffects
{
    // Debug information: the hero's map position and the memory left.
    private static void ShowDebugCoords(Hero hero, IOutput output)
    {
        output.Print("Coords = ");
        output.PrintNumber(hero.X, COORD_DIGITS);
        output.PrintNumber(hero.Y, COORD_DIGITS);
        output.Print("Memory Available: ");
        output.PrintNumber(FAKE_FREE_BYTES, COORD_DIGITS);
    }

    // Debug information: the hero's sector and the type of the extent the hero is in.
    private static void ShowDebugSector(World world, Hero hero, IOutput output)
    {
        output.Print("You are at: ");
        output.PrintNumber(hero.X / SECTOR_PIXELS, SECTOR_DIGITS);
        output.PrintNumber(hero.Y / SECTOR_PIXELS, SECTOR_DIGITS);
        output.Print("H/Sector = ");
        output.PrintNumber(hero.Sector, SECTOR_DIGITS);
        output.PrintContBytes(" Extent = "u8);
        output.PrintNumber(world.ExtentType, EXTENT_DIGITS);
    }

    // The hero's vitality on the stats line.
    private static void DrawVitality(Actors actors, IOutput output)
    {
        TextRoutines.MoveTo(output, VITALITY_X, STATS_Y);
        output.PrintContBytes("Vit:"u8);
        output.PrintNumber(actors.List[0].Vitality, STAT_DIGITS);
    }

    // Draws the hero's four stats (bravery, luck, kindness, wealth) along the stats line.
    private static void DrawStats(Hero hero, IOutput output)
    {
        if (hero.Luck < 0)
        {
            hero.Luck = 0;
        }

        DrawStat(output, BRAVERY_X, "Brv:"u8, hero.Brave);
        DrawStat(output, LUCK_X, "Lck:"u8, hero.Luck);
        DrawStat(output, KINDNESS_X, "Knd:"u8, hero.Kindness);
        DrawStat(output, WEALTH_X, "Wlth:"u8, hero.Wealth);
    }

    private static void DrawStat(IOutput output, int x, ReadOnlySpan<byte> label, short value)
    {
        TextRoutines.MoveTo(output, x, STATS_Y);
        output.PrintContBytes(label);
        output.PrintNumber(value, STAT_DIGITS);
    }
}
