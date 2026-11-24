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

    /// Every this many columns of the plain door is a dark board seam.
    private const int DARK_COLUMN_SPACING = 4;

    private const uint OUTLINE = 0xff20_1006;
    private const uint DARK_WOOD = 0xff7a_4a20;
    private const uint LIGHT_WOOD = 0xffb0_7438;
    private const uint PANEL_WOOD = 0xff95_5c2a;
    private const uint KNOB = 0xffff_dc50;
    private const int KNOB_X = WIDTH - 6;
    private const int KNOB_Y = HEIGHT / 2 + 2;

    private static readonly uint[] pixels = CursorPixels.Build(WIDTH, HEIGHT, PixelAt);

    /// The icon with its middle at an overlay position.
    public static Image At((int X, int Y) center) => new(center.X - WIDTH / 2, center.Y - HEIGHT / 2, WIDTH, HEIGHT, pixels);

    // Whether a pixel is part of the door: a rectangle with a half circle on top.
    private static bool Inside(int positionX, int positionY)
    {
        if (positionX < 0 || positionX >= WIDTH || positionY < 0 || positionY >= HEIGHT)
        {
            return false;
        }

        if (positionY >= ARCH_RADIUS)
        {
            return true;
        }

        int across = positionX - ARCH_RADIUS + 1;
        int up = ARCH_RADIUS - positionY;

        return across * across + up * up <= ARCH_RADIUS * ARCH_RADIUS;
    }

    private static uint PixelAt(int positionX, int positionY)
    {
        if (!Inside(positionX, positionY))
        {
            return 0;
        }

        bool edge = !Inside(positionX - 1, positionY) || !Inside(positionX + 1, positionY) || !Inside(positionX, positionY - 1) || !Inside(positionX, positionY + 1);

        if (edge)
        {
            return OUTLINE;
        }

        if (positionX == KNOB_X && (positionY == KNOB_Y || positionY == KNOB_Y + 1) || positionX == KNOB_X + 1 && (positionY == KNOB_Y || positionY == KNOB_Y + 1))
        {
            return KNOB;
        }

        // Two raised panels in the lower part of the door, a lighter board elsewhere.
        bool panel = positionY >= ARCH_RADIUS + PANEL_INSET - PANEL_GAP && positionY < HEIGHT - PANEL_INSET && positionX >= PANEL_INSET && positionX < WIDTH - PANEL_INSET;

        return panel ? PANEL_WOOD : positionX % DARK_COLUMN_SPACING == 0 ? DARK_WOOD : LIGHT_WOOD;  // panel wood inside; outside, every 4th column is dark
    }
}
