using FaeryTale.Game.Data;

namespace FaeryTale.Game.Remastered;

/// Converts between positions on the play field (lores pixels) and positions in the world.
///
/// The map is drawn into a bitmap and the display scrolls that bitmap by the part of the map's
/// position that is not a whole tile, so a world position is simply its distance from the map's
/// top left corner. The play field starts a little way in from the left edge of the screen.
public static class ScreenMap
{
    /// Where the play field starts across the screen (lores pixels).
    private const int PLAY_FIELD_LEFT = 16;

    /// The world position under a position on the screen.
    public static (int X, int Y) ToWorld(World world, (int X, int Y) screen) =>
        ((ushort)(world.MapX + screen.X - PLAY_FIELD_LEFT), (ushort)(world.MapY + screen.Y));

    /// Where a world position is on the screen (it can be off the play field). Because it is
    /// worked out from the map's position every frame, a marker stays on the same spot of ground
    /// as the map scrolls.
    public static (int X, int Y) ToScreen(World world, (int X, int Y) position) =>
        (unchecked((short)(position.X - world.MapX)) + PLAY_FIELD_LEFT, unchecked((short)(position.Y - world.MapY)));
}
