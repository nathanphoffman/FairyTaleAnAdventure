using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// Pausing is an entry of the Game menu, so the panel is on the Game menu when the game is paused and
/// (as the menu buttons do nothing while paused) stays there. In Remastered and Reimagined the panel goes
/// back to the first menu, the one with Items, Magic, Talk, Buy and Game, as soon as the game goes on.
public static class PauseMenu
{
    private const int ITEMS_MENU = 0;

    private static bool wasPaused;

    /// Forgets that the game was paused (a new game starts).
    public static void Reset() => wasPaused = false;

    /// Called every frame with whether the game is paused: when it just stopped being paused, shows
    /// the first menu.
    public static void Update(Data.Game game, IOutput output, bool paused)
    {
        if (wasPaused && !paused && game.Experience != Data.Experience.Original)
        {
            Menus.GoToMenu(game, output, ITEMS_MENU);
        }

        wasPaused = paused;
    }
}
