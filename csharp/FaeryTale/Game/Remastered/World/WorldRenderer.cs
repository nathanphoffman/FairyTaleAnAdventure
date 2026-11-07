using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The new world drawer of Remastered and Reimagined: draws the map tiles and the sprites the game
/// has already loaded straight into a picture of any size, at any zoom, from where the camera is.
/// It reads the same data as the old drawing (and the same palette, so the day and night colours and
/// fades come out the same) but needs none of the old screen, so the view can show much more.
///
/// A frame can be drawn between two game ticks: alpha is how far from the last tick to the present
/// one it is (1 is the present tick), and the camera and the sprites are drawn that far along.
public sealed class WorldRenderer
{
    // The camera looks a little above the hero's feet so the hero is in the middle of the picture.
    private const double CAMERA_LIFT = 12;

    private const int TILE_COLUMN_WRAP = 0x7ff;
    private const int TILE_ROW_WRAP = 0x7fff;

    // Outdoors the world is 32768 pixels across and high, but the rooms inside buildings are further
    // down the same map (their y positions go past 32768, up to 65536).
    private const int WORLD_WIDTH = 0x8000;
    private const int WORLD_HEIGHT = 0x10000;
    private const int PALETTE_SIZE = 32;

    /// The most bands of rows the map is drawn in at the same time.
    private const int MAX_BANDS = 16;
    private const uint OFF_MAP_COLOR = 0xff000000;
    private const uint OPAQUE = 0xff000000;

    // A tile is 16 pixels wide and 32 high; a sector is 16 x 8 tiles; a region is 64 x 32 sectors.
    private const int TILE_WIDTH_SHIFT = 4;
    private const int TILE_HEIGHT_SHIFT = 5;
    private const int SECTOR_COLUMN_SHIFT = 4;
    private const int SECTOR_ROW_SHIFT = 3;
    private const int REGION_SECTOR_COLUMNS = 64;
    private const int REGION_SECTOR_ROWS = 32;

    // Scenery that hides sprites: the terrain table entry is four bytes (the first picks the mask).
    private const int TERRAIN_ENTRY_BYTES = 4;
    private const int TERRAIN_TYPE_MASK = 15;
    private const int SHADOW_MASK_BYTES = 64;
    private const int SHADOW_ROW_BYTES = 2;
    private const int RAFT_SLOT = 1;

    private readonly WorldImages images = new();
    private readonly NeighborRegions neighbors = new();
    private readonly InteriorMask interior = new();
    private readonly uint[] palette = new uint[PALETTE_SIZE];
    private int[] columnToWorld = [];
    private int[] rowToWorld = [];

    /// The colours of the play field now (colour number to 0xAARRGGBB).
    public uint[] Palette => palette;

    /// The picture of a captured sprite, decoded (null when it is not in memory).
    public SpriteImage? PictureOf(Video video, CapturedSprite sprite) => images.Sprite(video.Graphics, sprite.Sheet, sprite.Inum);

    /// The picture of an object by its number (the pictures of the items lying about and of the pack),
    /// decoded (null when it is not in memory).
    public SpriteImage? ObjectPicture(Video video, int number) =>
        images.Sprite(video.Graphics, SpriteImages.SpriteSheet(video, SpriteImages.OBJECTS), number);

    /// Draws the world into frame (view.Width x view.Height pixels).
    public void Draw(uint[] frame, WorldView view, WorldCapture capture, Video video, double alpha)
    {
        if (capture.Game is not { } game)
        {
            Array.Fill(frame, OFF_MAP_COLOR);

            return;
        }

        capture.Prepare();
        LoadPalette(video);
        PlaceCamera(view, capture, alpha);
        MapPositions(view);
        interior.Prepare(game);
        DrawTiles(frame, view, game, video.Graphics);
        DrawSprites(frame, view, capture, game, video, alpha);
    }

