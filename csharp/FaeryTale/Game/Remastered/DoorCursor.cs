using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The mouse pointer when it is over a door: a little wooden door with an arched top. It is drawn
/// once from a few rules, in the same style as the wooden boards of the menus.
internal static class DoorCursor
{
    private const int WIDTH = 22;
    private const int HEIGHT = 30;
    private const int ARCH_RADIUS = WIDTH / 2;

    /// Panels on the door: how far in from the edge they start, and the gap between them.
    private const int PANEL_INSET = 4;
    private const int PANEL_GAP = 2;

    private const uint OUTLINE = 0xff20_1006;
    private const uint DARK_WOOD = 0xff7a_4a20;
    private const uint LIGHT_WOOD = 0xffb0_7438;
    private const uint PANEL_WOOD = 0xff95_5c2a;
    private const uint KNOB = 0xffff_dc50;
    private const int KNOB_X = WIDTH - 6;
    private const int KNOB_Y = HEIGHT / 2 + 2;

    private static readonly uint[] pixels = Draw();

    /// The icon with its middle at an overlay position.
    public static Image At((int X, int Y) center) => new(center.X - WIDTH / 2, center.Y - HEIGHT / 2, WIDTH, HEIGHT, pixels);

    // Every pixel of the icon: outside the door shape nothing, along its edge the outline, inside wood.
    private static uint[] Draw()
    {
        uint[] result = new uint[WIDTH * HEIGHT];

        for (int y = 0; y < HEIGHT; y++)
        {
            for (int x = 0; x < WIDTH; x++)
            {
                result[y * WIDTH + x] = PixelAt(x, y);
            }
        }

        return result;
    }

    // Whether a pixel is part of the door: a rectangle with a half circle on top.
    private static bool Inside(int x, int y)
    {
        if (x < 0 || x >= WIDTH || y < 0 || y >= HEIGHT)
        {
            return false;
        }

        if (y >= ARCH_RADIUS)
        {
            return true;
        }

        int across = x - ARCH_RADIUS + 1;
        int up = ARCH_RADIUS - y;

        return across * across + up * up <= ARCH_RADIUS * ARCH_RADIUS;
    }

    private static uint PixelAt(int x, int y)
    {
        if (!Inside(x, y))
        {
            return 0;
        }

        bool edge = !Inside(x - 1, y) || !Inside(x + 1, y) || !Inside(x, y - 1) || !Inside(x, y + 1);

        if (edge)
        {
            return OUTLINE;
        }

        if (x == KNOB_X && (y == KNOB_Y || y == KNOB_Y + 1) || x == KNOB_X + 1 && (y == KNOB_Y || y == KNOB_Y + 1))
        {
            return KNOB;
        }

        // Two raised panels in the lower part of the door, a lighter board elsewhere.
        bool panel = y >= ARCH_RADIUS + PANEL_INSET - PANEL_GAP && y < HEIGHT - PANEL_INSET && x >= PANEL_INSET && x < WIDTH - PANEL_INSET;

        return panel ? PANEL_WOOD : x % 4 == 0 ? DARK_WOOD : LIGHT_WOOD;
    }
}
