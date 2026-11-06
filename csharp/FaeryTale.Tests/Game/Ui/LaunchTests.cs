using FaeryTale.Game.Startup;
using FaeryTale.Gfx;
using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class LaunchTests
{
    [Fact]
    public void TheCameraGoesToTheEndAndComesBack()
    {
        int width = 320 + 100;

        Assert.Equal(0, MenuBackground.PanPosition(0, width));
        Assert.Equal(40, MenuBackground.PanPosition(2 * 40, width));
        Assert.Equal(100, MenuBackground.PanPosition(2 * 100, width));
        Assert.Equal(40, MenuBackground.PanPosition(2 * 160, width));
        Assert.Equal(0, MenuBackground.PanPosition(2 * 200, width));
        Assert.Equal(0, MenuBackground.PanPosition(12345, 320));
    }

    [Fact]
    public void EveryBlockIsFoundByItsTitleAndByItsLastLine()
    {
        var blocks = LaunchMenu.Layout();

        for (int index = 0; index < blocks.Count; index++)
        {
            Assert.Equal(index, LaunchMenu.OptionAt(blocks, Overlay.WIDTH / 2, blocks[index].TitleY - 4));
            Assert.Equal(index, LaunchMenu.OptionAt(blocks, Overlay.WIDTH / 2, blocks[index].LastBaseline));
        }
    }

    [Fact]
    public void TheHeadingTheGapsAndTheSidesPickNothing()
    {
        var blocks = LaunchMenu.Layout();

        Assert.Null(LaunchMenu.OptionAt(blocks, Overlay.WIDTH / 2, 56));
        Assert.Null(LaunchMenu.OptionAt(blocks, Overlay.WIDTH / 2, blocks[0].PanelSpan.Bottom + 1));
        Assert.Null(LaunchMenu.OptionAt(blocks, 0, blocks[0].TitleY));
    }

    [Fact]
    public void EverythingFitsAboveTheQuitEntryAndQuitIsInTheBottomRightCorner()
    {
        var blocks = LaunchMenu.Layout();
        var quit = LaunchMenu.QuitArea();

        Assert.True(blocks[^1].PanelSpan.Bottom < quit.Top);
        Assert.True(quit.Top + quit.Height <= 400);
        Assert.Equal(3, LaunchMenu.OptionAt(blocks, Overlay.WIDTH - 24, 376));
        Assert.Null(LaunchMenu.OptionAt(blocks, 470, 376));
        Assert.Equal(3, LaunchMenu.OptionForKey('Q'));
    }

    [Fact]
    public void NumberKeysPickOptionsAndLongTextWraps()
    {
        Assert.Equal(0, LaunchMenu.OptionForKey('1'));
        Assert.Equal(2, LaunchMenu.OptionForKey('3'));
        Assert.Null(LaunchMenu.OptionForKey('4'));
        Assert.Null(LaunchMenu.OptionForKey(0));

        Assert.Equal(["one two", "three", "four"], LaunchMenu.Wrap("one two three four", 9));
        Assert.All(LaunchMenu.Wrap(LaunchMenu.DescriptionOf(2), 64), line => Assert.True(line.Length <= 64));
    }

    [Fact]
    public void TheSplashFitsAndNamesThePeopleAndTheLicense()
    {
        foreach (Splash.Page page in Splash.PAGES)
        {
            var lines = Splash.BodyLines(page);

            Assert.True(lines[^1].Y <= 370);
            Assert.All(lines, line => Assert.True(line.X >= 0 && line.X + Overlay.TextWidth(line.Content, 1) <= Overlay.WIDTH));
        }

        string all = string.Concat(Splash.PAGES.SelectMany(page => page.Paragraphs));

        Assert.All(new[] { "David Joiner", "Hollyware", "MicroIllusions", "MIT-style", "2022" }, needle => Assert.Contains(needle, all));

        Overlay overlay = Splash.PageOverlay(Splash.PAGES[0], 0, "Press ESC to skip");

        Assert.True(overlay.Texts[^1].X + Overlay.TextWidth(overlay.Texts[^1].Content, 1) < overlay.Texts[0].X);
    }

    [Fact]
    public void TheBoardIsSolidAndTheFarCornerIsClear()
    {
        uint[] pixels = Wood.Plank(60, 30, 1, false);
        int fullWidth = 60 + Wood.SHADOW;

        Assert.Equal(fullWidth * (30 + Wood.SHADOW), pixels.Length);
        Assert.Equal(255u, pixels[10 * fullWidth + 10] >> 24);
        Assert.Equal(0u, pixels[^1]);

        uint below = pixels[(30 + 2) * fullWidth + 20] >> 24;
        uint right = pixels[15 * fullWidth + 60 + 2] >> 24;

        Assert.InRange(below, 1u, 254u);
        Assert.InRange(right, 1u, 254u);
        Assert.Equal(0u, pixels[(30 + 2) * fullWidth + 2] >> 24);
    }

    [Fact]
    public void BoardsAreTheSameEveryTimeAndDifferBySeedAndALitBoardIsBrighter()
    {
        Assert.Equal(Wood.Plank(50, 20, 4, false), Wood.Plank(50, 20, 4, false));
        Assert.NotEqual(Wood.Plank(50, 20, 4, false), Wood.Plank(50, 20, 5, false));

        uint[] plain = Wood.Plank(80, 40, 3, false);
        uint[] lit = Wood.Plank(80, 40, 3, true);
        int row = 20 * (80 + Wood.SHADOW);

        static uint Brightness(uint[] pixels, int from, int to) =>
            (uint)Enumerable.Range(from, to - from).Sum(i => (pixels[i] >> 16 & 0xff) + (pixels[i] >> 8 & 0xff) + (pixels[i] & 0xff)) / (uint)(to - from);

        Assert.True(Brightness(lit, row, row + 80) > Brightness(plain, row, row + 80));
    }

    [Fact]
    public void OnlyRealSdlKeyNamesDoAnything()
    {
        Assert.Equal(KeyNames.KeyKind.Fire, KeyNames.Lookup("Left Ctrl").Kind);
        Assert.Equal(KeyNames.KeyKind.None, KeyNames.Lookup("LCtrl").Kind); // not an SDL name, so scripts using it do nothing
        Assert.Equal(KeyNames.KeyKind.Arrow, KeyNames.Lookup("Right").Kind);
        Assert.Equal('A', KeyNames.Lookup("A").Value);
        Assert.Equal('A', KeyNames.Lookup("a").Value);
        Assert.Equal('5', KeyNames.Lookup("5").Value);
        Assert.Equal(10, KeyNames.Lookup("F1").Value);
        Assert.Equal(19, KeyNames.Lookup("F10").Value);
        Assert.Equal(20, KeyNames.Lookup("Keypad 7").Value);
        Assert.Equal(29, KeyNames.Lookup("Keypad 5").Value);
        Assert.Equal('0', KeyNames.Lookup("Keypad 0").Value);
        Assert.Equal(0x20, KeyNames.Lookup("Space").Value);
        Assert.Equal(KeyNames.KeyKind.None, KeyNames.Lookup("LShift").Kind);
        Assert.Equal(KeyNames.KeyKind.None, KeyNames.Lookup("Kp7").Kind);

        Assert.Equal(21, KeyNames.DirectionCode(true, false, false, false));
        Assert.Equal(22, KeyNames.DirectionCode(true, false, false, true));
        Assert.Equal(0, KeyNames.DirectionCode(false, false, false, false));
        Assert.Equal(0, KeyNames.DirectionCode(true, true, false, false));
    }

    [Fact]
    public void TheReplayEntriesAreInTheBottomLeftAndHaveKeys()
    {
        var blocks = LaunchMenu.Layout();
        var book = LaunchMenu.BookArea();
        var prologue = LaunchMenu.PrologueArea();

        Assert.Equal(4, LaunchMenu.OptionAt(blocks, book.Left + 4, book.Top + 4));
        Assert.Equal(5, LaunchMenu.OptionAt(blocks, prologue.Left + 4, prologue.Top + 4));
        Assert.True(prologue.Left >= book.Left + book.Width);
        Assert.Equal(4, LaunchMenu.OptionForKey('B'));
        Assert.Equal(5, LaunchMenu.OptionForKey('P'));
    }
}