    // The 12 bit colours of the play field as 8 bit ones.
    private void LoadPalette(Video video)
    {
        ushort[] colors = video.Screen.GetViewPort(ViewPortId.Page).Colors;

        for (int index = 0; index < PALETTE_SIZE; index++)
        {
            palette[index] = OPAQUE | Scene.Rgb4(index < colors.Length ? colors[index] : (ushort)0);
        }
    }

    // The camera follows the hero, a part of the last tick's move behind the present position.
    private static void PlaceCamera(WorldView view, WorldCapture capture, double alpha)
    {
        view.CenterX = capture.HeroNow.X + capture.HeroDelta.X * (1 - alpha);
        view.CenterY = capture.HeroNow.Y + capture.HeroDelta.Y * (1 - alpha) - CAMERA_LIFT;
        LogCamera(view, capture, alpha);
    }

    // FTA_CAMERA_LOG=file: one line per picture (milliseconds, camera x, game step, alpha) to see
    // whether the camera moves evenly from picture to picture.
    private static readonly string? cameraLogPath = Environment.GetEnvironmentVariable("FTA_CAMERA_LOG");

    private static void LogCamera(WorldView view, WorldCapture capture, double alpha)
    {
        if (cameraLogPath is not null)
        {
            File.AppendAllText(cameraLogPath, $"{System.Diagnostics.Stopwatch.GetTimestamp() * 1000.0 / System.Diagnostics.Stopwatch.Frequency:F2} {view.CenterX:F2} {capture.Tick} {alpha:F3}\n");
        }
    }

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

    // The sprites in the order the old drawing drew them (back to front).
    private void DrawSprites(uint[] frame, WorldView view, WorldCapture capture, Data.Game game, Video video, double alpha)
    {
        foreach (CapturedSprite sprite in capture.Sprites)
        {
            if (!interior.HidesPosition(sprite.ActorX, sprite.ActorY) && images.Sprite(video.Graphics, sprite.Sheet, sprite.Inum) is { } image)
            {
                DrawSprite(frame, view, game, video, sprite, image, alpha);
            }
        }
    }

    private void DrawSprite(uint[] frame, WorldView view, Data.Game game, Video video, CapturedSprite sprite, SpriteImage image, double alpha)
    {
        double left = sprite.Left + sprite.DeltaX * (1 - alpha);
        double top = sprite.Top + sprite.DeltaY * (1 - alpha);
        int rows = Math.Min(sprite.Rows, image.Height - sprite.SkipRows);
        (double screenLeft, double screenTop) = view.ToScreen(left, top);
        double scale = view.Scale;
        int x0 = Math.Max((int)Math.Floor(screenLeft), 0);
        int x1 = Math.Min((int)Math.Ceiling(screenLeft + image.Width * scale), view.Width);
        int y0 = Math.Max((int)Math.Floor(screenTop), 0);
        int y1 = Math.Min((int)Math.Ceiling(screenTop + rows * scale), view.Height);

        if (rows <= 0 || x0 >= x1 || y0 >= y1)
        {
            return;
        }

        int leftPixel = (int)Math.Floor(left);
        int topPixel = (int)Math.Floor(top);
        SceneryHider hider = SceneryHider.For(game, video.Graphics, sprite, (leftPixel, topPixel), image.Width);

        for (int y = y0; y < y1; y++)
        {
            int row = (int)Math.Floor((y + 0.5 - screenTop) / scale);

            if (row < 0 || row >= rows)
            {
                continue;
            }

            int imageRow = (row + sprite.SkipRows) * image.Width;

            for (int x = x0; x < x1; x++)
            {
                int column = (int)Math.Floor((x + 0.5 - screenLeft) / scale);

                if (column < 0 || column >= image.Width || !image.Opaque[imageRow + column] || hider.Hides(leftPixel + column, topPixel + row))
                {
                    continue;
                }

                frame[y * view.Width + x] = palette[image.Pixels[imageRow + column]];
            }
        }
    }

