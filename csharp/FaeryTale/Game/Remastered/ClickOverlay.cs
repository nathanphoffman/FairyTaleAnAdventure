using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// What click to move draws over the play field: the circle where the player clicked (it stays
/// on the same spot of ground as the map scrolls), and a door icon in place of the pointer while
/// the mouse is over a door.
public static class ClickOverlay
{
    /// The overlay grid is twice as wide and twice as tall as the game's screen.
    private const int OVERLAY_SCALE = 2;

    /// The play field ends at this screen line.
    private const int PLAY_FIELD_BOTTOM = 143;

    private const sbyte VIEW_NORMAL = 0;

    private static bool shown;

    /// Forgets that something was shown (a new game starts with a clear overlay).
    public static void Reset() => shown = false;

    /// Draws what is wanted this frame and takes it off when nothing is. Called every frame.
    public static void Update(Data.Game game, IOutput output)
    {
        bool overDoor = IsPointerOverDoor(game);

        if (ClickToMove.Marker is null && !overDoor)
        {
            if (shown)
            {
                output.SetOverlay(new Overlay());
                shown = false;
            }

            return;
        }

        Overlay overlay = new() { HidePointer = overDoor };

        if (ClickToMove.Marker is { } marker)
        {
            overlay.Images.Add(ClickRing.PictureAt(ToOverlay(game, marker.World), marker.Walkable));
        }

        if (overDoor)
        {
            (int x, int y) = InputHandler.RawPointer;

            overlay.Images.Add(DoorCursor.At((x * OVERLAY_SCALE, y * OVERLAY_SCALE)));
        }

        output.SetOverlay(overlay);
        shown = true;
    }

    // Whether the mouse is over a door of the play field (Remastered and Reimagined only).
    private static bool IsPointerOverDoor(Data.Game game)
    {
        if (game.Experience == Data.Experience.Original || game.Ui.ViewStatus != VIEW_NORMAL)
        {
            return false;
        }

        (int x, int y) = InputHandler.RawPointer;

        return y < PLAY_FIELD_BOTTOM && DoorTargets.DoorAt(game, ScreenMap.ToWorld(game.World, (x, y))) is not null;
    }

    private static (int X, int Y) ToOverlay(Data.Game game, (int X, int Y) world)
    {
        (int x, int y) = ScreenMap.ToScreen(game.World, world);

        return (x * OVERLAY_SCALE, y * OVERLAY_SCALE);
    }
}
