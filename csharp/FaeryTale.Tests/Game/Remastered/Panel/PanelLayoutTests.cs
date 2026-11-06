using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class PanelLayoutTests
{
    private const int FRAME_WIDTH = 640;
    private const int FIRST_LINE = 143;
    private const int LINES = 57;
    private const uint BLUE = 0x0000_00ff;
    private const uint BEIGE = 0x00ff_e0b0;
    private const uint SOLID = 0xff00_0000;

    // A panel that is all black, with a beige patch in the message box, a blue menu box in the first
    // slot holding one black pixel (a letter), and a coloured pixel in the compass.
    private static uint[] Panel()
    {
        uint[] frame = new uint[FRAME_WIDTH * (FIRST_LINE + LINES)];

        for (int y = FIRST_LINE + 10; y < FIRST_LINE + 40; y++)
        {
            Array.Fill(frame, BEIGE, y * FRAME_WIDTH + 20, 100);
        }

        for (int y = FIRST_LINE + 2; y < FIRST_LINE + 10; y++)
        {
            Array.Fill(frame, BLUE, y * FRAME_WIDTH + 430, 48);
        }

        frame[(FIRST_LINE + 5) * FRAME_WIDTH + 440] = 0;
        frame[(FIRST_LINE + 20) * FRAME_WIDTH + 580] = BEIGE;

        return frame;
    }

    private static PanelLayout Prepared(out uint[] frame)
    {
        frame = Panel();

        PanelLayout layout = new(FRAME_WIDTH, LINES);

        layout.Prepare(frame, FRAME_WIDTH, FIRST_LINE);

        return layout;
    }

    // A pixel of the new layout, by the classic line it came from (the layout has room above the panel,
    // and the message box is raised).
    private static uint At(PanelLayout layout, int x, int line) =>
        layout.Arranged[(line + PanelLayout.TOP_PAD_LINES - (x < 426 ? PanelLayout.MESSAGE_RAISE_LINES : 0)) * layout.ArrangedWidth + x];

    [Fact]
    public void TheBlackBackgroundLetsTheWoodShow()
    {
        PanelLayout layout = Prepared(out _);

        Assert.Equal(0u, At(layout, 300, 30));
        Assert.Equal(0u, At(layout, 5, 5));
    }

    [Fact]
    public void ThePiecesKeepTheirColoursAndTheMenuLabelStays()
    {
        PanelLayout layout = Prepared(out _);
        int menuX = 430 + PanelLayout.MENU_SHIFT;

        Assert.Equal(SOLID | BEIGE, At(layout, 30, 20));
        Assert.Equal(SOLID | BLUE, At(layout, menuX + 5, 5));
        Assert.Equal(SOLID, At(layout, menuX + 10, 5));
    }

    [Fact]
    public void TheMenuBoxesHaveABlackFrameRoundThem()
    {
        PanelLayout layout = Prepared(out _);
        int menuX = 430 + PanelLayout.MENU_SHIFT;

        Assert.Equal(SOLID, At(layout, menuX - 2, 5));
        Assert.Equal(SOLID, At(layout, menuX + 5, 0));
        Assert.Equal(0u, At(layout, menuX - 12, 5));
    }

    [Fact]
    public void TheCompassIsMovedOverAndUp()
    {
        PanelLayout layout = Prepared(out _);

        Assert.Equal(SOLID | BEIGE, At(layout, 580 + PanelLayout.COMPASS_SHIFT, 20 - PanelLayout.COMPASS_RAISE_LINES));
    }

    [Fact]
    public void ThePointerIsTurnedBackToTheClassicPlaces()
    {
        // The message box is up; the menu is 4 pixels over; the compass over and up.
        Assert.Equal((100, 50 + PanelLayout.MESSAGE_RAISE_LINES * 2), PanelLayout.ToClassic(100, 50));
        Assert.Equal((440, 50), PanelLayout.ToClassic(440 + PanelLayout.MENU_SHIFT, 50));
        Assert.Equal((580, 50 + PanelLayout.COMPASS_RAISE_LINES * 2), PanelLayout.ToClassic(580 + PanelLayout.COMPASS_SHIFT, 50));
    }
}

