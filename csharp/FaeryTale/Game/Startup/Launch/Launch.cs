using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;
using FaeryTale.Platform;

namespace FaeryTale.Game.Startup;

/// Starting the game: the title screen, the intro and getting the first brother started, up to
/// the first frame of the game, and RunGame (the original main).
public static class Launch
{
    // Where the view ports sit once the game starts.
    private const short PAGE_HEIGHT = 143;
    private const short TEXT_HEIGHT = 57;
    private const short LOWRES_WIDTH = 288;
    private const short PAGE_VIEW_HEIGHT = 140;
    private const short PAGE_VIEW_X = 16;

    // Screen modes (ViewStatus).
    private const sbyte VIEW_REDRAW = 99;

    /// The intro zoom: the normal size (half the width in pixels).
    private const int ZOOM_NORMAL = 156;

    /// Pens: the title text.
    private const byte TITLE_PEN = 1;

    // The title screen's palette: a blue page background and white text; the play screen's
    // background is darker while the panel loads.
    private const int BACKGROUND_COLOR = 0;
    private const int TEXT_COLOR = 1;
    private static readonly (uint, uint, uint) TITLE_BLUE = (0, 0, 6);
    private static readonly (uint, uint, uint) WHITE_RGB = (15, 15, 15);
    private static readonly (uint, uint, uint) LOADING_BLUE = (0, 0, 3);

    // Pens of the story text.
    private const byte STORY_PEN = 10;
    private const byte STORY_BACKGROUND_PEN = 11;

    /// The intro tune (its tracks are 12 to 15).
    private const int INTRO_TUNE = 3;

    /// Waits, in fiftieths of a second. In a real launch the title screen stays up a little less
    /// (the tests keep the original timing).
    private const int TITLE_WAIT = 50;
    private const int LAUNCH_TITLE_WAIT = 40;
    private const int PLACARD_WAIT = 20;

    /// The placard shown while the copy protection question is asked.
    private const int PLACARD_COPYRIGHT = 19;

    /// Disk track range holding the shadow masks.
    private const short SHADOW_FIRST_BLOCK = 896;
    private const short SHADOW_BLOCKS = 24;

    /// The message in the corner while the intro plays.
    private const string SKIP_HINT = "Press ESC to skip";

    // A caption under the original title screen, so it is clear that screen is the original game's
    // own (its text is left exactly as it was, copyright notice included).
    private const string TITLE_CAPTION = "Original title screen";
    private const int CAPTION_X = 16;
    private const int CAPTION_Y = 390;
    private const uint CAPTION_COLOR = 0xffa0_a0c0;

    /// The code Escape gives while it is set to skip the intro.
    public const int ESCAPE_KEY = 27;

    /// Whether the intro and the launch menu are skipped (the tests and --skip-launch).
    public static bool LaunchSkipped { get; set; }

    /// The game: the original main.
    public static void RunGame(IDisplayDevice device)
    {
        Data.Game game = new();
        Video video = new() { Device = device };

        game.World.LightTimer = 0;

        if (OpenClose.OpenAll(game, video) is var error && error != 0)
        {
            TextRoutines.ShowError(video, error);
            OpenClose.CloseAll();

            return;
        }

        if (StartUp(game, video))
        {
            GameLoop.Run(game, video);
            Music.Player.StopScore();
        }

        OpenClose.CloseAll();
    }

    /// Whether the intro is being skipped. Once Space or Escape has been seen the skip sticks (the
    /// original reset it on every poll without a key).
    public static int IntroSkipped(Data.Ui ui)
    {
        int key = InputHandler.NextKey();

        if (key == ' ' || key == ESCAPE_KEY)
        {
            ui.IntroSkipRequested = 1;
        }

        return ui.IntroSkipRequested;
    }

    /// Whether to leave the intro right now: the player has pressed Space or Escape. The tests
    /// skip the intro too, but they keep its full timing so their screenshots stay the same, so
    /// this is only true in a real launch.
    public static bool LeaveIntroNow(Data.Ui ui) => !LaunchSkipped && IntroSkipped(ui) != 0;

    /// Like Delay, but ends early when the player skips the intro.
    public static void SkippableDelay(Data.Game game, Video video, int ticks)
    {
        for (int tick = 0; tick < Math.Max(ticks, 1); tick++)
        {
            if (LeaveIntroNow(game.Ui))
            {
                return;
            }

            Display.Delay(game, video, 1);
        }
    }

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

