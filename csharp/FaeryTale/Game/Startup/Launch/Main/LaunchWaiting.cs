using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Startup;

/// Launch: waiting, delays and skipping the intro.
public static partial class Launch
{
    /// Whether the intro is being skipped. Once Space or Escape has been seen the skip sticks (the
    /// original reset it on every poll without a key).
    public static int IntroSkipped(Data.Ui ui)
    {
        int key = InputHandler.NextKey();

        if (key == ' ' || key == InputHandler.ESCAPE_KEY)
        {
            ui.IntroSkipRequested = 1;
        }

        return ui.IntroSkipRequested;
    }

    /// Whether to leave the intro right now: the player has pressed Space or Escape. The tests
    /// skip the intro too, but they keep its full timing so their screenshots stay the same, so
    /// this is only true in a real launch.
    public static bool LeaveIntroNow(Data.Ui ui) => !LaunchSkipped && IntroSkipped(ui) != 0;

    /// Like Delay, but ends early when the player skips the intro.
    public static void SkippableDelay(Data.Game game, Video video, int ticks)
    {
        for (int tick = 0; tick < Math.Max(ticks, 1); tick++)
        {
            if (LeaveIntroNow(game.Ui))
            {
                return;
            }

            Display.Delay(game, video, 1);
        }
    }

    // Waits for a click or a key, with a hint in the corner. Returns false if the player pressed
    // Escape.
    private static bool WaitForClick(Data.Game game, Video video)
    {
        video.SetOverlay(Overlay.CornerHint(CLICK_HINT));
        InputHandler.ResetKeyBuffer();

        while (!InputHandler.LeftButtonDown)
        {
            int key = InputHandler.NextKey();

            if (key != 0)
            {
                return key != InputHandler.ESCAPE_KEY;
            }

            Display.Delay(game, video, 1);
        }

        WaitForButtonUp(game, video);

        return true;
    }

    // Waits until the mouse button is let go, so one click counts once.
    private static void WaitForButtonUp(Data.Game game, Video video)
    {
        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }
    }

    /// Like SkippableDelay, but a click also ends the wait (a real launch only, so scripted tests
    /// keep their timing). Used by the story book: a click turns the page.
    public static void PageDelay(Data.Game game, Video video, int ticks)
    {
        for (int tick = 0; tick < Math.Max(ticks, 1); tick++)
        {
            if (LeaveIntroNow(game.Ui))
            {
                return;
            }

            if (!LaunchSkipped && InputHandler.LeftButtonDown)
            {
                WaitForButtonUp(game, video);

                return;
            }

            Display.Delay(game, video, 1);
        }
    }
}
