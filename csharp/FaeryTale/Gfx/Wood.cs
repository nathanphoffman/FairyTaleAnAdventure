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

    // Grain shape: how fast and how far the streaks wander along the board.
    private const float WANDER_NOISE_X_SCALE = 0.012f;
    private const float WANDER_NOISE_Y_SCALE = 0.15f;
    private const float WANDER_NOISE_STRENGTH = 12f;
    private const float WANDER_WAVE_STRENGTH = 4f;
    private const float WANDER_WAVE_SCALE = 0.021f;

    // Grain shape: the streaks themselves (sine stripes bent by noise).
    private const float STREAK_STRIPE_SCALE = 0.46f;
    private const float STREAK_NOISE_X_SCALE = 0.05f;
    private const float STREAK_NOISE_Y_SCALE = 0.4f;
    private const float STREAK_NOISE_BEND = 2.6f;
    private const float STREAK_HALF = 0.5f;  // turns the sine's -1 to 1 into 0 to 1

    // Grain shape: the fine fibres, and how the streaks and fibres are mixed (shares add up to 1).
    private const float FIBRE_NOISE_X_SCALE = 0.55f;
    private const float FIBRE_NOISE_Y_SCALE = 0.9f;
    private const float STREAK_SHARE = 0.62f;
    private const float FIBRE_SHARE = 0.38f;

    // Multipliers that scramble x, y and the seed in the hash, and the one that follows the first shift.
    private const uint HASH_X_MULTIPLIER = 0x9e37_79b1;
    private const uint HASH_Y_MULTIPLIER = 0x85eb_ca6b;
    private const uint HASH_SEED_MULTIPLIER = 0xc2b2_ae35;
    private const uint HASH_MIX_MULTIPLIER = 0x2c1b_3c6d;
    private const int HASH_FIRST_SHIFT = 15;
    private const int HASH_SECOND_SHIFT = 12;

    // Hash values keep their low 16 bits, so the largest is this.
    private const uint HASH_LOW_MASK = 0xffff;
    private const float HASH_LOW_MAX = 65535f;

    // Smoothstep curve: 3t^2 - 2t^3.
    private const float SMOOTHSTEP_LINEAR_TERM = 3f;
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

        for (int positionY = 0; positionY < fullHeight; positionY++)
        {
            for (int positionX = 0; positionX < fullWidth; positionX++)
            {
                pixels[positionY * fullWidth + positionX] = positionX < width && positionY < height ? BoardPixel(positionX, positionY, width, height, seed, tone) : ShadowPixel(positionX, positionY, width, height);
            }
        }

        return pixels;
    }

    // One pixel of the board: grain, then the bevel and outline.
    private static uint BoardPixel(int positionX, int positionY, int width, int height, uint seed, Tone tone)
    {
        if (positionX == 0 || positionY == 0 || positionX == width - 1 || positionY == height - 1)
        {
            return OUTLINE;
        }

        (float r, float g, float b) = Grain(positionX, positionY, seed);

        // Light falls from the top left: those edges are lit, the others in shade.
        float bevel = positionY <= BEVEL || positionX <= BEVEL ? LIT_EDGE_STEP : positionY >= height - 1 - BEVEL || positionX >= width - 1 - BEVEL ? SHADED_EDGE_STEP : 0f;

        (r, g, b) = (r + bevel, g + bevel, b + bevel);

        return Pack(r * tone.Brightness, g * tone.Brightness, b * tone.Brightness * tone.BlueShare, OPAQUE);
    }

    // The colour of the wood at a point: streaks that run along the board and wander a little,
    // with fine fibres on top.
    private static (float R, float G, float B) Grain(int positionX, int positionY, uint seed)
    {
        float fx = positionX;
        float fy = positionY;
        float wander = Noise(fx * WANDER_NOISE_X_SCALE, fy * WANDER_NOISE_Y_SCALE, seed) * WANDER_NOISE_STRENGTH + WANDER_WAVE_STRENGTH * MathF.Sin(fx * WANDER_WAVE_SCALE + seed);
        float streak = STREAK_HALF + STREAK_HALF * MathF.Sin((fy + wander) * STREAK_STRIPE_SCALE + Noise(fx * STREAK_NOISE_X_SCALE, fy * STREAK_NOISE_Y_SCALE, seed + 1) * STREAK_NOISE_BEND);
        float fibre = Noise(fx * FIBRE_NOISE_X_SCALE, fy * FIBRE_NOISE_Y_SCALE, seed + 2);
        float tone = Math.Clamp(streak * STREAK_SHARE + fibre * FIBRE_SHARE, 0f, 1f);

        return (
            DARK_GRAIN.R + (LIGHT_GRAIN.R - DARK_GRAIN.R) * tone,
            DARK_GRAIN.G + (LIGHT_GRAIN.G - DARK_GRAIN.G) * tone,
            DARK_GRAIN.B + (LIGHT_GRAIN.B - DARK_GRAIN.B) * tone);
    }

    // One pixel of the shadow: black, see-through, and fading out towards the edge. The shadow is
    // the board's shape moved down and right.
    private static uint ShadowPixel(int positionX, int positionY, int width, int height)
    {
        float offset = SHADOW / 2;
        float fx = positionX;
        float fy = positionY;

        // How far inside the moved board's edges the point is.
        float inside = MathF.Min(MathF.Min(fx - offset, width + offset - fx), MathF.Min(fy - offset, height + offset - fy));

        if (inside <= 0f)
        {
            return 0;
        }

        return (uint)(SHADOW_ALPHA * MathF.Min(inside / SHADOW_SOFT_EDGE, 1f)) << ALPHA_SHIFT;  // fade the shadow in as a 0-255 alpha, move to top byte
    }

    // Smooth random values between 0 and 1: each whole-number point has a fixed random value and
    // the points between are blended.
    private static float Noise(float positionX, float positionY, uint seed)
    {
        float ix = MathF.Floor(positionX);
        float iy = MathF.Floor(positionY);
        float tx = Smooth(positionX - ix);
        float ty = Smooth(positionY - iy);

        float Corner(float deltaX, float deltaY) => RandomAt((int)(ix + deltaX), (int)(iy + deltaY), seed);

        float topEdge = Corner(0, 0) + (Corner(1, 0) - Corner(0, 0)) * tx;
        float bottom = Corner(0, 1) + (Corner(1, 1) - Corner(0, 1)) * tx;

        return topEdge + (bottom - topEdge) * ty;
    }

    // Eases 0 to 1 so the blend has no hard corners.
    private static float Smooth(float t) => t * t * (SMOOTHSTEP_LINEAR_TERM - 2f * t);

    // A fixed pseudo-random value between 0 and 1 for a point.
    private static float RandomAt(int positionX, int positionY, uint seed)
    {
        // A cheap hash: multiplying by big odd numbers scrambles x, y and the seed, xor joins them, and
        // the two shift-xor steps below spread the high bits down so nearby spots look unrelated.
        uint hash = unchecked((uint)positionX * HASH_X_MULTIPLIER ^ (uint)positionY * HASH_Y_MULTIPLIER ^ seed * HASH_SEED_MULTIPLIER);

        hash ^= hash >> HASH_FIRST_SHIFT;  // xor-shift mixes high bits into low bits for a noisy hash
        hash = unchecked(hash * HASH_MIX_MULTIPLIER);
        hash ^= hash >> HASH_SECOND_SHIFT;  // mix the high bits down again to scatter the result

        // Keeps the low 16 bits and scales them to a number from 0 to 1.
        return (hash & HASH_LOW_MASK) / HASH_LOW_MAX;
    }

    // Red, green and blue (0 to 255, clamped) and an alpha as 0xAARRGGBB.
    private static uint Pack(float red, float green, float blue, uint alpha)
    {
        static uint Channel(float value) => (uint)Math.Clamp(value, 0f, MAX_CHANNEL);

        return alpha << ALPHA_SHIFT | Channel(red) << RED_SHIFT | Channel(green) << GREEN_SHIFT | Channel(blue);  // pack alpha and clamped channels into one ARGB value
    }
}
