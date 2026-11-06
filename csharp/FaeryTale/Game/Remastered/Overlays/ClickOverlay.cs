using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Platform;

namespace FaeryTale.Game.Remastered;

/// What click to move draws over the play field: the circle where the player clicked (it stays
/// on the same spot of ground as the map scrolls), and a door icon in place of the pointer while
/// the mouse is over a door, and an eye while shift is held.
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
        bool looking = IsLookMode(game);

        // The new world drawer draws the marker and the door icon itself, in its own pixels.
        if (output is Video video && WindowControls.NativeRendering && video.Capture.Enabled)
        {
            video.Capture.Marker = ClickToMove.Marker;
            video.Capture.DoorHover = overDoor;
            video.Capture.HoverPerson = PersonUnderPointer(game);
            video.Capture.TalkingTo = Conversation.Person;
            video.Capture.LookMode = IsLookMode(game);
            video.Capture.ItemHover = IsPointerOverItem(game, video.Capture);
            ClearOverlay(output);

            return;
        }

        if (ClickToMove.Marker is null && !overDoor && !looking)
        {
            ClearOverlay(output);

            return;
        }

        Overlay overlay = new() { HidePointer = overDoor || looking };

        if (ClickToMove.Marker is { } marker)
        {
            overlay.Images.Add(ClickRing.PictureAt(ToOverlay(game, marker.World), marker.Walkable));
        }

        if (overDoor || looking)
        {
            (int x, int y) = InputHandler.RawPointer;
            (int X, int Y) at = (x * OVERLAY_SCALE, y * OVERLAY_SCALE);

            overlay.Images.Add(looking ? EyeCursor.At(at) : DoorCursor.At(at));
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

        return PointerWorld.UnderPointer(game) is { } ground && DoorTargets.DoorAt(game, ground) is not null;
    }

    // Shift is held in a game with click to move: the pointer is an eye.
    private static bool IsLookMode(Data.Game game) =>
        InputHandler.ShiftHeld && game.Experience != Data.Experience.Original && game.Ui.ViewStatus == VIEW_NORMAL;

    // Whether the mouse is over something Take would pick up (the new drawer only).
    private static bool IsPointerOverItem(Data.Game game, WorldCapture capture) =>
        game.Experience != Data.Experience.Original
        && game.Ui.ViewStatus == VIEW_NORMAL
        && PointerWorld.UnderPointer(game) is { } ground
        && ItemTargets.ItemAt(capture, ground) is not null;

    // The person the mouse is over, if the new drawer can show their face.
    private static int? PersonUnderPointer(Data.Game game)
    {
        if (game.Experience == Data.Experience.Original || game.Ui.ViewStatus != VIEW_NORMAL)
        {
            return null;
        }

        return PointerWorld.UnderPointer(game) is { } ground ? PersonTargets.PersonAt(game, ground) : null;
    }

    // Takes off an overlay this class put up.
    private static void ClearOverlay(IOutput output)
    {
        if (shown)
        {
            output.SetOverlay(new Overlay());
            shown = false;
        }
    }

    private static (int X, int Y) ToOverlay(Data.Game game, (int X, int Y) world)
    {
        (int x, int y) = ScreenMap.ToScreen(game.World, world);

        return (x * OVERLAY_SCALE, y * OVERLAY_SCALE);
    }
}
