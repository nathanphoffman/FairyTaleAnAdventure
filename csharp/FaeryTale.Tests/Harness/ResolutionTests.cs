using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class ResolutionTests
{
    // The choices for a monitor size, with the monitor put back afterwards.
    private static List<WindowControls.Resolution> ChoicesFor(int width, int height)
    {
        (int Width, int Height) before = WindowControls.Monitor;

        try
        {
            WindowControls.Monitor = (width, height);

            return WindowControls.ResolutionChoices();
        }
        finally
        {
            WindowControls.Monitor = before;
        }
    }

    [Fact]
    public void TheMonitorSizeComesFirstAsBorderlessAndTheRestAreWindowsFromLargeToSmall()
    {
        List<WindowControls.Resolution> choices = ChoicesFor(1920, 1080);

        Assert.Equal(new WindowControls.Resolution(1920, 1080, true), choices[0]);
        Assert.All(choices.Skip(1), choice => Assert.False(choice.Borderless));
        Assert.Equal(new WindowControls.Resolution(640, 480, false), choices[^1]);
        Assert.Equal(choices.Skip(1).OrderByDescending(choice => choice.Width), choices.Skip(1));
    }

    [Fact]
    public void OnlySizesThatFitTheMonitorAreOffered()
    {
        List<WindowControls.Resolution> choices = ChoicesFor(1280, 720);

        Assert.All(choices, choice => Assert.True(choice.Width <= 1280 && choice.Height <= 720));
        Assert.DoesNotContain(choices, choice => choice.Width == 1920);
    }

    [Fact]
    public void TheWordsSayBorderlessOrWindowed()
    {
        Assert.Equal("1920x1080 (borderless)", new WindowControls.Resolution(1920, 1080, true).Text);
        Assert.Equal("800x600 (windowed)", new WindowControls.Resolution(800, 600, false).Text);
    }

    [Fact]
    public void ARequestIsTakenOnce()
    {
        WindowControls.RequestResolution(new WindowControls.Resolution(800, 600, false));

        Assert.Equal(new WindowControls.Resolution(800, 600, false), WindowControls.TakeResolutionRequest());
        Assert.Null(WindowControls.TakeResolutionRequest());
    }
}
