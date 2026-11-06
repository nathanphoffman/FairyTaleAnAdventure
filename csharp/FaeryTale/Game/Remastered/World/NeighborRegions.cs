using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Remastered;

/// The regions next to the one the game has loaded, so the new world drawer can show the land past
/// the edge of the loaded region (the game itself only keeps one region in memory, and its small
/// screen rarely showed that edge). Each is read from the disk once, the first time it is in view,
/// with its own tile pictures, and only the ground is drawn: its people and objects appear when the
/// game loads the region.
public sealed class NeighborRegions
{
    /// Tile numbers of a neighbour are given this much above its region number plus one, so the one
    /// number says both which region's pictures to use and which tile.
    public const int REGION_STEP = 256;

    private const int TILE_NUMBER_MASK = 255;
    private const int REGION_COLUMNS = 2;
    private const int REGION_ROWS = 4;
    private const int REGION_SECTOR_COLUMN_SHIFT = 6;
    private const int REGION_SECTOR_ROW_SHIFT = 5;
    private const int WORLD_SECTOR_COLUMNS = 128;
    private const int WORLD_SECTOR_ROWS = 128;
    private const int SECTOR_COLUMN_SHIFT = 4;
    private const int SECTOR_ROW_SHIFT = 3;
    private const int MAP_ROW_SHIFT = 7;
    private const int SECTOR_TILES = 128;
    private const int SECTOR_ROW_TILES = 16;
    private const int SECTOR_TILE_ROWS = 8;
    private const int TILE_BANKS = 4;

    /// A region's own data: the sector table and region map, the tile pictures and their decoded copy.
    private sealed record Snapshot(MapData Map, Graphics Graphics, WorldImages Images, int OriginX, int OriginY);

    private readonly Snapshot?[] snapshots = new Snapshot?[REGION_COLUMNS * REGION_ROWS];

    /// Loads the regions that tiles in this range (tile columns and rows of the world) belong to,
    /// when they are not the loaded one. It reads from the disk, so it is done before the tiles are
    /// drawn on several threads.
    public void Prepare(Data.Game game, (int First, int Last) columns, (int First, int Last) rows)
    {
        if (game.World.Region >= Regions.FIRST_INDOOR_REGION)
        {
            return;
        }

        foreach (int region in RegionsIn(columns, rows))
        {
            if (region != game.World.Region && snapshots[region] is null)
            {
                snapshots[region] = Load(region);
            }
        }
    }

    /// The tile at a tile column and row of the world if it is in a loaded neighbour: its tile
    /// number plus REGION_STEP times (its region + 1), or -1 when there is none.
    public int TileAt(Data.Game game, int column, int row)
    {
        int sectorColumn = column >> SECTOR_COLUMN_SHIFT;
        int sectorRow = row >> SECTOR_ROW_SHIFT;

        if (sectorColumn >= WORLD_SECTOR_COLUMNS || sectorRow >= WORLD_SECTOR_ROWS)
        {
            return -1;
        }

        int region = RegionOf(sectorColumn, sectorRow);

        if (region == game.World.Region || snapshots[region] is not { } snapshot)
        {
            return -1;
        }

        int sector = snapshot.Map.MapByte(((sectorRow - snapshot.OriginY) << MAP_ROW_SHIFT) + sectorColumn);
        int tile = snapshot.Map.SectorByte(sector * SECTOR_TILES + (row % SECTOR_TILE_ROWS) * SECTOR_ROW_TILES + (column % SECTOR_ROW_TILES));

        return tile + REGION_STEP * (region + 1);
    }

    /// The picture of a tile given by TileAt.
    public byte[] Picture(int tileAndRegion)
    {
        Snapshot snapshot = snapshots[tileAndRegion / REGION_STEP - 1]!;

        return snapshot.Images.Tile(snapshot.Graphics, tileAndRegion & TILE_NUMBER_MASK);
    }

    /// Makes the decoded pictures of every loaded neighbour ready (before the tiles are drawn on
    /// several threads, which only add to them).
    public void WarmUp()
    {
        foreach (Snapshot? snapshot in snapshots)
        {
            snapshot?.Images.Tile(snapshot.Graphics, 0);
        }
    }

    // The regions the sectors of a range of tiles are in.
    private static HashSet<int> RegionsIn((int First, int Last) columns, (int First, int Last) rows)
    {
        HashSet<int> regions = [];
        int firstColumn = Math.Clamp(columns.First >> SECTOR_COLUMN_SHIFT, 0, WORLD_SECTOR_COLUMNS - 1);
        int lastColumn = Math.Clamp(columns.Last >> SECTOR_COLUMN_SHIFT, 0, WORLD_SECTOR_COLUMNS - 1);
        int firstRow = Math.Clamp(rows.First >> SECTOR_ROW_SHIFT, 0, WORLD_SECTOR_ROWS - 1);
        int lastRow = Math.Clamp(rows.Last >> SECTOR_ROW_SHIFT, 0, WORLD_SECTOR_ROWS - 1);

        foreach (int column in new[] { firstColumn, lastColumn })
        {
            foreach (int row in new[] { firstRow, lastRow })
            {
                regions.Add(RegionOf(column, row));
            }
        }

        return regions;
    }

    // The region a sector of the world is in: two across, four down.
    private static int RegionOf(int sectorColumn, int sectorRow) =>
        ((sectorColumn >> REGION_SECTOR_COLUMN_SHIFT) & 1) + REGION_COLUMNS * ((sectorRow >> REGION_SECTOR_ROW_SHIFT) & (REGION_ROWS - 1));

    // Reads a region's map and tile pictures from the disk.
    private static Snapshot Load(int region)
    {
        Need need = Tables.RegionDiskBlocks[region];
        MapData map = new();
        Graphics graphics = new() { Image = new byte[Graphics.IMAGE_BYTES] };

        map.Allocate();
        Regions.LoadMapTables(map, need);

        for (int bank = 0; bank < TILE_BANKS; bank++)
        {
            graphics.LoadImageBank(bank, (short)need.Image[bank]);
        }

        return new Snapshot(map, graphics, new WorldImages(), (region & 1) << REGION_SECTOR_COLUMN_SHIFT, (region >> 1) << REGION_SECTOR_ROW_SHIFT);
    }
}
