using FaeryTale.Game.Output;
using FaeryTale.Game.Testing;

namespace FaeryTale.Game.Render;

/// Showing the finished page. The game draws on one of two pages while the other is on screen,
/// then swaps them.
public static class Display
{
    /// Waits one tick of 1/50 second, showing the screen as it is now.
    public static void WaitFrame(Data.Game game, IOutput output)
    {
        StateDump.DumpIfDue(game, output);
        output.WaitFrame();
    }

    /// Waits the given number of ticks (at least one), showing the screen as it is now.
    public static void Delay(Data.Game game, IOutput output, int ticks)
    {
        for (int tick = 0; tick < Math.Max(ticks, 1); tick++)
        {
            WaitFrame(game, output);
        }
    }

    /// Shows the next picture of an animation (one a refresh of the monitor in the window).
    public static void AnimationFrame(Data.Game game, IOutput output)
    {
        StateDump.DumpIfDue(game, output);
        output.ShowAnimationFrame();
    }

    /// Swaps the page being drawn with the one being shown, and shows the one just finished.
    public static void FlipPages(Data.Game game, IOutput output)
    {
        output.SwapPages();

        StateDump.DumpIfDue(game, output);
        output.Present();
        WaitFrame(game, output);
    }
}
