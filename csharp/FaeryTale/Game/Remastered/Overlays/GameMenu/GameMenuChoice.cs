using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Platform;
using static FaeryTale.Gfx.MenuStyle;

namespace FaeryTale.Game.Remastered;

/// What the player chose in the game menu.
public enum GameMenuChoice
{
    /// Close the menu and carry on playing.
    Resume,

    /// Leave this game and go back to the main menu (the choice of Original, Remastered or Reimagined).
    BackToGameSelection,

    /// Leave the program.
    ExitGame,
}
