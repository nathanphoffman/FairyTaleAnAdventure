using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Render;

/// Fading the page colours: with the time of day, to and from black, and the teleport flicker.
///
/// Colours: the screen has 32 colours, each 12 bits (4 bits red, green, blue). The game fades the
/// whole picture by scaling those components (FadePage) and loads them as the page palette. Night
/// is a darker, bluer scaling; indoors it is always full brightness.
public static class Fading
{
    public const int COLOR_COUNT = 32;

    // Colour 31 of the page changes with where the hero is.
    private const int COLOR_31 = 31;
    private const ushort REGION_SWAMP = 4;
    private const ushort REGION_SECRET = 9;
    private const ushort SWAMP_COLOR = 0x0980;
    private const ushort SECRET_OPEN_COLOR = 0x00f0;
    private const ushort SECRET_SHUT_COLOR = 0x0445;
    private const ushort DEFAULT_COLOR_31 = 0x0bdf;

    // Fade limits: nothing goes below these at night.
    private const int NIGHT_MIN_RED = 10;
    private const int NIGHT_MIN_GREEN = 25;
    private const int NIGHT_MIN_BLUE = 60;
    private const int FULL_LEVEL = 100;
    private const int MAX_COMPONENT = 15;

    // Night blue lift: one third of the green that is missing from full light.
    private const int NIGHT_BLUE_SHARE = 3;

    // A component in its nibble position (up to 0xf0) times a level in percent: dividing by 1600
    // gives back a 4 bit component.
    private const int SCALE_DIVISOR = 1600;
    private const int PERCENT = 100;

    // The lake colours get a moonlight tint at night while green stays above these levels.
    private const int FIRST_LAKE_COLOR = 16;
    private const int LAST_LAKE_COLOR = 24;
    private const int MOONLIGHT_MIN_GREEN = 20;
    private const int MOONLIGHT_STRONG_BELOW = 50;
    private const int MOONLIGHT_WEAK_BELOW = 75;
    private const int MOONLIGHT_STRONG_BOOST = 2;
    private const int MOONLIGHT_WEAK_BOOST = 1;
    private const int RED_MASK = 0x0f00;
    private const int GREEN_MASK = 0x00f0;
    private const int BLUE_MASK = 0x000f;
    private const int NIBBLE_BITS = 4;
    private const int RANDOM_COLOR_MASK = 0xfff;

    /// Fading goes in steps of 5 percent, one tick each.
    private const int FADE_STEP = 5;

    // Daylight: the page colours follow the time of day.
    private const int DAY_DARKEN_RED = 80;
    private const int DAY_DARKEN_GREEN = 61;
    private const int DAY_DARKEN_BLUE = 62;
    private const int LIGHT_SPELL_BONUS = 200;

    /// The daylight fade runs every fourth tick.
    private const ushort FADE_TICK_MASK = 3;

    /// ViewStatus above this means the screen has to be rebuilt.
    private const sbyte VIEW_REBUILD = 97;

    /// Fades the page colours to the levels r, g, b (percent). With limit the night limits apply
    /// (red never below 10%, green 25%, blue 60%) and some blue is mixed in from green, which
    /// gives the bluish moonlight. The light spell (LightTimer) makes reds as bright as greens.
    /// Colour 31 is changed first because it depends on where the hero is (swamp, secret room).
    public static void FadePage(Data.Game game, IOutput output, (short R, short G, short B) levels, short limit, ushort[] colors)
    {
        game.PageColors[COLOR_31] = Color31Here(game.World);

        bool night = limit != 0;
        ((int r, int g, int b), int blueFromGreen) = ClampLevels((levels.R, levels.G, levels.B), night);
        ushort[] faded = new ushort[COLOR_COUNT];

        for (int index = 0; index < COLOR_COUNT; index++)
        {
            faded[index] = FadedColor(index, colors[index], (r, g, b), blueFromGreen, night, game.World.LightTimer != 0);
        }

        output.LoadPageColors(faded);
    }

    /// The teleport effect: random colours flicker for a moment (32 flashes).
    public static void FlickerColors(Data.Game game, IOutput output)
    {
        for (int flash = 0; flash < COLOR_COUNT; flash++)
        {
            ushort[] colors = new ushort[COLOR_COUNT];

            for (int index = 1; index < COLOR_COUNT; index++)
            {
                colors[index] = (ushort)game.Rng.BitRand(RANDOM_COLOR_MASK);
            }

            output.LoadPageColors(colors);
            Display.Delay(game, output, 1);
        }
    }

    /// Fades the page out to black.
    public static void FadeDown(Data.Game game, IOutput output)
    {
        for (int level = FULL_LEVEL; level >= 0; level -= FADE_STEP)
        {
            FadeToLevel(game, output, (short)level);
        }
    }

    /// Fades the page back in from black.
    public static void FadeNormal(Data.Game game, IOutput output)
    {
        for (int level = 0; level <= FULL_LEVEL; level += FADE_STEP)
        {
            FadeToLevel(game, output, (short)level);
        }
    }

    /// Updates the page colours for the time of day, every fourth tick. LightLevel follows the
    /// sun; inside buildings (regions 8 and 9) it is always bright.
    public static void DayFade(Data.Game game, IOutput output)
    {
        int bonus = game.World.LightTimer != 0 ? LIGHT_SPELL_BONUS : 0;

        if ((game.World.DayClock & FADE_TICK_MASK) != 0 && game.Ui.ViewStatus <= VIEW_REBUILD)
        {
            return;
        }

        ushort[] colors = (ushort[])game.PageColors.Clone();

        if (game.World.Region < Regions.FIRST_INDOOR_REGION)
        {
            int level = game.World.LightLevel;

            FadePage(
                game,
                output,
                ((short)(level - DAY_DARKEN_RED + bonus), (short)(level - DAY_DARKEN_GREEN), (short)(level - DAY_DARKEN_BLUE)),
                1,
                colors);
        }
        else
        {
            FadePage(game, output, (FULL_LEVEL, FULL_LEVEL, FULL_LEVEL), 1, colors);
        }
    }

    // One step of a fade to or from black: all three colours at the same level, then one tick.
    private static void FadeToLevel(Data.Game game, IOutput output, short level)
    {
        ushort[] colors = (ushort[])game.PageColors.Clone();

        FadePage(game, output, (level, level, level), 0, colors);
        Display.Delay(game, output, 1);
    }

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
