using FaeryTale.Game.Land;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Startup;

/// The picture behind the launch menu: a long strip of the real game map (grass, hills and trees
/// near the start) that is built once and then panned slowly from side to side.
internal sealed class MenuBackground
{
    // Where the strip starts: the top left corner of the map, in pixels. The camera pans east along
    // it, over meadows with trees and a rocky hill, and back again. It is in the region the first
    // brother starts in, so the region is already loaded when the game begins.
    private const ushort START_X = 18200;
    private const ushort START_Y = 12400;
    private const ushort REGION = 3;

    // A tile is 16 pixels wide and 32 high, and 2 bytes wide in a plane.
    private const int TILE_WIDTH = 16;
    private const int TILE_WIDTH_BYTES = 2;
    private const int TILE_WIDTH_SHIFT = 4;
    private const int TILE_HEIGHT_SHIFT = 5;
    private const ushort TILE_HEIGHT_MASK = 31;

    // The strip is this many screens of tiles wide and this many tile rows high (a screen is 200
    // lines, so 7 rows would leave no room for the start offset).
    private const int STRIP_SCREENS = 20;
    private const int STRIP_ROWS = 8;

    /// The part of the strip that is shown is the size of the game screen.
    private const int VIEW_WIDTH = 320;

    /// The camera moves one pixel every this many ticks (of 1/50 second).
    private const int TICKS_PER_PIXEL = 2;

    private readonly Bitmap strip;
    private readonly int topLine;

    private MenuBackground(Bitmap strip, int topLine)
    {
        this.strip = strip;
        this.topLine = topLine;
    }

    /// Loads the region and builds the strip. The page colours become the colours of the map.
    public static MenuBackground Load(Data.Game game, Video video)
    {
        game.World.MapX = START_X;
        game.World.MapY = START_Y;
        game.World.TileX = START_X >> TILE_WIDTH_SHIFT;
        game.World.TileY = START_Y >> TILE_HEIGHT_SHIFT;
        game.World.NewRegion = REGION;
        Regions.LoadRegionNow(game, video);
        Regions.RefreshMinimap(game, video);

        Bitmap strip = BuildStrip(game, video.Graphics);

        video.LoadPageColors(game.PageColors);

        return new MenuBackground(strip, START_Y & TILE_HEIGHT_MASK);
    }

    /// Draws the part of the strip the camera sees after tick ticks onto the page that is on screen.
    public void Draw(Video video, uint tick)
    {
        // The bitmap that is on screen: after the intro it need not be the one drawn on.
        Screen screen = video.Screen;
        BitmapId page = screen.VpPage.Ras is { } ras ? screen.Ras[ras].Bitmap : screen.DrawingBitmapId;

        BitmapOps.CopyWindow(strip, screen.GetBitmap(page), (PanPosition(tick, strip.Width), topLine));
    }

    /// Where the left edge of the view is after tick ticks: it goes right to the end of the strip
    /// and back again, over and over.
    internal static int PanPosition(uint tick, int stripWidth)
    {
        int travel = Math.Max(stripWidth - VIEW_WIDTH, 0);

        if (travel == 0)
        {
            return 0;
        }

        int along = (int)(tick / TICKS_PER_PIXEL % (uint)(travel * 2));

        return along <= travel ? along : travel * 2 - along;
    }

    // Draws the tiles of the map from the start onwards into a bitmap, one screen of columns at a
    // time (the game's own map lookup gives 19 columns of 6 tiles).
    private static Bitmap BuildStrip(Data.Game game, Graphics graphics)
    {
        int columns = MapLookup.SCREEN_COLUMNS;
        int rows = MapLookup.SCREEN_ROWS;
        Bitmap strip = new(STRIP_SCREENS * columns * TILE_WIDTH, STRIP_ROWS * MapTiles.TILE_LINES, MapTiles.PLANE_COUNT);
        int firstColumn = START_X >> TILE_WIDTH_SHIFT;
        int firstRow = START_Y >> TILE_HEIGHT_SHIFT;

        for (int screen = 0; screen < STRIP_SCREENS; screen++)
        {
            for (int band = 0; band < (STRIP_ROWS + rows - 1) / rows; band++)
            {
                MapLookup.FillMinimap(game, firstColumn + screen * columns, firstRow + band * rows);

                for (int column = 0; column < columns; column++)
                {
                    for (int row = 0; row < Math.Min(rows, STRIP_ROWS - band * rows); row++)
                    {
                        DrawTile(graphics, strip, game.Map.Minimap[column * rows + row], (screen * columns + column, band * rows + row));
                    }
                }
            }
        }

        return strip;
    }

    // Copies one tile image into the strip at tile column and row at.
    private static void DrawTile(Graphics graphics, Bitmap strip, int tile, (int Column, int Row) at)
    {
        int rowBytes = strip.BytesPerRow;

        for (int plane = 0; plane < MapTiles.PLANE_COUNT; plane++)
        {
            int imageStart = plane * MapTiles.IMAGE_PLANE_SIZE + tile * MapTiles.TILE_BYTES;
            Span<byte> bytes = strip.Plane(plane);
            int start = at.Row * MapTiles.TILE_LINES * rowBytes + at.Column * TILE_WIDTH_BYTES;

            for (int line = 0; line < MapTiles.TILE_LINES; line++)
            {
                graphics.Image.AsSpan(imageStart + line * TILE_WIDTH_BYTES, TILE_WIDTH_BYTES).CopyTo(bytes[(start + line * rowBytes)..]);
            }
        }
    }
}
