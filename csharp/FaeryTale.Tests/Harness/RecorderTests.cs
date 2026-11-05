using FaeryTale.Game.Output;
using FaeryTale.Gfx;

namespace FaeryTale.Tests;

public class RecorderTests
{
    [Fact]
    public void RecordsCallsAsReadableText()
    {
        Recorder output = new();

        output.SetTextPosition(200, 90);
        output.PrintContBytes("ERROR "u8);
        output.PrintNumber(5, 2);

        Assert.Equal(
            new[] { "SetTextPosition(200, 90)", "PrintContBytes(\"ERROR \")", "PrintNumber(5, 2)" },
            output.Calls.Select(call => call.ToString()));
    }

    [Fact]
    public void ContinuedTextJoinsTheLastPrintedLine()
    {
        Recorder recorder = new();

        // Print and PrintCont are default methods of the interface, so call them through it.
        IOutput output = recorder;
        output.Print("Hello");
        output.PrintCont(" there");
        output.Print("Next");

        Assert.Equal(new[] { "Hello there", "Next" }, recorder.PrintedLines);
    }

    [Fact]
    public void EachWaitFrameCountsOneTick()
    {
        Recorder output = new();

        output.WaitFrame();
        output.WaitFrame();

        Assert.Equal(2u, output.Ticks);
    }

    [Fact]
    public void ColoursAndOverlaysAreSummarised()
    {
        Recorder output = new();

        output.LoadPageColors(new ushort[] { 0x0123, 0x0FFF });
        output.SetOverlay(Overlay.CornerHint("Press ESC to skip"));

        Assert.Equal("LoadPageColors([0x0123, 0x0FFF])", output.Calls[0].ToString());
        Assert.Equal("SetOverlay(1 shades, 0 images, 1 texts)", output.Calls[1].ToString());
    }

    [Fact]
    public void AnEmptyOverlayIsEmptyAndAHintIsNot()
    {
        Assert.True(new Overlay().IsEmpty);
        Assert.False(Overlay.CornerHint("x").IsEmpty);
    }
}
