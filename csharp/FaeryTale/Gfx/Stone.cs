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

    // Value noise is centred on its midpoint of 0..1 so blotches and speckle go both lighter and darker.
    private const double NOISE_MIDPOINT = 0.5;

    // Speckle spreads its pixel coordinates with these multipliers and offsets so rows and columns differ.
    private const int SPECKLE_X_MULTIPLIER = 7;
    private const int SPECKLE_X_OFFSET = 3;
    private const int SPECKLE_Y_MULTIPLIER = 13;
    private const int SPECKLE_Y_OFFSET = 5;

    // Large primes used by the coordinate hash to scramble x, y and the seed.
    private const int HASH_X_PRIME = 374761393;
    private const int HASH_Y_PRIME = 668265263;
    private const uint HASH_SEED_PRIME = 2246822519u;
    private const uint HASH_SPREAD_PRIME = 1274126177u;

    // Hash bit folding: the first shift and the final shift (half of 32 bits).
    private const int HASH_FIRST_SHIFT = 13;
    private const int HASH_FINAL_SHIFT = 16;

    // The low 16 bits of the hash become the 0 to 1 noise value.
    private const uint HASH_LOW_MASK = 0xffff;
    private const double HASH_LOW_MAX = 65535.0;

    // Colour channel limits and the ARGB packing of a grey.
    private const int CHANNEL_MAX = 255;
    private const uint OPAQUE_ALPHA = 0xff00_0000;
    private const int RED_SHIFT = 16;
    private const int GREEN_SHIFT = 8;
    private const uint BLUISH_EXTRA = 6;

    /// A slab width x height pixels as 0xAARRGGBB. seed makes each slab's pattern different.
    public static uint[] Slab(int width, int height, uint seed) => Make(width, height, seed, 0);

    /// The same slab lit (the one under the pointer): lighter.
    public static uint[] LitSlab(int width, int height, uint seed) => Make(width, height, seed, LIT_LIFT);

    // The slab, every grey level raised by lift.
    private static uint[] Make(int width, int height, uint seed, int lift)
    {
        uint[] pixels = new uint[width * height];

        for (int positionY = 0; positionY < height; positionY++)
        {
            for (int positionX = 0; positionX < width; positionX++)
            {
                pixels[positionY * width + positionX] = PixelAt(positionX, positionY, (width, height), seed, lift);
            }
        }

        return pixels;
    }

    // One pixel: the outline round the slab, the bevel inside it, then the stone itself.
    private static uint PixelAt(int positionX, int positionY, (int Width, int Height) size, uint seed, int lift)
    {
        if (positionX == 0 || positionY == 0 || positionX == size.Width - 1 || positionY == size.Height - 1)
        {
            return OUTLINE;
        }

        int grey = BASE_GREY + Blotch(positionX, positionY, seed) + Speckle(positionX, positionY, seed) + lift;

        if (positionX <= BEVEL || positionY <= BEVEL)
        {
            grey += BEVEL_STEP;
        }
        else if (positionX >= size.Width - 1 - BEVEL || positionY >= size.Height - 1 - BEVEL)
        {
            grey -= BEVEL_STEP;
        }

        return Grey(grey);
    }

    // Soft patches: value noise, smoothly mixed between the corners of a grid square.
    private static int Blotch(int positionX, int positionY, uint seed)
    {
        int cellX = positionX / BLOTCH_SIZE;
        int cellY = positionY / BLOTCH_SIZE;
        double across = (positionX % BLOTCH_SIZE) / (double)BLOTCH_SIZE;
        double down = (positionY % BLOTCH_SIZE) / (double)BLOTCH_SIZE;
        double topEdge = Mix(Noise(cellX, cellY, seed), Noise(cellX + 1, cellY, seed), across);
        double bottom = Mix(Noise(cellX, cellY + 1, seed), Noise(cellX + 1, cellY + 1, seed), across);

        return (int)((Mix(topEdge, bottom, down) - NOISE_MIDPOINT) * BLOTCH_RANGE);
    }

    // Fine grain, a different value for every pixel.
    private static int Speckle(int positionX, int positionY, uint seed) => (int)((Noise(positionX * SPECKLE_X_MULTIPLIER + SPECKLE_X_OFFSET, positionY * SPECKLE_Y_MULTIPLIER + SPECKLE_Y_OFFSET, seed ^ 0x9e37u) - NOISE_MIDPOINT) * SPECKLE_RANGE);

    private static double Mix(double from, double to, double part) => from + (to - from) * part;

    // A repeatable number from 0 to 1 for a place (a small hash of the coordinates).
    private static double Noise(int positionX, int positionY, uint seed)
    {
        uint hash = unchecked((uint)(positionX * HASH_X_PRIME + positionY * HASH_Y_PRIME) ^ seed * HASH_SEED_PRIME);  // multiply by large primes to scramble x, y and seed, XOR mixes them

        hash = unchecked((hash ^ (hash >> HASH_FIRST_SHIFT)) * HASH_SPREAD_PRIME);  // fold high bits down with shift+XOR, then multiply to spread bits
        hash ^= hash >> HASH_FINAL_SHIFT;  // fold the top 16 bits into the bottom for final mixing

        return (hash & HASH_LOW_MASK) / HASH_LOW_MAX;
    }

    // A grey of a brightness (kept in range), slightly bluish like cold stone.
    private static uint Grey(int brightness)
    {
        uint level = (uint)Math.Clamp(brightness, 0, CHANNEL_MAX);

        return OPAQUE_ALPHA | level << RED_SHIFT | level << GREEN_SHIFT | Math.Min(level + BLUISH_EXTRA, (uint)CHANNEL_MAX);  // pack ARGB: alpha 0xff, level in red and green, blue a bit brighter
    }
}
