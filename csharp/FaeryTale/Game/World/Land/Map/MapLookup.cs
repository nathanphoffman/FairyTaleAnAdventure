using FaeryTale.Game.Data;

namespace FaeryTale.Game.Land.Map;

/// The map routines that were 68000 assembly in the original (fsubs.asm). Behaviour, including
/// 16 bit wrap-around, follows the assembly.
///
/// World coordinates are 15 bit. A tile ("image") is 16 pixels wide and 32 tall; a sector is
/// 16 x 8 tiles; the region map is 128 x 32 sectors, one byte each.
///
/// Looking up the ground under a pixel goes: pixel -> tile column/row -> which sector (from the
/// region map) -> which tile (from that sector in the sector table) -> the terrain table for that
/// tile. The odd wrap-around tricks (WrapSectorColumn, ClampSectorRow) copy what the assembly did
/// when the position is outside the loaded region.
public static class MapLookup
{
    /// The play field is this many tiles wide and tall.
    public const int SCREEN_COLUMNS = 19;
    public const int SCREEN_ROWS = 6;

    /// Sectors are 16 x 8 tiles of 128 bytes; the region map has 128 sector bytes per row.
    public const int SECTOR_BYTES = 128;
    public const short SECTOR_ROWS_IN_REGION = 32;

    private const int MAP_ROW_BYTES = 128;

    // Shifts and masks that go between pixels, tiles and sectors.
    private const int TILE_WIDTH_SHIFT = 4;
    private const int TILE_HEIGHT_SHIFT = 5;
    private const int SECTOR_WIDTH_SHIFT = 4;
    private const int SECTOR_HEIGHT_SHIFT = 3;
    private const int TILE_COLUMN_MASK = 15;
    private const int TILE_ROW_MASK = 7;
    private const int SECTOR_ROW_TILES = 16;

    // A sector row of the region map is 128 bytes: the shift that multiplies by it.
    private const int MAP_ROW_SHIFT = 7;

    // A sector column outside the map: bit 6 set means off the edge, and bit 5 says which edge.
    private const int COLUMN_OFF_MAP = 64;
    private const int COLUMN_OFF_LOW_SIDE = 32;
    private const int LAST_SECTOR_COLUMN = 63;
    private const int ROW_OFF_MAP = 32;
    private const int ROW_OFF_LOW_SIDE = 16;
    private const int LAST_SECTOR_ROW = 31;

    // A terrain table entry is 4 bytes: the second holds the type in its high nibble, the third
    // the mask of solid sub-areas, and the fourth the overview colour.
    private const int TERRAIN_ENTRY_BYTES = 4;
    private const int TERRAIN_TYPE_BYTE = 1;
    private const int TERRAIN_MASK_BYTE = 2;
    private const int TERRAIN_TYPE_SHIFT = 4;

    // The 8 sub-areas of a tile are picked by one bit of the mask, from the pixel position.
    private const byte FIRST_SUBAREA_BIT = 0x80;
    private const int RIGHT_HALF_PIXEL = 8;
    private const int LOWER_ROW_PIXEL = 8;
    private const int LOWER_PAIR_PIXEL = 16;
    private const int RIGHT_HALF_SHIFT = 4;

    // Collision probes: how far right/left and down from the actor's position, and which terrain
    // types block on each side.
    private const int PROBE_SIDE = 4;
    private const int PROBE_DOWN = 2;
    private const int ALWAYS_BLOCKS = 1;
    private const int RIGHT_PROBE_BLOCKS_FROM = 10;
    private const int LEFT_PROBE_BLOCKS_FROM = 8;

    // The minimap covers the world's 2048 tile columns, and 32768 tile rows.
    private const int TILE_COLUMN_WRAP = 0x07ff;
    private const int TILE_ROW_WRAP = 0x7fff;

    // Regions above this number use the second half of the sector table.
    private const int LAST_FIRST_HALF_REGION = 7;
    private const int REGION_BYTE_MASK = 0xff;
    private const int SECOND_HALF_OFFSET = 256;

    // The high byte of the hero's position is the sector.
    private const int SECTOR_POSITION_SHIFT = 8;

    /// Sector column from an image column: values outside 0-63 wrap, as in the assembly.
    internal static ushort WrapSectorColumn(ushort value)
    {
        if ((value & COLUMN_OFF_MAP) != 0)
        {
            return (value & COLUMN_OFF_LOW_SIDE) != 0 ? (ushort)0 : (ushort)LAST_SECTOR_COLUMN;
        }

        return value;
    }

    /// Sector row from an image row; rows outside 0-31 are clamped.
    internal static ushort ClampSectorRow(short value) => (ushort)Math.Clamp((int)value, 0, SECTOR_ROWS_IN_REGION - 1);

    /// The sector number for a sector column and row of the region map.
    private static byte SectorAt(Data.Game game, ushort column, ushort row) =>
        game.Map.MapByte((ushort)(row * MAP_ROW_BYTES + column + game.World.OriginX));

    /// Terrain type (0-15) at a pixel position, 0 if that sub-area is clear.
    ///
    /// Each tile is divided into 8 sub-areas (2 across, 4 down); the terrain table has a bit mask
    /// saying which of them are solid, so a tile can be partly walkable (a tree trunk in the
    /// middle of a clearing, say). The bit is picked from pixel bits 3 (x), 3 and 4 (y).
    public static int TerrainAt(Data.Game game, int x, int y)
    {
        ushort ux = (ushort)x;
        ushort uy = (ushort)y;
        byte bit = SubareaBit(ux, uy);

        // The tile column and row: tiles are 16 pixels wide and 32 high.
        int entry = TileNumber(game, (ushort)(ux >> TILE_WIDTH_SHIFT), (ushort)(uy >> TILE_HEIGHT_SHIFT)) * TERRAIN_ENTRY_BYTES;

        if ((game.Map.TerrainByte(entry + TERRAIN_MASK_BYTE) & bit) == 0)
        {
            return 0;
        }

        // The terrain type is the high nibble of the second byte of the tile's entry.
        return game.Map.TerrainByte(entry + TERRAIN_TYPE_BYTE) >> TERRAIN_TYPE_SHIFT;
    }

