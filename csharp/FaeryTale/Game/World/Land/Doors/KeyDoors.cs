using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Land;

/// Opening doors with keys.
///
/// A door on the map is a few tiles that look like a door. Walking into one with a key (or none
/// needed) swaps those tiles for the "open" ones. OPEN_LIST says, per door and per map set, which
/// tiles to swap in and which key is needed.
public static class KeyDoors
{
    // Key types.
    public const byte NOKEY = 0;
    public const byte GOLD = 1;
    public const byte GREEN = 2;
    public const byte KBLUE = 3;
    public const byte RED = 4;
    public const byte GREY = 5;
    public const byte WHITE = 6;

    // How the second new tile is placed, relative to the first.
    private const byte ABOVE = 1;
    private const byte SIDE = 2;
    private const byte BACK = 3;
    private const byte CABINET_LAYOUT = 4;

    /// The terrain type that marks a door.
    private const int DOOR_TERRAIN = 15;

    // A door is looked for this far (pixels) to the right and left, and a tile is 16 pixels wide
    // and 32 high (shifts 4 and 5). Each map set covers 64 sector numbers (a shift of 6).
    private const ushort DOOR_SEARCH_SIDE = 4;
    private const ushort TILE_WIDTH = 16;
    private const ushort TILE_HEIGHT = 32;
    private const int TILE_WIDTH_SHIFT = 4;
    private const int TILE_HEIGHT_SHIFT = 5;
    private const int MAP_SET_SHIFT = 6;

    /// A door can be up to three tiles wide, so at most two steps to its left edge.
    private const int DOOR_MAX_LEFT_STEPS = 2;

    /// The tiles of an opened cabinet: the open door's top, and the three around it.
    private static readonly byte[] CABINET_TILES = [87, 86, 88];

    /// One kind of door: DoorId is the tile it starts as, in the map set MapId; it turns into New1
    /// (and New2 next to it, placed by Above); KeyType is the key that opens it.
    internal sealed record DoorOpen(byte DoorId, ushort MapId, byte New1, byte New2, byte Above, byte KeyType);

    internal static readonly DoorOpen[] OPEN_LIST =
    [
        new(64, 360, 123, 124, SIDE, GREEN),          // HSTONE
        new(120, 360, 125, 126, SIDE, NOKEY),         // HWOOD
        new(122, 360, 127, 0, 0, NOKEY),              // VWOOD
        new(64, 280, 124, 125, SIDE, GREY),           // HSTONE2
        new(77, 280, 126, 0, 0, GREY),                // VSTONE2
        new(82, 480, 84, 85, SIDE, KBLUE),            // CRYST
        new(64, 480, 105, 106, SIDE, GREEN),          // OASIS
        new(128, 240, 154, 155, ABOVE, WHITE),        // MARBLE
        new(39, 680, 41, 42, SIDE, GOLD),             // HGATE
        new(25, 680, 27, 26, BACK, GOLD),             // VGATE
        new(114, 760, 116, 117, ABOVE, RED),          // SECRET
        new(118, 760, 116, 117, ABOVE, GREY),         // TUNNEL
        new(136, 800, 133, 134, 135, GOLD),           // GOLDEN
        new(187, 800, 76, 77, SIDE, NOKEY),           // HSTON3
        new(73, 720, 75, 0, 0, NOKEY),                // VSTON3
        new(165, 800, 85, 86, CABINET_LAYOUT, GREEN), // CABINET
        new(210, 840, 208, 209, SIDE, NOKEY),         // BLUE
    ];

    /// Tries to open a door near the pixel (x, y) with a key of type keyType (0 when walking into
    /// it without a key). Looks 4 pixels either side for the door terrain, then walks left and
    /// down to the door's top-left tile. Returns 1 if a door opened, 0 if not ("It's locked." is
    /// said the first time).
    public static int OpenDoorNear(Data.Game game, IOutput output, ushort x, ushort y, uint keyType)
    {
        if (FindDoorCorner(game, x, y) is not { } corner)
        {
            return 0;
        }

        // We know where the door starts; now which type is it?
        byte sectorId = Tile(game, corner.X, corner.Y);

        // The high bits of the sector number say which map set it belongs to.
        ushort mapId = game.Map.LoadedBlocks.Image[sectorId >> MAP_SET_SHIFT];

        if (FindOpen(sectorId, mapId, keyType) is { } open)
        {
            SwapInOpenDoorTiles(game, open, corner.X, corner.Y);
            game.Ui.ViewStatus = ViewStatus.REBUILD_ALL;
            game.Ui.HeldDirection = 0;
            output.Print("It opened.");

            return 1;
        }

        if (game.World.BumpedLockedDoor == 0 && keyType == 0)
        {
            output.Print("It's locked.");
        }

        game.World.BumpedLockedDoor = 1;

        return 0;
    }