    /// Hides the parts of a sprite that scenery (trees, walls, roofs) is in front of: the same rules
    /// and masks as the old drawing, worked out for each tile the sprite is over.
    private sealed class SceneryHider
    {
        private readonly int firstColumn;
        private readonly int firstRow;
        private readonly int columns;
        private readonly int[] maskOfCell;
        private readonly byte[] shadow;

        private SceneryHider(int firstColumn, int firstRow, int columns, int[] maskOfCell, byte[] shadow)
        {
            this.firstColumn = firstColumn;
            this.firstRow = firstRow;
            this.columns = columns;
            this.maskOfCell = maskOfCell;
            this.shadow = shadow;
        }

        public static SceneryHider For(Data.Game game, Graphics graphics, CapturedSprite sprite, (int X, int Y) corner, int width)
        {
            int ground = corner.Y + sprite.GroundY - sprite.Top;
            int firstColumn = corner.X >> TILE_WIDTH_SHIFT;
            int firstRow = corner.Y >> TILE_HEIGHT_SHIFT;
            int columns = ((corner.X + width - 1) >> TILE_WIDTH_SHIFT) - firstColumn + 1;
            int rows = ((corner.Y + Math.Max(sprite.Rows, 1) - 1) >> TILE_HEIGHT_SHIFT) - firstRow + 1;
            int[] masks = new int[Math.Max(columns, 1) * Math.Max(rows, 1)];

            Array.Fill(masks, -1);

            if (!sprite.InFront && sprite.Actor is { } actor)
            {
                for (int xm = 0; xm < columns; xm++)
                {
                    for (int ym = 0; ym < rows; ym++)
                    {
                        masks[ym * columns + xm] = MaskOfCell(game, actor, sprite.Slot, ground, firstColumn + xm, firstRow + ym, xm, ym);
                    }
                }
            }

            return new SceneryHider(firstColumn, firstRow, columns, masks, graphics.Shadow);
        }

        /// Whether the world pixel is hidden behind scenery.
        public bool Hides(int worldX, int worldY)
        {
            int column = (worldX >> TILE_WIDTH_SHIFT) - firstColumn;
            int row = (worldY >> TILE_HEIGHT_SHIFT) - firstRow;

            if (column < 0 || column >= columns || row < 0 || row * columns + column >= maskOfCell.Length || maskOfCell[row * columns + column] < 0)
            {
                return false;
            }

            int at = maskOfCell[row * columns + column] * SHADOW_MASK_BYTES + (worldY & (WorldImages.TILE_HEIGHT - 1)) * SHADOW_ROW_BYTES + ((worldX & (WorldImages.TILE_WIDTH - 1)) >> 3);

            return at < shadow.Length && ((shadow[at] >> (7 - (worldX & 7))) & 1) != 0;
        }

        // The mask number of the scenery tile at a cell, or -1 when the tile does not hide the sprite.
        private static int MaskOfCell(Data.Game game, Shape actor, int slot, int ground, int tileColumn, int tileRow, int xm, int ym)
        {
            int tile = TileAt(game, tileColumn, tileRow);

            if (tile < 0)
            {
                return -1;
            }

            int entry = tile * TERRAIN_ENTRY_BYTES;
            short type = (short)(game.Map.TerrainByte(entry + 1) & TERRAIN_TYPE_MASK);

            if (actor.State == ActorState.FALL)
            {
                if (entry <= SceneryMasks.FALL_TILE_LIMIT)
                {
                    return -1;
                }

                type = 3;
            }

            bool onBridge = game.Hero.Sector == SceneryMasks.BRIDGE_SECTOR && slot != RAFT_SLOT;
            short feetAboveTile = (short)(ground - (tileRow << TILE_HEIGHT_SHIFT));

            switch (SceneryMasks.RuleFor(type, xm, ym, feetAboveTile, onBridge))
            {
                case SceneryMasks.SceneryRule.Skip:
                    return -1;

                case SceneryMasks.SceneryRule.UseFullMask:
                    entry = SceneryMasks.TILE_FULL_MASK;

                    break;
            }

            return game.Map.TerrainByte(entry);
        }
    }
}
