using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Startup;

/// Launch: the blue page and the title screen.
public static partial class Launch
{
    /// Blanks the page and makes it blue with white text, ready for the title and the launch menu.
    public static void PrepareBluePage(Data.Game game, Video video)
    {
        video.Screen.VpPage.Ras = 0;
        video.Screen.Pages[1].Ras = 1;
        ClearBothPages(video.Screen);
        video.Screen.CurrentPort = PortId.Map;
        ScreenEffects.SetIntroZoom(game, video, ZOOM_NORMAL);
        SetPageColor(video.Screen, BACKGROUND_COLOR, TITLE_BLUE);
        SetPageColor(video.Screen, TEXT_COLOR, WHITE_RGB);
    }

    // The blue page of the launch menu again after a replay.
    private static void BackToBluePage(Data.Game game, Video video)
    {
        video.SetOverlay(new Overlay());
        StopMusic();
        PrepareBluePage(game, video);
        SetUpStoryText(video.Screen);
        game.Ui.IntroSkipRequested = 0;
        InputHandler.ResetKeyBuffer();
    }

    // The skip message plus the caption of the original title screen.
    private static Overlay TitleOverlay()
    {
        Overlay overlay = Overlay.CornerHint(SKIP_HINT);

        overlay.Texts.Add(new Text(TITLE_CAPTION, CAPTION_X, CAPTION_Y, CAPTION_COLOR, 1));

        return overlay;
    }

    /// Blanks both pages.
    internal static void ClearBothPages(Screen screen)
    {
        foreach (BitmapId page in new[] { screen.ViewingBitmapId, screen.DrawingBitmapId })
        {
            screen.GetPort(PortId.Map).Target = page;
            screen.Clear(PortId.Map, 0);
        }
    }

    // Sets a colour of the page palette from 4-bit red, green and blue.
    internal static void SetPageColor(Screen screen, int color, (uint R, uint G, uint B) rgb) =>
        screen.SetColor(ViewPortId.Page, color, rgb);

    // The legal title text on a blue page, then the music and the sound effects are read in.
    internal static void ShowTitleScreen(Data.Game game, Video video)
    {
        PrepareBluePage(game, video);

        byte penColor = video.Screen.GetPort(PortId.Map).Pen.Fg;

        video.Screen.SetAPen(PortId.Map, TITLE_PEN);
        TextRoutines.PrintPositioned(video, StoryText.TITLE_TEXT);
        video.Screen.SetAPen(PortId.Map, penColor);

        SkippableDelay(game, video, TitleWait());

        SetUpStoryText(video.Screen);

        Audio.SoundBank.ReadScore();
        Audio.SoundBank.ReadSample();

        SkippableDelay(game, video, TitleWait());
    }
}
