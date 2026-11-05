using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Render;

/// Drawing the map into the play field: single tiles, whole screens, new edges and scrolling.
public static class MapTiles
{
    /// One bitplane of 256 tiles, 64 bytes per tile.
    public const int IMAGE_PLANE_SIZE = 16384;
    public const int TILE_BYTES = 64;
    public const int TILE_LINES = 32;

    /// Bytes per scan line of the play field bitmaps.
    public const int ROW_BYTES = 40;
    public const int PLANE_COUNT = 5;

    /// There are 8 scroll directions.
    private const int DIRECTION_MASK = 7;

    /// One way to scroll: source x (bytes), source line, destination x, destination line, width in
    /// bytes and number of lines.
    internal readonly record struct ScrollMode(int SrcX, int SrcLine, int DstX, int DstLine, int Width, int Lines);

    /// Directions are numbered by how the picture moves:
    /// 0 right, 1 down+right, 2 down, 3 down+left, 4 left, 5 up+left, 6 up, 7 up+right.
    /// The new edge is filled in by DrawMapColumn / DrawMapRow afterwards.
    internal static readonly ScrollMode[] SCROLL_MODES =
    [
        new(0, 0, 2, 0, 36, 192),
        new(0, 0, 2, 32, 36, 160),
        new(0, 0, 0, 32, 38, 160),
        new(2, 0, 0, 32, 36, 160),
        new(2, 0, 0, 0, 36, 192),
        new(2, 32, 0, 0, 36, 160),
        new(0, 32, 0, 0, 38, 160),
        new(0, 32, 2, 0, 36, 160),
    ];

    /// Draws one tile at byte offset destination in each of the five planes (2 bytes wide, 32 lines).
    public static void DrawTile(Graphics graphics, Screen screen, int tile, int destination)
    {
        for (int plane = 0; plane < PLANE_COUNT; plane++)
        {
            int source = plane * IMAGE_PLANE_SIZE + tile * TILE_BYTES;
            Span<byte> bytes = screen.DrawBitmap.Plane(plane);
            int at = destination;

            for (int line = 0; line < TILE_LINES; line++)
            {
                bytes[at] = graphics.Image[source];
                bytes[at + 1] = graphics.Image[source + 1];
                source += 2;
                at += ROW_BYTES;
            }
        }
    }

    /// Draws the tile at minimap position (col, row) to its place on screen.
    public static void DrawMinimapTile(Data.Game game, Video video, int column, int row) =>
        DrawTile(video.Graphics, video.Screen, game.Map.Minimap[column * MapLookup.SCREEN_ROWS + row], row * TILE_LINES * ROW_BYTES + column * 2);

    /// The whole screen, 19 x 6 tiles.
    public static void DrawMap(Data.Game game, Video video)
    {
        for (int column = 0; column < MapLookup.SCREEN_COLUMNS; column++)
        {
            for (int row = 0; row < MapLookup.SCREEN_ROWS; row++)
            {
                DrawMinimapTile(game, video, column, row);
            }
        }
    }

    /// One column of tiles; x is the byte offset on screen (0 or 36).
    public static void DrawMapColumn(Data.Game game, Video video, int x)
    {
        for (int row = 0; row < MapLookup.SCREEN_ROWS; row++)
        {
            DrawMinimapTile(game, video, x / 2, row);
        }
    }

    /// One row of tiles; r is twice the row number (0 or 10).
    public static void DrawMapRow(Data.Game game, Video video, int r)
    {
        for (int column = 0; column < MapLookup.SCREEN_COLUMNS; column++)
        {
            DrawMinimapTile(game, video, column, r / 2);
        }
    }

    /// Shifts the map one tile in one of 8 directions (the blitter did this in the original).
    /// Moving down copies from the bottom up, so lines are not overwritten before they are read.
    public static void ScrollMap(Screen screen, int direction)
    {
        ScrollMode mode = SCROLL_MODES[direction & DIRECTION_MASK];
        Gfx.Bitmap bitmap = screen.DrawBitmap;

        for (int plane = 0; plane < PLANE_COUNT; plane++)
        {
            Span<byte> bytes = bitmap.Plane(plane);

            for (int index = 0; index < mode.Lines; index++)
            {
                // Copy bottom-up when moving down so rows are not overwritten before being read.
                int line = mode.DstLine > mode.SrcLine ? mode.Lines - 1 - index : index;
                int from = (mode.SrcLine + line) * ROW_BYTES + mode.SrcX;
                int to = (mode.DstLine + line) * ROW_BYTES + mode.DstX;

                bytes.Slice(from, mode.Width).CopyTo(bytes[to..]);
            }
        }
    }
}
