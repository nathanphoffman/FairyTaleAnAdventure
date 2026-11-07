namespace FaeryTale.Gfx;

/// Wooden planks, made up in code: a board with grain, a lighter top and left edge, a darker
/// bottom and right edge, and a soft shadow under it, as if the board lay on a table. They are
/// made once (they take a moment) and then drawn as ready-made pictures.
public static class Wood
{
    /// The shadow reaches this far right of and below the board.
    public const int SHADOW = 7;

    // Wood colours, darkest and lightest grain.
    private static readonly (float R, float G, float B) DARK_GRAIN = (88f, 52f, 24f);
    private static readonly (float R, float G, float B) LIGHT_GRAIN = (176f, 118f, 66f);

    /// How bright a board is and how much of its blue is kept: a hovered board is lit, brighter and a
    /// little warmer.
    private readonly record struct Tone(float Brightness, float BlueShare);

    private static readonly Tone PLAIN = new(1f, 1f);
    private static readonly Tone LIT = new(1.22f, 0.92f);

    // The bevel makes the lit edges (top and left) lighter and the others darker by this much.
    private const float LIT_EDGE_STEP = 32f;
    private const float SHADED_EDGE_STEP = -46f;

    private const float MAX_CHANNEL = 255f;
    private const uint OPAQUE = 255;
    private const int ALPHA_SHIFT = 24;
    private const int RED_SHIFT = 16;
    private const int GREEN_SHIFT = 8;

    // Width of the bevelled edge, and the outline colour.
    private const int BEVEL = 2;
    private const uint OUTLINE = 0xff2a_1608;

    // How dark the shadow is at its darkest (out of 255) and how far its soft edge reaches.
    private const float SHADOW_ALPHA = 150f;
    private const float SHADOW_SOFT_EDGE = 5f;

    /// A board width x height pixels, with room for its shadow, as 0xAARRGGBB pixels (so width +
    /// SHADOW wide and height + SHADOW tall). seed makes each board's grain different.
    public static uint[] Plank(int width, int height, uint seed) => Make(width, height, seed, PLAIN);

    /// The same board lit (the one under the pointer): brighter and a little warmer.
    public static uint[] LitPlank(int width, int height, uint seed) => Make(width, height, seed, LIT);

    // The board, with grain and shadow, in a tone.
    private static uint[] Make(int width, int height, uint seed, Tone tone)
    {
        int fullWidth = width + SHADOW;
        int fullHeight = height + SHADOW;
        uint[] pixels = new uint[fullWidth * fullHeight];

        for (int y = 0; y < fullHeight; y++)
        {
            for (int x = 0; x < fullWidth; x++)
            {
                pixels[y * fullWidth + x] = x < width && y < height ? BoardPixel(x, y, width, height, seed, tone) : ShadowPixel(x, y, width, height);
            }
        }

        return pixels;
    }

    // One pixel of the board: grain, then the bevel and outline.
    private static uint BoardPixel(int x, int y, int width, int height, uint seed, Tone tone)
    {
        if (x == 0 || y == 0 || x == width - 1 || y == height - 1)
        {
            return OUTLINE;
        }

        (float r, float g, float b) = Grain(x, y, seed);

        // Light falls from the top left: those edges are lit, the others in shade.
        float bevel = y <= BEVEL || x <= BEVEL ? LIT_EDGE_STEP : y >= height - 1 - BEVEL || x >= width - 1 - BEVEL ? SHADED_EDGE_STEP : 0f;

        (r, g, b) = (r + bevel, g + bevel, b + bevel);

        return Pack(r * tone.Brightness, g * tone.Brightness, b * tone.Brightness * tone.BlueShare, OPAQUE);
    }

    // The colour of the wood at a point: streaks that run along the board and wander a little,
    // with fine fibres on top.
    private static (float R, float G, float B) Grain(int x, int y, uint seed)
    {
        float fx = x;
        float fy = y;
        float wander = Noise(fx * 0.012f, fy * 0.15f, seed) * 12f + 4f * MathF.Sin(fx * 0.021f + seed);
        float streak = 0.5f + 0.5f * MathF.Sin((fy + wander) * 0.46f + Noise(fx * 0.05f, fy * 0.4f, seed + 1) * 2.6f);
        float fibre = Noise(fx * 0.55f, fy * 0.9f, seed + 2);
        float tone = Math.Clamp(streak * 0.62f + fibre * 0.38f, 0f, 1f);

        return (
            DARK_GRAIN.R + (LIGHT_GRAIN.R - DARK_GRAIN.R) * tone,
            DARK_GRAIN.G + (LIGHT_GRAIN.G - DARK_GRAIN.G) * tone,
            DARK_GRAIN.B + (LIGHT_GRAIN.B - DARK_GRAIN.B) * tone);
    }

    // One pixel of the shadow: black, see-through, and fading out towards the edge. The shadow is
    // the board's shape moved down and right.
    private static uint ShadowPixel(int x, int y, int width, int height)
    {
        float offset = SHADOW / 2;
        float fx = x;
        float fy = y;

        // How far inside the moved board's edges the point is.
        float inside = MathF.Min(MathF.Min(fx - offset, width + offset - fx), MathF.Min(fy - offset, height + offset - fy));

        if (inside <= 0f)
        {
            return 0;
        }

        return (uint)(SHADOW_ALPHA * MathF.Min(inside / SHADOW_SOFT_EDGE, 1f)) << ALPHA_SHIFT;
    }

    // Smooth random values between 0 and 1: each whole-number point has a fixed random value and
    // the points between are blended.
    private static float Noise(float x, float y, uint seed)
    {
        float ix = MathF.Floor(x);
        float iy = MathF.Floor(y);
        float tx = Smooth(x - ix);
        float ty = Smooth(y - iy);

        float Corner(float dx, float dy) => RandomAt((int)(ix + dx), (int)(iy + dy), seed);

        float top = Corner(0, 0) + (Corner(1, 0) - Corner(0, 0)) * tx;
        float bottom = Corner(0, 1) + (Corner(1, 1) - Corner(0, 1)) * tx;

        return top + (bottom - top) * ty;
    }

    // Eases 0 to 1 so the blend has no hard corners.
    private static float Smooth(float t) => t * t * (3f - 2f * t);

    // A fixed pseudo-random value between 0 and 1 for a point.
    private static float RandomAt(int x, int y, uint seed)
    {
        // A cheap hash: multiplying by big odd numbers scrambles x, y and the seed, xor joins them, and
        // the two shift-xor steps below spread the high bits down so nearby spots look unrelated.
        uint hash = unchecked((uint)x * 0x9e37_79b1 ^ (uint)y * 0x85eb_ca6b ^ seed * 0xc2b2_ae35);

        hash ^= hash >> 15;
        hash = unchecked(hash * 0x2c1b_3c6d);
        hash ^= hash >> 12;

        // Keeps the low 16 bits and scales them to a number from 0 to 1.
        return (hash & 0xffff) / 65535f;
    }

    // Red, green and blue (0 to 255, clamped) and an alpha as 0xAARRGGBB.
    private static uint Pack(float red, float green, float blue, uint alpha)
    {
        static uint Channel(float value) => (uint)Math.Clamp(value, 0f, MAX_CHANNEL);

        return alpha << ALPHA_SHIFT | Channel(red) << RED_SHIFT | Channel(green) << GREEN_SHIFT | Channel(blue);
    }
}