    // The bit of a tile's solid-area mask for the sub-area at pixel (ux, uy).
    private static byte SubareaBit(ushort ux, ushort uy)
    {
        byte bit = FIRST_SUBAREA_BIT;

        if ((ux & RIGHT_HALF_PIXEL) != 0)
        {
            // The right half of the tile: the second column of sub-areas.
            bit >>= RIGHT_HALF_SHIFT;
        }

        if ((uy & LOWER_ROW_PIXEL) != 0)
        {
            // The lower half of a 16 pixel row of sub-areas.
            bit >>= 1;
        }

        if ((uy & LOWER_PAIR_PIXEL) != 0)
        {
            // The lower two of the four rows of sub-areas.
            bit >>= 2;
        }

        return bit;
    }

    /// Collision test with two probes either side of the character (4 pixels right and left, 2
    /// down). Returns the blocking terrain type, 0 if the way is clear. Type 1 always blocks; the
    /// right-hand probe is blocked by 10 and above, the left-hand one by 8 and above.
    public static int ProbeCollision(Data.Game game, int x, int y)
    {
        int right = TerrainAt(game, x + PROBE_SIDE, y + PROBE_DOWN);

        if (right == ALWAYS_BLOCKS || right >= RIGHT_PROBE_BLOCKS_FROM)
        {
            return right;
        }

        int left = TerrainAt(game, x - PROBE_SIDE, y + PROBE_DOWN);

        if (left == ALWAYS_BLOCKS || left >= LEFT_PROBE_BLOCKS_FROM)
        {
            return left;
        }

        return 0;
    }

    /// Index into the sector table of the tile byte for a tile position (wraps vertically too).
    public static int TileIndex(Data.Game game, int imx, int imy)
    {
        ushort ux = (ushort)imx;
        ushort uy = (ushort)imy;

        // Pixel column to sector column: 16 pixels to a tile, 16 tiles to a sector.
        ushort column = WrapSectorColumn((ushort)((ux >> TILE_WIDTH_SHIFT) - game.World.OriginX));

        // Pixel row to sector row: 8 tiles to a sector, so 8 times 32 divided by 32.
        ushort row = (ushort)((uy >> SECTOR_HEIGHT_SHIFT) - game.World.OriginY);

        if ((row & ROW_OFF_MAP) != 0)
        {
            row = (row & ROW_OFF_LOW_SIDE) != 0 ? (ushort)0 : (ushort)LAST_SECTOR_ROW;
        }

        int sector = SectorAt(game, column, row);

        return (ushort)(sector * SECTOR_BYTES + (uy & TILE_ROW_MASK) * SECTOR_ROW_TILES + (ux & TILE_COLUMN_MASK));
    }

    /// Fills the minimap (19 columns of 6 tile numbers, column by column) for the area starting
    /// at tile (tileX, tileY), and works out which sector the hero is in.
    public static void FillMinimap(Data.Game game, int tileX, int tileY)
    {
        ushort x = (ushort)tileX;
        int next = 0;

        for (int column = 0; column < SCREEN_COLUMNS; column++)
        {
            x &= TILE_COLUMN_WRAP;

            for (int row = 0; row < SCREEN_ROWS; row++)
            {
                ushort y = (ushort)(tileY + row);

                game.Map.Minimap[next] = TileNumber(game, x, (ushort)(y & TILE_ROW_WRAP));
                next++;
            }

            x++;
        }

        game.Hero.Sector = HeroSector(game);
    }

    // The tile number at tile column x and row y of the world.
    public static byte TileNumber(Data.Game game, ushort x, ushort y)
    {
        // Tile column to sector column: 16 tiles to a sector.
        ushort column = (ushort)(WrapSectorColumn((ushort)((x >> SECTOR_WIDTH_SHIFT) - game.World.OriginX)) + game.World.OriginX);

        // Tile row to sector row: 8 tiles to a sector.
        ushort row = ClampSectorRow((short)((y >> SECTOR_HEIGHT_SHIFT) - game.World.OriginY));

        // Row and column become one byte offset into the map: the row is shifted up, the column added.
        int sector = game.Map.MapByte((ushort)((row << MAP_ROW_SHIFT) + column));

        return game.Map.SectorByte((ushort)(sector * SECTOR_BYTES + (y & TILE_ROW_MASK) * SECTOR_ROW_TILES + (x & TILE_COLUMN_MASK)));
    }

    // The sector the hero is in; the second half of the sector table is used from region 8 on.
    private static short HeroSector(Data.Game game)
    {
        // The assembly read the high byte of hero x and y (the 68000 is big-endian).
        int heroColumn = game.Hero.X >> SECTOR_POSITION_SHIFT;

        // The hero's sector row: 256 pixels to a sector, wrapping like the original 16 bit math.
        int heroRow = (short)((ushort)((game.Hero.Y >> SECTOR_POSITION_SHIFT) - game.World.OriginY));

        // The region map has 128 sectors in each row.
        int sector = game.Map.MapByte((ushort)((heroRow << MAP_ROW_SHIFT) + heroColumn));

        if ((game.World.Region & REGION_BYTE_MASK) > LAST_FIRST_HALF_REGION)
        {
            sector += SECOND_HALF_OFFSET;
        }

        return (short)sector;
    }
}
