using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;
using FaeryTale.Platform;

namespace FaeryTale.Game.Startup;

/// Starting the game: the title screen, the intro and getting the first brother started, up to
/// the first frame of the game, and RunGame (the original main).
public static partial class Launch
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
            StopMusic();
            video.SetOverlay(new Overlay());
            backToMenu = !game.Ui.ExitProgram;
        }

        OpenClose.CloseAll();

        return backToMenu;
    }

    // Clears the keyboard, mouse and screen state a game leaves behind in static fields.
    private static void ForgetLastGame()
    {
        InputHandler.Reset();
        Conversation.Close();
        EnemyPathing.Reset();
        ClickOverlay.Reset();
        MenuGreying.Reset();
        PauseMenu.Reset();
        Journal.Reset();
        WindowControls.SystemCursorShown = false;
        WindowControls.NativeRendering = false;
    }

    // Returns false if the player failed the copy protection or quit. In a launch the splash, the
    // intro and the launch menu come first; the tests skip them (a scripted Space press skips the
    // intro) but keep the intro's timing.
    private static bool StartUp(Data.Game game, Video video)
    {
        StopMusic();
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
        StopMusic();
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

    // Whether the menu's tune (the intro tune) is playing. The menu is shown again after the book, the
    // prologue and every game, and the tune must be there each time, so it is started when it is not.
    private static bool menuMusicPlaying;
}
