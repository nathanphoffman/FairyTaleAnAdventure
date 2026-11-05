using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Land.TravelDoors;

/// Going through the doors between the outdoors and the buildings and caves. Doors that need a key
/// are in KeyDoors.
public static class TravelDoors
{
    internal const int DOOR_COUNT = 86;
    internal const sbyte CAVE = 18;

    private const ushort FIRST_INSIDE_REGION = 8;

    // The hero is lined up with a door when these bits of the position are in range: a tile is 16
    // pixels wide and 32 high, so bit 4 of y says which half of the tile row.
    internal const ushort LOWER_HALF_BIT = 0x10;
    internal const ushort TILE_COLUMN_MASK = 15;
    internal const int TILE_WIDTH = 16;

    // Snaps the hero's position to a tile (16 wide, 32 high).
    private const ushort TILE_X_MASK = 0xfff0;
    private const ushort TILE_Y_MASK = 0xffe0;

    /// Is the hero standing at a door? Outdoors the doors are in sorted order and found by binary
    /// search; indoors they are searched in turn. If so, the hero is moved to the other side.
    /// Riding disables doors.
    public static void CheckDoor(Data.Game game, IOutput output)
    {
        if (game.Hero.Riding != 0)
        {
            return;
        }

        ushort x = (ushort)(game.Hero.X & TILE_X_MASK);
        ushort y = (ushort)(game.Hero.Y & TILE_Y_MASK);

        if (game.World.Region < FIRST_INSIDE_REGION)
        {
            EnteringDoors.EnterDoor(game, output, x, y);
        }
        else
        {
            LeavingDoors.LeaveDoor(game, output, x, y);
        }
    }
}
