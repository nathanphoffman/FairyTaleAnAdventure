using FaeryTale.Game.Startup;

namespace FaeryTale.Tests;

public class PageTurnTests
{
    private const uint RED = 0xffff0000;
    private const uint BLUE = 0xff0000ff;
    private const int ROW = 200;

    private static PageTurn Turn()
    {
        uint[] old = Enumerable.Repeat(RED, PageTurn.PAGE_WIDTH * PageTurn.PAGE_HEIGHT).ToArray();
        uint[] fresh = Enumerable.Repeat(BLUE, PageTurn.PAGE_WIDTH * PageTurn.PAGE_HEIGHT).ToArray();

        return new PageTurn(old, fresh);
    }

    private static uint At(uint[] picture, int x, int y) => picture[y * PageTurn.OUTPUT_WIDTH + x];

    [Fact]
    public void ThePageStartsAsTheOldPictureAndEndsAsTheNewOne()
    {
        PageTurn turn = Turn();

        Assert.Equal(RED, At(turn.Render(0), 450, ROW));
        Assert.Equal(RED, At(turn.Render(0), 200, ROW));
        Assert.Equal(BLUE, At(turn.Render(1), 450, ROW));
        Assert.Equal(BLUE, At(turn.Render(1), 200, ROW));
    }

    [Fact]
    public void HalfWayTheOldPageIsOnTheLeftAndTheNewOneOnTheRight()
    {
        uint[] picture = Turn().Render(0.5);

        Assert.Equal(RED, At(picture, 100, ROW));
        Assert.Equal(BLUE, At(picture, 560, ROW));
    }

    [Fact]
    public void TheRisingPageIsNarrowerThanItWasAtTheStart()
    {
        // At a third of the way the lifting old page no longer reaches the right edge of the book.
        uint[] picture = Turn().Render(1.0 / 3);

        Assert.Equal(BLUE, At(picture, 560, ROW));
        // The page is a little darker as it rises, so it is only red, not exactly RED.
        Assert.True(((At(picture, 360, ROW) >> 16) & 0xff) > 150);
    }
}
