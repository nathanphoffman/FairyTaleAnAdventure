namespace FaeryTale.Gfx;

/// Slabs of grey stone, made up in code like the wooden boards of the menus: a speckled grey with
/// soft darker and lighter blotches, a light edge on the top and left and a dark one on the bottom
/// and right, as if the slab were cut from rock and laid on a table. They are the inventory's panel
/// and its squares, made once and drawn as ready-made pictures.
public static class Stone
{
    // The stone's grey, darkest and lightest.
    private const int BASE_GREY = 104;
    private const int BLOTCH_RANGE = 26;
    private const int SPECKLE_RANGE = 14;

    // Blotches are value noise on a grid this many pixels apart.
    private const int BLOTCH_SIZE = 9;

    // The bevelled edge: its width, and how much lighter or darker it makes the stone.
    private const int BEVEL = 2;
    private const int BEVEL_STEP = 34;

    // How much lighter a lit slab is.
    private const int LIT_LIFT = BEVEL_STEP / 2;

    private const uint OUTLINE = 0xff1c_1c20;

    /// A slab width x height pixels as 0xAARRGGBB. seed makes each slab's pattern different.
    public static uint[] Slab(int width, int height, uint seed) => Make(width, height, seed, 0);

    /// The same slab lit (the one under the pointer): lighter.
    public static uint[] LitSlab(int width, int height, uint seed) => Make(width, height, seed, LIT_LIFT);

    // The slab, every grey level raised by lift.
    private static uint[] Make(int width, int height, uint seed, int lift)
    {
        uint[] pixels = new uint[width * height];

        for (int y = 0; y < height; y++)
        {
            for (int x = 0; x < width; x++)
            {
                pixels[y * width + x] = PixelAt(x, y, (width, height), seed, lift);
            }
        }

        return pixels;
    }

    // One pixel: the outline round the slab, the bevel inside it, then the stone itself.
    private static uint PixelAt(int x, int y, (int Width, int Height) size, uint seed, int lift)
    {
        if (x == 0 || y == 0 || x == size.Width - 1 || y == size.Height - 1)
        {
            return OUTLINE;
        }

        int grey = BASE_GREY + Blotch(x, y, seed) + Speckle(x, y, seed) + lift;

        if (x <= BEVEL || y <= BEVEL)
        {
            grey += BEVEL_STEP;
        }
        else if (x >= size.Width - 1 - BEVEL || y >= size.Height - 1 - BEVEL)
        {
            grey -= BEVEL_STEP;
        }

        return Grey(grey);
    }

    // Soft patches: value noise, smoothly mixed between the corners of a grid square.
    private static int Blotch(int x, int y, uint seed)
    {
        int cellX = x / BLOTCH_SIZE;
        int cellY = y / BLOTCH_SIZE;
        double across = (x % BLOTCH_SIZE) / (double)BLOTCH_SIZE;
        double down = (y % BLOTCH_SIZE) / (double)BLOTCH_SIZE;
        double top = Mix(Noise(cellX, cellY, seed), Noise(cellX + 1, cellY, seed), across);
        double bottom = Mix(Noise(cellX, cellY + 1, seed), Noise(cellX + 1, cellY + 1, seed), across);

        return (int)((Mix(top, bottom, down) - 0.5) * BLOTCH_RANGE);
    }

    // Fine grain, a different value for every pixel.
    private static int Speckle(int x, int y, uint seed) => (int)((Noise(x * 7 + 3, y * 13 + 5, seed ^ 0x9e37u) - 0.5) * SPECKLE_RANGE);

    private static double Mix(double from, double to, double part) => from + (to - from) * part;

    // A repeatable number from 0 to 1 for a place (a small hash of the coordinates).
    private static double Noise(int x, int y, uint seed)
    {
        uint hash = unchecked((uint)(x * 374761393 + y * 668265263) ^ seed * 2246822519u);

        hash = unchecked((hash ^ (hash >> 13)) * 1274126177u);
        hash ^= hash >> 16;

        return (hash & 0xffff) / 65535.0;
    }

    // A grey of a brightness (kept in range), slightly bluish like cold stone.
    private static uint Grey(int brightness)
    {
        uint level = (uint)Math.Clamp(brightness, 0, 255);

        return 0xff00_0000 | level << 16 | level << 8 | Math.Min(level + 6, 255u);
    }
}
