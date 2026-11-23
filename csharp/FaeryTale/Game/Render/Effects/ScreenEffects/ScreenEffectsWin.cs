using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Render;

/// ScreenEffects: the zoom and sunrise of the ending.
public static partial class ScreenEffects
{
    /// The size of the picture during the intro's zoom: x is half the width in pixels (0 to 160),
    /// the height keeps the 8:5 shape. Moves the page and text view ports to match, fades the
    /// colours with the size (black when small), and shows the result.
    public static void SetIntroZoom(Data.Game game, IOutput output, int positionX)
    {
        int positionY = positionX * ZOOM_ASPECT_HEIGHT / ZOOM_ASPECT_WIDTH;

        Display.Delay(game, output, 2);
        output.SetZoomSize(positionX, positionY);
        Fading.FadePage(
            game,
            output,
            ((short)(positionY * 2 - ZOOM_FADE_RED), (short)(positionY * 2 - ZOOM_FADE_GREEN), (short)(positionY * 2 - ZOOM_FADE_BLUE)),
            0,
            Tables.IntroColors);
        Display.FlipPages(game, output);
    }

    /// The win screen: shows the picture and brings the sun up behind it by cycling the sky
    /// colours through SunColors, with two glowing colours that fade out near the end.
    public static void WinColors(Data.Game game, IOutput output)
    {
        TextRoutines.PlacardText(output, WIN_MESSAGE_FIRST);
        Messages.Name(game, output);
        TextRoutines.PlacardText(output, WIN_MESSAGE_SECOND);
        output.DrawPlacardBorder();
        Display.Delay(game, output, WIN_MESSAGE_PAUSE);
        ShowWinPictureInBlack(game, output);
        RiseSun(game, output);

        Display.Delay(game, output, SUNRISE_END_PAUSE);
        output.LoadPageColors(Tables.BlackColors);
    }

    /// The palette for step number of the sunrise over the ending picture. Colours 1 and 28 are
    /// white, 0 and 31 black; the sky and sun colours come from SunColors offset by number, and
    /// the glow (29, 30) is a fixed orange at first and then fades out.
    internal static ushort[] SunrisePalette(int number)
    {
        ushort[] colors = new ushort[Fading.COLOR_COUNT];

        foreach (int white in WHITE_COLORS)
        {
            colors[white] = WHITE;
        }

        for (int color = FIRST_SUNRISE_COLOR; color <= LAST_SUNRISE_COLOR; color++)
        {
            int index = number + color;

            colors[color] = index > 0 ? (ushort)ObjectTables.SunColors[index] : (ushort)0;
        }

        if (number > SUN_GLOW_UNTIL)
        {
            colors[GLOW_FIRST] = GLOW_BRIGHT_FIRST;
            colors[GLOW_SECOND] = GLOW_BRIGHT_SECOND;
        }
        else
        {
            int fade = (number + GLOW_FADE_OFFSET) / 2;

            colors[GLOW_FIRST] = (ushort)(RED_STEP * fade);
            colors[GLOW_SECOND] = (ushort)(RED_STEP * (fade / 2));
        }

        return colors;
    }

    // Shows the winning picture with every colour black, so the sunrise can fade it in.
    private static void ShowWinPictureInBlack(Data.Game game, IOutput output)
    {
        output.LoadPictureToDrawingPage("winpic");
        output.LoadPageColors(Tables.BlackColors);
        output.LoadPanelColors(Tables.BlackColors);
        output.SetPanelVisible(false);
        SetIntroZoom(game, output, WIN_SCREEN_SIZE);
    }

    // Steps through the sunrise palettes, one pause each.
    private static void RiseSun(Data.Game game, IOutput output)
    {
        for (int number = SUNRISE_START; number > SUNRISE_END; number--)
        {
            output.LoadPageColors(SunrisePalette(number));

            if (number == SUNRISE_START)
            {
                Display.Delay(game, output, SUNRISE_FIRST_PAUSE);
            }

            Display.Delay(game, output, SUNRISE_STEP_PAUSE);
        }
    }
}