    // Returns false if the player failed the copy protection or quit. In a launch the splash, the
    // intro and the launch menu come first; the tests skip them (a scripted Space press skips the
    // intro) but keep the intro's timing.
    private static bool StartUp(Data.Game game, Video video)
    {
        Music.Player.StopScore();
        bool launching = !LaunchSkipped;

        if (launching)
        {
            WindowControls.EscapeSkips = true;
            Splash.Show(game, video, SKIP_HINT);
            video.SetOverlay(TitleOverlay());
        }

        ShowTitleScreen(game, video);

        if (launching)
        {
            video.SetOverlay(Overlay.CornerHint(SKIP_HINT));
        }

        PrepareIntro(game, video);

        if (IntroSkipped(game.Ui) == 0)
        {
            StoryPages.PlayIntro(game, video);
        }

        if (launching)
        {
            WindowControls.EscapeSkips = false;
            video.SetOverlay(new Overlay());

            if (LaunchMenu.ChooseExperience(game, video) is not { } experience)
            {
                return false;
            }

            game.Experience = experience;
        }

        ShowPlayScreenBackground(game, video);
        Display.Delay(game, video, PLACARD_WAIT);
        StartFirstBrother(game, video);

        return true;
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

        byte pen = video.Screen.GetPort(PortId.Map).Pen.Fg;

        video.Screen.SetAPen(PortId.Map, TITLE_PEN);
        TextRoutines.PrintPositioned(video, StoryText.TITLE_TEXT);
        video.Screen.SetAPen(PortId.Map, pen);

        SkippableDelay(game, video, TitleWait());

        video.Screen.CurrentPort = PortId.Text;
        video.Screen.GetPort(PortId.Text).Target = BitmapId.TextPlane0;
        video.Screen.SetFontKind(PortId.Text, FontKind.Story);
        video.Screen.SetAPen(PortId.Text, STORY_PEN);
        video.Screen.SetBPen(PortId.Text, STORY_BACKGROUND_PEN);

        Audio.SoundBank.ReadScore();
        Audio.SoundBank.ReadSample();

        SkippableDelay(game, video, TitleWait());
    }

    // How long each pause on the title screen lasts, in ticks.
    private static int TitleWait() => LaunchSkipped ? TITLE_WAIT : LAUNCH_TITLE_WAIT;

    // Starts the intro tune, and flips the pages twice so both show the same thing.
    internal static void PrepareIntro(Data.Game game, Video video)
    {
        video.Screen.VpPage.Ras = 0;
        Music.Player.PlayScore(INTRO_TUNE);
        video.Screen.LoadColors(ViewPortId.Text, Tables.BlackColors);
        video.Screen.Pages[1].Ras = 1;
        ScreenEffects.SetIntroZoom(game, video, 0);

        Display.FlipPages(game, video); // "prime the pump"
        Display.FlipPages(game, video);
    }

    // Switches to the normal playing screen: loads the shadow masks and the panel picture, and
    // shows the copyright placard.
    internal static void ShowPlayScreenBackground(Data.Game game, Video video)
    {
        video.Screen.Pages[1].Ras = 1;
        ClearBothPages(video.Screen);
        video.Screen.LoadColors(ViewPortId.Text, Tables.BlackColors);
        ScreenEffects.SetIntroZoom(game, video, ZOOM_NORMAL);
        SetPageColor(video.Screen, BACKGROUND_COLOR, LOADING_BLUE);

        Disk.LoadTrackRange(SHADOW_FIRST_BLOCK, SHADOW_BLOCKS, video.Graphics.Shadow);

        SetPageColor(video.Screen, BACKGROUND_COLOR, TITLE_BLUE);
        IffPicture.LoadPicture("hiscreen", video.Screen.GetBitmap(BitmapId.Text), 0, 0);
        SetPageColor(video.Screen, TEXT_COLOR, WHITE_RGB);

        video.Screen.CurrentPort = PortId.Map;
        video.Screen.GetPort(PortId.Map).Target = video.Screen.DrawingBitmapId;
        MessagePage.ShowStillScreen(game, video);
        video.Screen.SetAPen(PortId.Map, TITLE_PEN);
        TextRoutines.PlacardText(video, PLACARD_COPYRIGHT);

        InputHandler.ResetKeyBuffer();
    }

    // Starts the first brother and sets up the text page and the colours for play.
    internal static void StartFirstBrother(Data.Game game, Video video)
    {
        for (int ras = 0; ras < 2; ras++)
        {
            video.Screen.Ras[ras].X = 0;
            video.Screen.Ras[ras].Y = 0;
        }

        Music.Player.StopScore();
        Creatures.Revive.Revival.Revive(game, video, 1);

        video.Screen.GetPort(PortId.Map).Target = video.Screen.DrawingBitmapId;
        video.Screen.VpText.Y = PAGE_HEIGHT;
        video.Screen.VpText.Height = TEXT_HEIGHT;
        video.Screen.VpPage.Height = PAGE_VIEW_HEIGHT;
        video.Screen.VpPage.X = PAGE_VIEW_X;
        video.Screen.VpPage.Width = LOWRES_WIDTH;
        video.Screen.VpPage.Y = 0;
        video.Screen.LoadColors(ViewPortId.Text, Tables.TextColors);

        foreach (FPage page in video.Screen.Pages)
        {
            page.ObCount = 0;
            page.WFlag = 0;
        }

        game.Ui.ViewStatus = VIEW_REDRAW;
        game.Ui.CurrentMenu = 0;
        Menus.PrintOptions(game, video);
    }
}
