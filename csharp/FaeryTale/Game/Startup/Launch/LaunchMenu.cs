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
internal static class LaunchMenu
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

    /// The wooden boards behind the heading, each option and Quit, made once. Each entry has a
    /// plain board and a lit one (for when the pointer is on it).
    private sealed record Planks(Image Heading, Image[][] Options, Image[] Quit, Image[] Book, Image[] Prologue);

    /// Shows the menu and waits for a click on an option (or its number key); returns what was
    /// chosen and, for Play, which version of the game.
    public static (LaunchAction Action, Experience Experience) Choose(Data.Game game, Video video)
    {
        List<Block> blocks = Layout();

        Launch.PrepareBluePage(game, video);

        MenuBackground background = MenuBackground.Load(game, video);
        uint tick = 0;

        WindowControls.SystemCursorShown = true;
        Planks planks = MakePlanks(blocks);
        video.SetOverlay(MenuOverlay(blocks, planks, null));

        int? drawn = null;
        int choice;

        while (true)
        {
            (int pointerX, int pointerY) = InputHandler.RawPointer;

            // The pointer is in lores game pixels; the menu is in overlay pixels, twice as many.
            int? hovered = OptionAt(blocks, pointerX * 2, pointerY * 2);

            if (hovered != drawn)
            {
                video.SetOverlay(MenuOverlay(blocks, planks, hovered));
                drawn = hovered;
            }

            background.Draw(video, tick);
            tick++;

            if (hovered is { } option && InputHandler.LeftButtonDown)
            {
                choice = option;

                break;
            }

            if (OptionForKey(InputHandler.NextKey()) is { } keyOption)
            {
                choice = keyOption;

                break;
            }

            Display.Delay(game, video, 1);
        }

        // Wait for the button to go up, so the click does not carry on into the game.
        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }

        WindowControls.SystemCursorShown = false;
        video.SetOverlay(new Overlay());
        InputHandler.ResetKeyBuffer();
        InputHandler.ForgetLastMenu();

        return choice switch
        {
            QUIT => (LaunchAction.Quit, Experience.Original),
            BOOK => (LaunchAction.ShowBook, Experience.Original),
            PROLOGUE => (LaunchAction.ShowPrologue, Experience.Original),
            _ => (LaunchAction.Play, OPTIONS[choice].Experience),
        };
    }

    /// Breaks text into lines of at most width characters, between words.
    internal static List<string> Wrap(string text, int width)
    {
        List<string> lines = [];
        int start = 0;
        int? lastSpace = null;

        for (int index = 0; index < text.Length; index++)
        {
            if (text[index] == ' ')
            {
                lastSpace = index;
            }

            if (index + 1 - start > width && lastSpace is { } space)
            {
                lines.Add(text[start..space]);
                start = space + 1;
                lastSpace = null;
            }
        }

        lines.Add(text[start..]);

        return lines;
    }

    /// Works out where every option goes, one block under the other.
    internal static List<Block> Layout()
    {
        List<Block> blocks = [];
        int nextY = FIRST_OPTION_Y;

        foreach ((string _, string description, Experience _) in OPTIONS)
        {
            Block block = new(nextY, Wrap(description, DESCRIPTION_WIDTH));

            blocks.Add(block);
            nextY = block.PanelSpan.Bottom + PANEL_GAP + TITLE_ASCENT + PANEL_PADDING;
        }

        return blocks;
    }

    /// The option whose block the point (overlay pixels) is in, if any; QUIT for the Quit entry. A
    /// block is hit anywhere from side margin to side margin.
    internal static int? OptionAt(List<Block> blocks, int x, int y)
    {
        foreach ((int entry, (int Left, int Top, int Width, int Height) area) in new[] { (QUIT, QuitArea()), (BOOK, BookArea()), (PROLOGUE, PrologueArea()) })
        {
            if (x >= area.Left && x < area.Left + area.Width && y >= area.Top && y < area.Top + area.Height)
            {
                return entry;
            }
        }

        for (int index = 0; index < blocks.Count; index++)
        {
            (int blockTop, int blockBottom) = blocks[index].PanelSpan;

            if (x >= SIDE_MARGIN && x < Overlay.WIDTH - SIDE_MARGIN && y >= blockTop && y < blockBottom)
            {
                return index;
            }
        }

        return null;
    }

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

    /// The area behind the Quit entry.
    internal static (int Left, int Top, int Width, int Height) QuitArea()
    {
        int left = QuitLeft() - PANEL_PADDING;

        return (left, QUIT_Y - TITLE_ASCENT - PANEL_PADDING, Overlay.WIDTH - left, TITLE_ASCENT + PANEL_PADDING * 2 + Overlay.GLYPH_SIZE);
    }

    /// The areas behind the two replay entries, side by side from the left margin.
    internal static (int Left, int Top, int Width, int Height) BookArea() => ReplayArea(BOOK_LABEL, REPLAY_LEFT);

    internal static (int Left, int Top, int Width, int Height) PrologueArea()
    {
        (int left, int _, int width, int _) = BookArea();

        return ReplayArea(PROLOGUE_LABEL, left + width + PANEL_GAP);
    }

    // One replay entry: its text on a board starting at left.
    private static (int Left, int Top, int Width, int Height) ReplayArea(string label, int left) =>
        (left, QUIT_Y - TITLE_ASCENT - PANEL_PADDING, Overlay.TextWidth(label, TITLE_SCALE) + PANEL_PADDING * 2, TITLE_ASCENT + PANEL_PADDING * 2 + Overlay.GLYPH_SIZE);

    /// The description of option number (for tests).
    internal static string DescriptionOf(int number) => OPTIONS[number].Description;

    // The x position of the Quit entry: its text ends at the right margin.
    private static int QuitLeft() => QUIT_RIGHT - Overlay.TextWidth(QUIT_LABEL, TITLE_SCALE);

    // Makes a board for each area of the menu.
    private static Planks MakePlanks(List<Block> blocks)
    {
        Image[][] options = blocks.Select((block, index) => Board(BlockArea(block, OPTIONS[index].Label), 21u + (uint)index)).ToArray();

        return new Planks(Board(HeadingArea(), 11)[0], options, Board(QuitArea(), 31), Board(BookArea(), 32), Board(PrologueArea(), 33));
    }

    // The heading, the options and Quit as text on wooden boards, the hovered one lit.
    private static Overlay MenuOverlay(List<Block> blocks, Planks planks, int? hovered)
    {
        Overlay overlay = new();

        overlay.Images.Add(planks.Heading);
        AddCentered(overlay, HEADING, HEADING_Y, HEADING_COLOR, TITLE_SCALE);

        for (int index = 0; index < blocks.Count; index++)
        {
            bool lit = hovered == index;
            Block block = blocks[index];

            overlay.Images.Add(planks.Options[index][lit ? 1 : 0]);
            AddCentered(overlay, OPTIONS[index].Label, block.TitleY, lit ? HOVER_COLOR : TITLE_COLOR, TITLE_SCALE);

            for (int number = 0; number < block.Lines.Count; number++)
            {
                AddCentered(overlay, block.Lines[number], block.TitleY + DESCRIPTION_OFFSET + number * LINE_SPACING, lit ? LIT_DESCRIPTION_COLOR : DESCRIPTION_COLOR, BODY_SCALE);
            }
        }

        bool quitLit = hovered == QUIT;

        overlay.Images.Add(planks.Quit[quitLit ? 1 : 0]);
        AddText(overlay, QUIT_LABEL, (QuitLeft(), QUIT_Y), quitLit ? HOVER_COLOR : TITLE_COLOR, TITLE_SCALE);
        AddReplayEntry(overlay, (planks.Book, BOOK_LABEL, BookArea().Left), hovered == BOOK);
        AddReplayEntry(overlay, (planks.Prologue, PROLOGUE_LABEL, PrologueArea().Left), hovered == PROLOGUE);

        return overlay;
    }

    // A replay entry: its board (lit or not) and its label.
    private static void AddReplayEntry(Overlay overlay, (Image[] Boards, string Label, int Left) entry, bool lit)
    {
        overlay.Images.Add(entry.Boards[lit ? 1 : 0]);
        AddText(overlay, entry.Label, (entry.Left + PANEL_PADDING, QUIT_Y), lit ? HOVER_COLOR : TITLE_COLOR, TITLE_SCALE);
    }

    // The area behind the heading.
    private static (int Left, int Top, int Width, int Height) HeadingArea()
    {
        int width = Overlay.TextWidth(HEADING, TITLE_SCALE) + PANEL_PADDING * 2;

        return ((Overlay.WIDTH - width) / 2, HEADING_Y - TITLE_ASCENT - PANEL_PADDING, width, TITLE_ASCENT + PANEL_PADDING * 2);
    }

    // The area behind an option: as wide as its longest line plus some padding.
    private static (int Left, int Top, int Width, int Height) BlockArea(Block block, string title)
    {
        int longest = block.Lines.Max(line => Overlay.TextWidth(line, BODY_SCALE));
        int width = Math.Max(longest, Overlay.TextWidth(title, TITLE_SCALE)) + PANEL_PADDING * 2;
        (int top, int bottom) = block.PanelSpan;

        return ((Overlay.WIDTH - width) / 2, top, width, bottom - top);
    }
}
