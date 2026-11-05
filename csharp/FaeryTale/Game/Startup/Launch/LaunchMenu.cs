using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Platform;

namespace FaeryTale.Game.Startup;

/// The launch menu, shown after the intro: the player picks which version of the game to play.
/// For now all three start the same game; the choice is kept in Game.Experience. A Quit entry in
/// the bottom right corner leaves the game.
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
        ("Remastered Experience", "The original experience with modern conveniences (maps, menus, visual upscales, etc.).", Experience.Remastered),
        ("Reimagined Experience", "The remastered experience with additional content by Nathan Hoffman to extend and reimagine the game with more TTRPG flair, story, spells, monsters, and places.", Experience.Reimagined),
    ];

    private const string HEADING = "Choose your adventure";

    // Titles are drawn at the size of the classic game text, descriptions at half that.
    private const int TITLE_SCALE = 2;
    private const int BODY_SCALE = 1;

    // The Quit entry sits in the bottom right corner. As a hover or a choice it is numbered one
    // past the real options.
    private const string QUIT_LABEL = "Quit";
    private const int QUIT = 3;
    private const int QUIT_Y = 380;
    private const int QUIT_RIGHT = Overlay.WIDTH - 16;

    // Descriptions are wrapped to this many characters, and sit this far apart.
    private const int DESCRIPTION_WIDTH = 64;
    private const int LINE_SPACING = 13;

    // Baseline of the heading and of the first option's title.
    private const int HEADING_Y = 56;
    private const int FIRST_OPTION_Y = 124;

    // A title is this far above its baseline, and its first description line this far below it.
    private const int TITLE_ASCENT = 14 * TITLE_SCALE / 2;
    private const int DESCRIPTION_OFFSET = 21;

    // Space between the text and the edge of its dark panel, and between two panels.
    private const int PANEL_PADDING = 8;
    private const int PANEL_GAP = 12;

    /// An entry can be clicked from side margin to side margin.
    private const int SIDE_MARGIN = 16;

    // Colours of the text on the wood: cream, gold for the heading, yellow under the pointer, with
    // a dark shadow behind each letter so it can be read on the grain.
    private const uint TITLE_COLOR = 0xffff_f4dc;
    private const uint HEADING_COLOR = 0xffff_e08a;
    private const uint HOVER_COLOR = 0xffff_e566;
    private const uint DESCRIPTION_COLOR = 0xffee_dcb8;
    private const uint LIT_DESCRIPTION_COLOR = Overlay.WHITE;
    private const uint TEXT_SHADOW_COLOR = 0xff1e_1006;

    /// The wooden boards behind the heading, each option and Quit, made once. Each entry has a
    /// plain board and a lit one (for when the pointer is on it).
    private sealed record Planks(Image Heading, Image[][] Options, Image[] Quit);

    /// Shows the menu and waits for a click on an option (or its number key); returns the choice,
    /// or null when the player chose Quit.
    public static Experience? ChooseExperience(Data.Game game, Video video)
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

        return choice < OPTIONS.Length ? OPTIONS[choice].Experience : null;
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
        (int left, int top, int _, int height) = QuitArea();

        if (x >= left && y >= top && y < top + height)
        {
            return QUIT;
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

    /// The option a key chooses: 1, 2 or 3, or Q to quit.
    internal static int? OptionForKey(int key)
    {
        if (key == 'Q')
        {
            return QUIT;
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

    /// The description of option number (for tests).
    internal static string DescriptionOf(int number) => OPTIONS[number].Description;

    // The x position of the Quit entry: its text ends at the right margin.
    private static int QuitLeft() => QUIT_RIGHT - Overlay.TextWidth(QUIT_LABEL, TITLE_SCALE);

    // Makes a board for each area of the menu.
    private static Planks MakePlanks(List<Block> blocks)
    {
        Image[][] options = blocks.Select((block, index) => Board(BlockArea(block, OPTIONS[index].Label), 21u + (uint)index)).ToArray();

        return new Planks(Board(HeadingArea(), 11)[0], options, Board(QuitArea(), 31));
    }

    // A plain and a lit board for an area (left, top, width, height).
    private static Image[] Board((int Left, int Top, int Width, int Height) area, uint seed)
    {
        Image Make(bool lit) => new(area.Left, area.Top, area.Width + Wood.SHADOW, area.Height + Wood.SHADOW, Wood.Plank(area.Width, area.Height, seed, lit));

        return [Make(false), Make(true)];
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

        return overlay;
    }

    // Adds a line of text in the middle of the screen with its baseline at y.
    private static void AddCentered(Overlay overlay, string text, int y, uint color, int scale) =>
        AddText(overlay, text, ((Overlay.WIDTH - Overlay.TextWidth(text, scale)) / 2, y), color, scale);

    // Adds a line of text with a dark shadow behind it, down and to the right.
    private static void AddText(Overlay overlay, string text, (int X, int Y) at, uint color, int scale)
    {
        overlay.Texts.Add(new Text(text, at.X + scale, at.Y + scale, TEXT_SHADOW_COLOR, scale));
        overlay.Texts.Add(new Text(text, at.X, at.Y, color, scale));
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
