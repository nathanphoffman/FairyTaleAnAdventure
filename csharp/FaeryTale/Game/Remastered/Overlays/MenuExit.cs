using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Platform;

namespace FaeryTale.Game.Remastered;

/// The steps every menu shares when it opens and closes.
internal static class MenuExit
{
    /// Waits out the click that opened a menu, so it does not also choose something in it.
    public static void WaitForRelease(Data.Game game, Video video)
    {
        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }
    }

    /// Takes the menu off the screen, and keeps the click or key that closed it out of the game.
    public static void Leave(Data.Game game, Video video)
    {
        WaitForRelease(game, video);
        video.SetOverlay(new Overlay());
        InputHandler.TakeClick();
        InputHandler.ResetKeyBuffer();
        InputHandler.ForgetLastMenu();
    }
}
