using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Remastered;

/// WorldRenderer: the tiles of the map.
public sealed partial class WorldRenderer
{
    // The world pixel under every column and every row of the picture.
    private void MapPositions(WorldView view)
    {
        if (columnToWorld.Length != view.Width)
        {
            columnToWorld = new int[view.Width];
        }

        if (rowToWorld.Length != view.Height)
        {
            rowToWorld = new int[view.Height];
        }

        for (int x = 0; x < view.Width; x++)
        {
            columnToWorld[x] = (int)Math.Floor(view.ToWorld(x + 0.5, 0).X);
        }

        for (int y = 0; y < view.Height; y++)
        {
            rowToWorld[y] = (int)Math.Floor(view.ToWorld(0, y + 0.5).Y);
        }
    }

    // The map. The picture is cut into bands of rows that are drawn at the same time on the machine's
    // cores; within a band rows that fall on the same world row are copied instead of drawn again.
    private void DrawTiles(uint[] frame, WorldView view, Data.Game game, Graphics graphics)
    {
        int firstColumn = columnToWorld[0] >> TILE_WIDTH_SHIFT;
        int lastColumn = columnToWorld[view.Width - 1] >> TILE_WIDTH_SHIFT;
        int bands = Math.Clamp(Environment.ProcessorCount, 1, MAX_BANDS);
        int rowsPerBand = (view.Height + bands - 1) / bands;

        // The land past the loaded region comes from the regions next to it, read once when it comes
        // into view. The decoded tiles are looked after once here, so the bands only ever add to them.
        neighbors.Prepare(game, (firstColumn, lastColumn), (rowToWorld[0] >> TILE_HEIGHT_SHIFT, rowToWorld[view.Height - 1] >> TILE_HEIGHT_SHIFT));
        neighbors.WarmUp();
        images.Tile(graphics, 0);

        Parallel.For(0, bands, band =>
        {
            int top = band * rowsPerBand;

            DrawTileBand(frame, view, game, graphics, (top, Math.Min(top + rowsPerBand, view.Height)), (firstColumn, lastColumn));
        });
    }

    // The rows from top up to (not including) bottom of the picture.
    private void DrawTileBand(uint[] frame, WorldView view, Data.Game game, Graphics graphics, (int Top, int Bottom) rows, (int First, int Last) columns)
    {
        int[] tileNumbers = new int[columns.Last - columns.First + 1];
        int previousWorldRow = int.MinValue;

        for (int y = rows.Top; y < rows.Bottom; y++)
        {
            int worldRow = rowToWorld[y];

            if (y > rows.Top && worldRow == previousWorldRow)
            {
                Array.Copy(frame, (y - 1) * view.Width, frame, y * view.Width, view.Width);

                continue;
            }

            previousWorldRow = worldRow;
            FillTileRow(game, tileNumbers, columns.First, worldRow >> TILE_HEIGHT_SHIFT);
            DrawTileLine(frame, view, graphics, (y, worldRow), tileNumbers, columns.First);
        }
    }

    // The tile numbers of one row of tiles across the picture (-1 where there is no map).
    private void FillTileRow(Data.Game game, int[] tileNumbers, int firstColumn, int row)
    {
        for (int index = 0; index < tileNumbers.Length; index++)
        {
            int column = firstColumn + index;
            bool hidden = interior.Hides(column, row);
            int number = hidden ? -1 : TileAt(game, column, row);

            tileNumbers[index] = number >= 0 || hidden || column < 0 || row < 0 ? number : neighbors.TileAt(game, column, row);
        }
    }

    // The tile at a tile column and row of the world, or -1 outside the region that is loaded
    // (its pictures are not in memory).
    private static int TileAt(Data.Game game, int column, int row)
    {
        if (column < 0 || row < 0 || (column << TILE_WIDTH_SHIFT) >= WORLD_WIDTH || (row << TILE_HEIGHT_SHIFT) >= WORLD_HEIGHT)
        {
            return -1;
        }

        int sectorColumn = (column >> SECTOR_COLUMN_SHIFT) - game.World.OriginX;
        int sectorRow = (row >> SECTOR_ROW_SHIFT) - game.World.OriginY;

        if (sectorColumn < 0 || sectorColumn >= REGION_SECTOR_COLUMNS || sectorRow < 0 || sectorRow >= REGION_SECTOR_ROWS)
        {
            return -1;
        }

        return MapLookup.TileNumber(game, (ushort)(column & TILE_COLUMN_WRAP), (ushort)(row & TILE_ROW_WRAP));
    }

    // One line of the picture from the tiles.
    private void DrawTileLine(uint[] frame, WorldView view, Graphics graphics, (int Y, int WorldRow) line, int[] tileNumbers, int firstTileColumn)
    {
        int tileLine = line.WorldRow & (WorldImages.TILE_HEIGHT - 1);
        int rowStart = line.Y * view.Width;
        int lastTile = int.MinValue;
        byte[]? tile = null;

        for (int x = 0; x < view.Width; x++)
        {
            int worldColumn = columnToWorld[x];
            int tileColumn = worldColumn >> TILE_WIDTH_SHIFT;

            if (tileColumn != lastTile)
            {
                lastTile = tileColumn;
                int number = tileNumbers[tileColumn - firstTileColumn];

                tile = number < 0 ? null : number >= NeighborRegions.REGION_STEP ? neighbors.Picture(number) : images.Tile(graphics, number);
            }

            frame[rowStart + x] = tile is null ? OFF_MAP_COLOR : palette[tile[tileLine * WorldImages.TILE_WIDTH + (worldColumn & (WorldImages.TILE_WIDTH - 1))]];
        }
    }
}
