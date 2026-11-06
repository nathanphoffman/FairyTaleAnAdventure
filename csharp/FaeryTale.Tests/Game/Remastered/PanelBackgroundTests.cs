using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class PanelBackgroundTests
{
    private const int FRAME_WIDTH = 640;
    private const int FIRST_LINE = 143;
    private const int LINES = 57;
    private const uint BLUE = 0x0000_00ff;
    private const uint BROWN = 0x0070_4020;

    // A panel that is all black, with a blue menu box in the first slot holding one black pixel
    // (a letter), and a beige patch that is not black.
    private static uint[] BlackPanelWithBox()
    {
        uint[] frame = new uint[FRAME_WIDTH * (FIRST_LINE + LINES)];

        for (int y = FIRST_LINE + 2; y < FIRST_LINE + 10; y++)
        {
            Array.Fill(frame, BLUE, y * FRAME_WIDTH + 430, 48);
        }

        frame[(FIRST_LINE + 5) * FRAME_WIDTH + 440] = 0;

        return frame;
    }

    [Fact]
    public void BlackBackgroundBecomesWoodButALabelInsideABoxStays()
    {
        uint[] frame = BlackPanelWithBox();
        PanelBackground background = new(FRAME_WIDTH, LINES);

        background.Prepare(frame, FRAME_WIDTH, FIRST_LINE);

        Assert.NotEqual(0u, background.Pixel(frame, FRAME_WIDTH, FIRST_LINE, 100, 30) & 0x00ff_ffff);
        Assert.Equal(0u, background.Pixel(frame, FRAME_WIDTH, FIRST_LINE, 440, 5));
        Assert.Equal(BLUE, background.Pixel(frame, FRAME_WIDTH, FIRST_LINE, 435, 5));
    }

    [Fact]
    public void AnEmptyBoxIsBackgroundToo()
    {
        uint[] frame = BlackPanelWithBox();
        PanelBackground background = new(FRAME_WIDTH, LINES);

        background.Prepare(frame, FRAME_WIDTH, FIRST_LINE);

        // The second slot of the first row is all black.
        Assert.NotEqual(0u, background.Pixel(frame, FRAME_WIDTH, FIRST_LINE, 490, 5) & 0x00ff_ffff);
    }

    [Fact]
    public void ColouredPixelsAreNeverChanged()
    {
        uint[] frame = BlackPanelWithBox();

        frame[(FIRST_LINE + 30) * FRAME_WIDTH + 100] = BROWN;

        PanelBackground background = new(FRAME_WIDTH, LINES);

        background.Prepare(frame, FRAME_WIDTH, FIRST_LINE);

        Assert.Equal(BROWN, background.Pixel(frame, FRAME_WIDTH, FIRST_LINE, 100, 30));
    }
}
