using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// Pausing is an entry of the Game menu, so the panel is on the Game menu when the game is paused and
/// (as the menu buttons do nothing while paused) stays there. In Remastered and Reimagined the panel goes
/// back to the first menu, the one with Items, Magic, Talk, Buy and Game, as soon as the game goes on.
public static class PauseMenu
{
    private const int ITEMS_MENU = 0;

    // The game menu entry that pauses the game; bit 0 set means paused.
    private const int GAME_MENU = 4;
    private const int PAUSE_ENTRY = 5;

    private static bool wasPaused;

    /// Whether the game is paused now.
    public static bool IsPaused(Data.Game game) => (game.Menus[GAME_MENU].Enabled[PAUSE_ENTRY] & 1) != 0;

    /// Forgets that the game was paused (a new game starts).
    public static void Reset()
    {
        wasPaused = false;
        QueuedUse.Reset();
    }

    /// Called every frame with whether the game is paused: when it just stopped being paused, shows
    /// shows the first menu and does what was queued while it was paused.
    public static void Update(Data.Game game, IOutput output, bool paused)
    {
        if (wasPaused && !paused && game.Experience != Data.Experience.Original)
        {
            Menus.GoToMenu(game, output, ITEMS_MENU);
            QueuedUse.Run(game, output);
        }

        wasPaused = paused;
    }
}
