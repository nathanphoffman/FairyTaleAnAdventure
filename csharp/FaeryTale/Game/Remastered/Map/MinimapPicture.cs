using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The small map in the corner of the world view: the land around the hero from the world
/// overview, with a red mark where the hero is. Drawn in units (see BubbleLayout); each map pixel is
/// one unit wide and two tall, as a tile is twice as high as it is wide.
public static class MinimapPicture
{
    /// How much land it shows, in tiles.
    public const int COLUMNS = 112;
    public const int ROWS = 40;

    private const int ROW_HEIGHT = 2;
    private const int FRAME = 2;

    /// Units across and down the whole picture with its frame.
    public const int WIDTH = COLUMNS + 2 * FRAME;
    public const int HEIGHT = ROWS * ROW_HEIGHT + 2 * FRAME;

    // The hero's mark: a red square with a white ring.
    private const int MARK_RADIUS = 3;
    private const uint MARK_RING = 0xffff_ffff;
    private const uint MARK_FILL = 0xffe0_1010;

    private const uint OUTER_FRAME = 0xff20_1006;
    private const uint INNER_FRAME = 0xffd8_b070;

    /// The picture for a hero standing on a tile of the world overview.
    public static Image Of(uint[] world, (int Column, int Row) hero)
    {
        int left = Math.Clamp(hero.Column - COLUMNS / 2, 0, WorldOverview.COLUMNS - COLUMNS);
        int top = Math.Clamp(hero.Row - ROWS / 2, 0, WorldOverview.ROWS - ROWS);
        uint[] pixels = new uint[WIDTH * HEIGHT];

        for (int y = 0; y < HEIGHT; y++)
        {
            for (int x = 0; x < WIDTH; x++)
            {
                pixels[y * WIDTH + x] = PixelAt(world, (left, top), x, y);
            }
        }

        DrawMark(pixels, (FRAME + hero.Column - left, FRAME + (hero.Row - top) * ROW_HEIGHT + 1));

        return new Image(0, 0, WIDTH, HEIGHT, pixels);
    }

    // One pixel: the two frame lines round the edge, the land inside.
    private static uint PixelAt(uint[] world, (int Column, int Row) corner, int x, int y)
    {
        int depth = Math.Min(Math.Min(x, WIDTH - 1 - x), Math.Min(y, HEIGHT - 1 - y));

        if (depth < FRAME)
        {
            return depth == 0 ? OUTER_FRAME : INNER_FRAME;
        }

        int column = corner.Column + x - FRAME;
        int row = corner.Row + (y - FRAME) / ROW_HEIGHT;

        return world[row * WorldOverview.COLUMNS + column];
    }

    // The mark centred on a pixel of the picture.
    private static void DrawMark(uint[] pixels, (int X, int Y) centre)
    {
        for (int dy = -MARK_RADIUS; dy <= MARK_RADIUS; dy++)
        {
            for (int dx = -MARK_RADIUS; dx <= MARK_RADIUS; dx++)
            {
                int x = centre.X + dx;
                int y = centre.Y + dy;

                if (x >= 0 && x < WIDTH && y >= 0 && y < HEIGHT)
                {
                    pixels[y * WIDTH + x] = Math.Max(Math.Abs(dx), Math.Abs(dy)) == MARK_RADIUS ? MARK_RING : MARK_FILL;
                }
            }
        }
    }
}
