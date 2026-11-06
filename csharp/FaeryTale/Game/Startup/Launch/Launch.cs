using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
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
    private const string CLICK_HINT = "Click to continue";
    private const string BOOK_HINT = "Click to turn the page, Esc to skip";

    // A caption under the original title screen, so it is clear that screen is the original game's
    // own (its text is left exactly as it was, copyright notice included).
    private const string TITLE_CAPTION = "Original title screen";
    private const int CAPTION_X = 16;
    private const int CAPTION_Y = 390;
    private const uint CAPTION_COLOR = 0xffa0_a0c0;

    /// Whether the intro and the launch menu are skipped (the tests and --skip-launch).
    public static bool LaunchSkipped { get; set; }

    /// The game: the original main. After a game ends the player is taken back to the launch menu
    /// (unless they chose to exit the program or the test scripts are running).
    public static void RunGame(IDisplayDevice device)
    {
        Func<Data.Game, Video, bool> begin = StartUp;

        while (PlayOnce(device, begin) && !LaunchSkipped)
        {
            begin = ReturnToMenu;
        }
    }

    // One game from opening to closing, with a fresh game state. Returns true when the player
    // should be offered the launch menu again.
    private static bool PlayOnce(IDisplayDevice device, Func<Data.Game, Video, bool> begin)
    {
        Data.Game game = new();
        Video video = new() { Device = device };

        device.Source = video;
        ForgetLastGame();
        game.World.LightTimer = 0;

        if (OpenClose.OpenAll(game, video) is var error && error != 0)
        {
            TextRoutines.ShowError(video, error);
            OpenClose.CloseAll();

            return false;
        }

        bool backToMenu = false;

        if (begin(game, video))
        {
            GameLoop.Run(game, video);
            Music.Player.StopScore();
            video.SetOverlay(new Overlay());
            backToMenu = !game.Ui.ExitProgram;
        }

        OpenClose.CloseAll();

        return backToMenu;
    }

    // Clears the keyboard, mouse and walking state a game leaves behind in static fields.
    private static void ForgetLastGame()
    {
        InputHandler.Reset();
        ClickToMove.Cancel();
        EnemyPathing.Reset();
        ClickOverlay.Reset();
        WindowControls.SystemCursorShown = false;
        WindowControls.NativeRendering = false;
    }

    /// Whether the intro is being skipped. Once Space or Escape has been seen the skip sticks (the
    /// original reset it on every poll without a key).
    public static int IntroSkipped(Data.Ui ui)
    {
        int key = InputHandler.NextKey();

        if (key == ' ' || key == InputHandler.ESCAPE_KEY)
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
            Splash.Show(game, video, SKIP_HINT);
            video.SetOverlay(TitleOverlay());
        }

        ShowTitleScreen(game, video);

        if (launching)
        {
            video.SetOverlay(Overlay.CornerHint(BOOK_HINT));
        }

        PrepareIntro(game, video);

        if (IntroSkipped(game.Ui) == 0)
        {
            StoryPages.PlayIntro(game, video);
        }

        video.SetOverlay(new Overlay());

        return ChooseAndBegin(game, video);
    }

    // Back from a game: straight to the menu, without the splash and the intro. Returns false
    // when the player chose Quit.
    private static bool ReturnToMenu(Data.Game game, Video video)
    {
        Music.Player.StopScore();
        PrepareBluePage(game, video);
        SetUpStoryText(video.Screen);

        return ChooseAndBegin(game, video);
    }

    // Asks which version to play (the tests skip the question), then starts the first brother.
    // Returns false when the player chose Quit.
    private static bool ChooseAndBegin(Data.Game game, Video video)
    {
        if (!LaunchSkipped)
        {
            if (!AskWhatToDo(game, video))
            {
                return false;
            }
        }
        else if (Environment.GetEnvironmentVariable("FTA_EXPERIENCE") is { } name && Enum.TryParse(name, true, out Experience scripted))
        {
            // Lets the test scripts start in another mode, and at another difficulty.
            if (Enum.TryParse(Environment.GetEnvironmentVariable("FTA_DIFFICULTY"), true, out DifficultyLevel scriptedLevel))
            {
                Difficulty.Chosen = scriptedLevel;
            }

            ChooseExperience(game, video, scripted);
        }

        ShowPlayScreenBackground(game, video);

        if (game.Experience == Experience.Original)
        {
            Display.Delay(game, video, PLACARD_WAIT);
        }

        StartFirstBrother(game, video);

        return true;
    }

    // Shows the launch menu until the player picks a version to play (true) or Quit (false). The
    // book and the prologue are replayed on the way, and the menu comes back after each.
    private static bool AskWhatToDo(Data.Game game, Video video)
    {
        while (true)
        {
            (LaunchAction action, Experience experience) = LaunchMenu.Choose(game, video);

            switch (action)
            {
                case LaunchAction.Quit:
                    return false;

                case LaunchAction.ShowBook:
                    ReplayBook(game, video);
                    break;

                case LaunchAction.ShowPrologue:
                    ReplayPrologue(game, video);
                    break;

                default:
                    ChooseExperience(game, video, experience);

                    return true;
            }
        }
    }

    // The story book of the intro again (Space or Escape skips it).
    private static void ReplayBook(Data.Game game, Video video)
    {
        game.Ui.IntroSkipRequested = 0;
        PrepareBluePage(game, video);
        video.SetOverlay(Overlay.CornerHint(BOOK_HINT));
        PrepareIntro(game, video);
        StoryPages.PlayIntro(game, video);
        BackToBluePage(game, video);
    }

    // The text that flashes up when the original game starts: the copyright notice and the first
    // brother's story page, drawn the way the original game draws them. Each screen stays until the
    // player clicks (or presses a key); Escape leaves.
    private static void ReplayPrologue(Data.Game game, Video video)
    {
        game.Experience = Experience.Original;
        ShowPlayScreenBackground(game, video);

        if (WaitForClick(game, video))
        {
            Creatures.Revive.Brothers.ShowOpeningPlacard(game, video);
            WaitForClick(game, video);
        }

        BackToBluePage(game, video);
    }

    // Waits for a click or a key, with a hint in the corner. Returns false if the player pressed
    // Escape.
    private static bool WaitForClick(Data.Game game, Video video)
    {
        video.SetOverlay(Overlay.CornerHint(CLICK_HINT));
        InputHandler.ResetKeyBuffer();

        while (!InputHandler.LeftButtonDown)
        {
            int key = InputHandler.NextKey();

            if (key != 0)
            {
                return key != InputHandler.ESCAPE_KEY;
            }

            Display.Delay(game, video, 1);
        }

        WaitForButtonUp(game, video);

        return true;
    }

    // Waits until the mouse button is let go, so one click counts once.
    private static void WaitForButtonUp(Data.Game game, Video video)
    {
        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }
    }

    /// Like SkippableDelay, but a click also ends the wait (a real launch only, so scripted tests
    /// keep their timing). Used by the story book: a click turns the page.
    public static void PageDelay(Data.Game game, Video video, int ticks)
    {
        for (int tick = 0; tick < Math.Max(ticks, 1); tick++)
        {
            if (LeaveIntroNow(game.Ui))
            {
                return;
            }

            if (!LaunchSkipped && InputHandler.LeftButtonDown)
            {
                WaitForButtonUp(game, video);

                return;
            }

            Display.Delay(game, video, 1);
        }
    }

    // The blue page of the launch menu again after a replay.
    private static void BackToBluePage(Data.Game game, Video video)
    {
        video.SetOverlay(new Overlay());
        Music.Player.StopScore();
        PrepareBluePage(game, video);
        SetUpStoryText(video.Screen);
        game.Ui.IntroSkipRequested = 0;
        InputHandler.ResetKeyBuffer();
    }

    // Remembers the chosen mode. Remastered and Reimagined let the pointer roam the whole screen.
    private static void ChooseExperience(Data.Game game, Video video, Experience experience)
    {
        game.Experience = experience;
        game.Difficulty = Difficulty.Chosen;
        InputHandler.RemasteredControls = experience != Experience.Original;

        // Remastered and Reimagined draw the world with the new world drawer at the window's own size.
        video.Capture.Enabled = experience != Experience.Original;
        WindowControls.NativeRendering = experience != Experience.Original;
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

        SetUpStoryText(video.Screen);

        Audio.SoundBank.ReadScore();
        Audio.SoundBank.ReadSample();

        SkippableDelay(game, video, TitleWait());
    }

    // Points the text port at the panel with the story font and the paper colours for play.
    private static void SetUpStoryText(Screen screen)
    {
        screen.CurrentPort = PortId.Text;
        screen.GetPort(PortId.Text).Target = BitmapId.TextPlane0;
        screen.SetFontKind(PortId.Text, FontKind.Story);
        screen.SetAPen(PortId.Text, STORY_PEN);
        screen.SetBPen(PortId.Text, STORY_BACKGROUND_PEN);
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

        // The copyright placard is a flash of text between the menu and the game: only the
        // original game has it.
        if (game.Experience == Experience.Original)
        {
            video.Screen.SetAPen(PortId.Map, TITLE_PEN);
            TextRoutines.PlacardText(video, PLACARD_COPYRIGHT);
        }

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
