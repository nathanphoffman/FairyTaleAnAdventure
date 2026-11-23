using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using static FaeryTale.Gfx.MenuStyle;
using FaeryTale.Platform;

namespace FaeryTale.Game.Startup;

/// What the player chose on the launch menu.
internal enum LaunchAction
{
    Play,
    ShowBook,
    ShowPrologue,
    Quit,
}

/// The launch menu, shown after the intro: the player picks which version of the game to play.
/// For now all three start the same game; the choice is kept in Game.Experience. A Quit entry in
/// the bottom right corner leaves the game, and two entries in the bottom left corner replay the
/// story book and the opening text.
internal static partial class LaunchMenu
{
    /// Where one option sits: the baseline of its title and the lines of its description.
    internal sealed record Block(int TitleY, List<string> Lines)
    {
        /// The baseline of the last line of the block.
        public int LastBaseline => TitleY + DESCRIPTION_OFFSET + (Lines.Count - 1) * LINE_SPACING;

        /// The top and bottom of the dark panel behind the block.
        public (int Top, int Bottom) PanelSpan => (TitleY - TITLE_ASCENT - PANEL_PADDING, LastBaseline + PANEL_PADDING);
    }

    /// The things on offer, top to bottom: the label, a description under it, and what it starts.
    private static readonly (string Label, string Description, Experience Experience)[] OPTIONS =
    [
        ("Original Experience", "The way David Joiner made the game (converted for modern PCs).", Experience.Original),
        ("Remastered Experience", "The original experience with modern conveniences.", Experience.Remastered),
        ("Reimagined Experience", "The remastered experience with reworked content.", Experience.Reimagined),
    ];

    // Each board's wood grain is made from its own seed so no two boards look alike.
    private const uint OPTION_BOARD_SEED_BASE = 21;
    private const uint HEADING_BOARD_SEED = 11;
    private const uint QUIT_BOARD_SEED = 31;
    private const uint BOOK_BOARD_SEED = 32;
    private const uint PROLOGUE_BOARD_SEED = 33;

    private const string HEADING = "Choose your adventure";

    // Descriptions are drawn at half the size of the titles.
    private const int BODY_SCALE = 1;

    // The Quit entry sits in the bottom right corner. As a hover or a choice it is numbered one
    // past the real options.
    private const string QUIT_LABEL = "Quit";
    private const int QUIT = 3;
    private const int QUIT_Y = 380;
    private const int QUIT_RIGHT = Overlay.WIDTH - 16;

    // The replay entries sit in the bottom left corner, numbered after Quit.
    private const string BOOK_LABEL = "Show Book";
    private const string PROLOGUE_LABEL = "Show Prologue";
    private const int BOOK = 4;
    private const int PROLOGUE = 5;
    private const int REPLAY_LEFT = 16;

    // Descriptions are wrapped to this many characters, and sit this far apart.
    private const int DESCRIPTION_WIDTH = 64;
    private const int LINE_SPACING = 13;

    // Baseline of the heading and of the first option's title.
    private const int HEADING_Y = 56;
    private const int FIRST_OPTION_Y = 124;

    // A description's first line is this far below its title, and two panels are this far apart.
    private const int DESCRIPTION_OFFSET = 21;
    private const int PANEL_GAP = 12;

    /// An entry can be clicked from side margin to side margin.
    private const int SIDE_MARGIN = 16;

    /// Shows the menu and waits for a click on an option (or its number key); returns what was
    /// chosen and, for Play, which version of the game.
    public static (LaunchAction Action, Experience Experience) Choose(Data.Game game, Video video)
    {
        List<Block> blocks = Layout();

        Launch.PrepareBluePage(game, video);

        MenuBackground background = MenuBackground.Load(game, video);

        WindowControls.SystemCursorShown = true;
        Planks planks = MakePlanks(blocks);
        video.SetOverlay(MenuOverlay(blocks, planks, null));

        int choice = WaitForChoice(game, video, (blocks, planks), background);

        LeaveMenu(game, video);

        return ActionFor(choice);
    }

    // Draws the menu until an option is clicked or its key is pressed; returns the option.
    private static int WaitForChoice(Data.Game game, Video video, (List<Block> Blocks, Planks Planks) menu, MenuBackground background)
    {
        uint tick = 0;
        int? drawn = null;

        while (true)
        {
            (int pointerX, int pointerY) = InputHandler.RawPointer;

            // The pointer is in lores game pixels; the menu is in overlay pixels, twice as many.
            int? hovered = OptionAt(menu.Blocks, pointerX * 2, pointerY * 2);

            if (hovered != drawn)
            {
                video.SetOverlay(MenuOverlay(menu.Blocks, menu.Planks, hovered));
                drawn = hovered;
            }

            background.Draw(video, tick);
            tick++;

            if (hovered is { } option && InputHandler.LeftButtonDown)
            {
                return option;
            }

            if (OptionForKey(InputHandler.NextKey()) is { } keyOption)
            {
                return keyOption;
            }

            Display.Delay(game, video, 1);
        }
    }

    // Clears the menu away and forgets the click and keys that chose the option.
    private static void LeaveMenu(Data.Game game, Video video)
    {
        // Wait for the button to go up, so the click does not carry on into the game.
        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }

        WindowControls.SystemCursorShown = false;
        video.SetOverlay(new Overlay());
        InputHandler.ResetKeyBuffer();
        InputHandler.ForgetLastMenu();
    }

    // What the launcher does for an option.
    private static (LaunchAction Action, Experience Experience) ActionFor(int choice) => choice switch
    {
        QUIT => (LaunchAction.Quit, Experience.Original),
        BOOK => (LaunchAction.ShowBook, Experience.Original),
        PROLOGUE => (LaunchAction.ShowPrologue, Experience.Original),
        _ => (LaunchAction.Play, OPTIONS[choice].Experience),
    };

    /// The option a key chooses: 1, 2 or 3, Q to quit, B for the book or P for the prologue.
    internal static int? OptionForKey(int key)
    {
        switch (key)
        {
            case 'Q':
                return QUIT;

            case 'B':
                return BOOK;

            case 'P':
                return PROLOGUE;
        }

        int number = key - '1';

        return key >= '1' && number < OPTIONS.Length ? number : null;
    }
}
