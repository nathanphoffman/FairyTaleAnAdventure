using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// A picture of the whole outdoor world for the minimap and the big map: one pixel for each tile,
/// 2048 across and 1024 down, in the colours the game's own overview map uses (the magic map
/// spell) at full daylight. It is read from the disk once, the first time it is wanted.
///
/// A tile is 16 pixels wide and 32 high in the world, so on screen each pixel of this picture is
/// drawn twice as tall as it is wide.
public static class WorldOverview
{
    public const int COLUMNS = 2048;
    public const int ROWS = 1024;

    // The world is 2 regions across and 4 down; a region is 64 sectors across and 32 down.
    private const int REGION_COUNT = 8;
    private const int REGION_SECTOR_COLUMN_SHIFT = 6;
    private const int REGION_SECTOR_ROW_SHIFT = 5;
    private const int REGION_SECTOR_COLUMNS = 64;
    private const int REGION_SECTOR_ROWS = 32;

    // A sector is 16 tiles across and 8 down, 128 tile numbers; the region map has 128 sectors to a row.
    private const int SECTOR_TILE_COLUMNS = 16;
    private const int SECTOR_TILE_ROWS = 8;
    private const int SECTOR_TILES = 128;
    private const int MAP_ROW_SHIFT = 7;

    // The terrain table has four bytes for each tile; the last is the colour on the overview map.
    private const int TERRAIN_ENTRY_BYTES = 4;
    private const int TERRAIN_COLOR_BYTE = 3;
    private const int PALETTE_MASK = 31;
    private const uint OPAQUE = 0xff00_0000;

    // A world position to the tile it is on (tiles are 16 pixels wide and 32 high).
    private const int TILE_COLUMN_SHIFT = 4;
    private const int TILE_ROW_SHIFT = 5;

    private static readonly Lazy<uint[]> pixels = new(Build);

    /// The picture (0xAARRGGBB), COLUMNS pixels to a row. The first call reads the disk.
    public static uint[] Pixels => pixels.Value;

    /// The column and row of the picture a world position is on.
    public static (int Column, int Row) TileOf(int worldX, int worldY) =>
        (Math.Clamp(worldX >> TILE_COLUMN_SHIFT, 0, COLUMNS - 1), Math.Clamp(worldY >> TILE_ROW_SHIFT, 0, ROWS - 1));

    // Reads every region's map and paints it into the picture.
    private static uint[] Build()
    {
        uint[] palette = ObjectTables.InitialPageColors.Select(color => OPAQUE | Scene.Rgb4(color)).ToArray();
        uint[] picture = new uint[COLUMNS * ROWS];

        for (int region = 0; region < REGION_COUNT; region++)
        {
            PaintRegion(picture, palette, region);
        }

        return picture;
    }

    // Reads one region's sector table and region map from the disk and paints its sectors.
    private static void PaintRegion(uint[] picture, uint[] palette, int region)
    {
        MapData map = new();

        map.Allocate();
        Regions.LoadMapTables(map, Tables.RegionDiskBlocks[region]);
        Regions.LoadTerrainTables(map, Tables.RegionDiskBlocks[region]);

        int firstColumn = (region & 1) << REGION_SECTOR_COLUMN_SHIFT;
        int firstRow = (region >> 1) << REGION_SECTOR_ROW_SHIFT;

        for (int row = firstRow; row < firstRow + REGION_SECTOR_ROWS; row++)
        {
            for (int column = firstColumn; column < firstColumn + REGION_SECTOR_COLUMNS; column++)
            {
                int sector = map.MapByte((ushort)(((row - firstRow) << MAP_ROW_SHIFT) + column));

                PaintSector(picture, palette, map, sector, (column * SECTOR_TILE_COLUMNS, row * SECTOR_TILE_ROWS));
            }
        }
    }

    // Paints the 16 x 8 tiles of one sector at a tile column and row of the world.
    private static void PaintSector(uint[] picture, uint[] palette, MapData map, int sector, (int Column, int Row) at)
    {
        for (int tileRow = 0; tileRow < SECTOR_TILE_ROWS; tileRow++)
        {
            for (int tileColumn = 0; tileColumn < SECTOR_TILE_COLUMNS; tileColumn++)
            {
                int tile = map.SectorByte(sector * SECTOR_TILES + tileRow * SECTOR_TILE_COLUMNS + tileColumn);
                byte color = map.TerrainByte(tile * TERRAIN_ENTRY_BYTES + TERRAIN_COLOR_BYTE);

                picture[(at.Row + tileRow) * COLUMNS + at.Column + tileColumn] = palette[color & PALETTE_MASK];
            }
        }
    }
}
