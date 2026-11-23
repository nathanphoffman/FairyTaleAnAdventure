using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using static FaeryTale.Gfx.MenuStyle;
using FaeryTale.Platform;

namespace FaeryTale.Game.Startup;

/// What the player chose on the launch menu.
internal enum LaunchAction
{
    Play,
    ShowBook,
    ShowPrologue,
    Quit,
}