    /// The door kind that sectorId (in map set mapId) is, if the key keyType opens it (a door that
    /// needs no key opens for anything). A door of the right kind with the wrong key is skipped.
    internal static DoorOpen? FindOpen(byte sectorId, ushort mapId, uint keyType) =>
        OPEN_LIST.FirstOrDefault(open =>
            open.MapId == mapId && open.DoorId == sectorId && (open.KeyType == NOKEY || open.KeyType == keyType));

    /// The tiles an opened door changes, as (x, y, new tile). The door's top-left tile is (x, y).
    internal static List<(int X, int Y, byte Tile)> OpenChanges(DoorOpen open, int x, int y)
    {
        List<(int X, int Y, byte Tile)> changes = [(x, y, open.New1)];
        byte second = open.New2;

        if (second == 0)
        {
            return changes;
        }

        switch (open.Above)
        {
            case ABOVE:
                changes.Add((x, y - 1, second));

                break;

            case BACK:
                changes.Add((x - 1, y, second));

                break;

            case CABINET_LAYOUT:
                changes.Add((x, y - 1, CABINET_TILES[0]));
                changes.Add((x + 1, y, CABINET_TILES[1]));
                changes.Add((x + 1, y - 1, CABINET_TILES[2]));

                break;

            default:
                changes.Add((x + 1, y, second));

                if (open.Above != SIDE)
                {
                    changes.Add((x + 2, y, open.Above));
                }

                break;
        }

        return changes;
    }

    // The tile number at tile position (x, y) of the map.
    private static byte Tile(Data.Game game, int x, int y) => game.Map.SectorByte(MapLookup.TileIndex(game, x, y));

    // Changes the tile number at tile position (x, y) of the map.
    private static void SetTile(Data.Game game, int x, int y, byte value) => game.Map.SetSectorByte(MapLookup.TileIndex(game, x, y), value);

    // Looks for a door tile at the point, 4 pixels right of it, or 4 pixels left of it, then moves
    // to the door's top-left tile. Returns that tile's column and row, or null if there is no door.
    private static (int X, int Y)? FindDoorCorner(Data.Game game, ushort x, ushort y)
    {
        bool found = MapLookup.TerrainAt(game, x, y) == DOOR_TERRAIN;

        if (!found)
        {
            x += DOOR_SEARCH_SIDE;
            found = MapLookup.TerrainAt(game, x, y) == DOOR_TERRAIN;
        }

        if (!found)
        {
            x -= 2 * DOOR_SEARCH_SIDE;
            found = MapLookup.TerrainAt(game, x, y) == DOOR_TERRAIN;
        }

        if (!found)
        {
            return null;
        }

        // A door can be up to three tiles wide; step to its left edge.
        for (int step = 0; step < DOOR_MAX_LEFT_STEPS; step++)
        {
            if (MapLookup.TerrainAt(game, x - TILE_WIDTH, y) == DOOR_TERRAIN)
            {
                x -= TILE_WIDTH;
            }
        }

        if (MapLookup.TerrainAt(game, x, y + TILE_HEIGHT) == DOOR_TERRAIN)
        {
            y += TILE_HEIGHT;
        }

        // Pixel position to tile column and row (shift = divide by the tile size).
        return (x >> TILE_WIDTH_SHIFT, y >> TILE_HEIGHT_SHIFT);
    }

    // Replaces the closed door tile at (x, y) with its open version, and the neighbouring tiles the
    // door covers (which ones depends on the door's layout).
    private static void SwapInOpenDoorTiles(Data.Game game, DoorOpen open, int x, int y)
    {
        foreach ((int tileX, int tileY, byte value) in OpenChanges(open, x, y))
        {
            SetTile(game, tileX, tileY, value);
        }
    }
}
