using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The red crosshair that marks an enemy that can be attacked (as the pointer) and the enemy being
/// attacked (over it): a ring with four lines crossing it, with a dark edge so it shows on any
/// ground. Made once for each size.
internal static class Crosshair
{
    private const uint RED = 0xffe0_2020;
    private const uint EDGE = 0xff20_0808;

    // The ring is this part of the radius thick (at least 2 pixels); the lines reach from this part of
    // the radius inside the ring to this part outside it.
    private const int RING_THICKNESS_DIVISOR = 6;
    private const double LINE_INNER = 0.4;
    private const double LINE_OUTER = 1.35;

    private static readonly Dictionary<int, uint[]> made = [];

    /// The crosshair with its middle at a position, as big across as twice the radius (and a little
    /// more for the lines and the edge).
    public static Image At((int X, int Y) center, int radius)
    {
        int reach = (int)Math.Ceiling(radius * LINE_OUTER) + 2;
        int size = reach * 2 + 1;

        if (!made.TryGetValue(radius, out uint[]? pixels))
        {
            pixels = Draw(radius, reach);
            made[radius] = pixels;
        }

        return new Image(center.X - reach, center.Y - reach, size, size, pixels);
    }

    // Every pixel: red on the ring and the lines, a dark edge next to it, else see-through.
    private static uint[] Draw(int radius, int reach)
    {
        int size = reach * 2 + 1;
        bool[] red = new bool[size * size];
        uint[] pixels = new uint[size * size];

        for (int positionY = 0; positionY < size; positionY++)
        {
            for (int positionX = 0; positionX < size; positionX++)
            {
                red[positionY * size + positionX] = IsRed(positionX - reach, positionY - reach, radius);
            }
        }

        for (int positionY = 0; positionY < size; positionY++)
        {
            for (int positionX = 0; positionX < size; positionX++)
            {
                pixels[positionY * size + positionX] = red[positionY * size + positionX] ? RED : TouchesRed(red, size, positionX, positionY) ? EDGE : 0; // red pixel, else edge if next to red, else transparent
            }
        }

        return pixels;
    }

    // Whether a pixel (offset from the middle) is on the ring or on one of the lines.
    private static bool IsRed(int deltaX, int deltaY, int radius)
    {
        int half = Math.Max(2, radius / RING_THICKNESS_DIVISOR) / 2;
        double distance = Math.Sqrt(deltaX * deltaX + deltaY * deltaY);
        bool onRing = Math.Abs(distance - radius) <= half;
        bool onLine = (Math.Abs(deltaX) <= half || Math.Abs(deltaY) <= half) && distance >= radius * LINE_INNER && distance <= radius * LINE_OUTER;

        return onRing || onLine;
    }

    // Whether a pixel has a red one above, below or beside it.
    private static bool TouchesRed(bool[] red, int size, int positionX, int positionY) =>
        positionX > 0 && red[positionY * size + positionX - 1] || positionX < size - 1 && red[positionY * size + positionX + 1]
        || positionY > 0 && red[(positionY - 1) * size + positionX] || positionY < size - 1 && red[(positionY + 1) * size + positionX];
}
