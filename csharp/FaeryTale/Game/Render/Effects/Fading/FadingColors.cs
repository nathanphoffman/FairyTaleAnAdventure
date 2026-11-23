using FaeryTale.Game.Data;

namespace FaeryTale.Game.Render;

/// Fading: the light level of one color.
public static partial class Fading
{
    // Colour 31 depends on where the hero is: the swamp, the secret room (open or shut), or neither.
    private static ushort Color31Here(World world) => world.Region switch
    {
        REGION_SWAMP => SWAMP_COLOR,
        REGION_SECRET when world.SecretTimer != 0 => SECRET_OPEN_COLOR,
        REGION_SECRET => SECRET_SHUT_COLOR,
        _ => DEFAULT_COLOR_31,
    };

    // Keeps the levels between 0 and 100 percent, or at night between the night limits, and works
    // out how much blue is mixed in from green (only at night).
    private static ((int R, int G, int B) Levels, int BlueFromGreen) ClampLevels((int R, int G, int B) levels, bool night)
    {
        int r = Math.Min(levels.R, FULL_LEVEL);
        int g = Math.Min(levels.G, FULL_LEVEL);
        int b = Math.Min(levels.B, FULL_LEVEL);

        if (night)
        {
            r = Math.Max(r, NIGHT_MIN_RED);
            g = Math.Max(g, NIGHT_MIN_GREEN);
            b = Math.Max(b, NIGHT_MIN_BLUE);

            return ((r, g, b), (FULL_LEVEL - g) / NIGHT_BLUE_SHARE);
        }

        return ((Math.Max(r, 0), Math.Max(g, 0), Math.Max(b, 0)), 0);
    }

    // Colour index of the palette (4 bits per channel) scaled to the light levels (percent). At
    // night blue is lifted by blueFromGreen times green, the lake colours (16-24) get a moonlight
    // tint, and blue is capped. With a magic light (lit) red is brought up to the green.
    internal static ushort FadedColor(int index, ushort color, (int R, int G, int B) levels, int blueFromGreen, bool night, bool lit)
    {
        // Colours are 12 bit 0RGB. Red is moved down a nibble so all three work on the same scale.
        int red = (color & RED_MASK) >> NIBBLE_BITS;
        int green = color & GREEN_MASK;
        int blue = color & BLUE_MASK;

        if (lit && red < green)
        {
            red = green;
        }

        red = levels.R * red / SCALE_DIVISOR;
        green = levels.G * green / SCALE_DIVISOR;
        blue = (levels.B * blue + blueFromGreen * green) / PERCENT;

        if (night)
        {
            blue += MoonlightBoost(index, levels.G);
            blue = Math.Min(blue, MAX_COMPONENT);
        }

        // Back to 0RGB.
        return (ushort)((red << (2 * NIBBLE_BITS)) + (green << NIBBLE_BITS) + blue);
    }

    private static int MoonlightBoost(int index, int greenLevel)
    {
        if (index < FIRST_LAKE_COLOR || index > LAST_LAKE_COLOR || greenLevel <= MOONLIGHT_MIN_GREEN)
        {
            return 0;
        }

        if (greenLevel < MOONLIGHT_STRONG_BELOW)
        {
            return MOONLIGHT_STRONG_BOOST;
        }

        return greenLevel < MOONLIGHT_WEAK_BELOW ? MOONLIGHT_WEAK_BOOST : 0;
    }
}
