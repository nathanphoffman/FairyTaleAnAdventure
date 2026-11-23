using FaeryTale.Gfx;
using static FaeryTale.Gfx.MenuStyle;

namespace FaeryTale.Game.Startup;

/// LaunchMenu: the areas, texts and entries of the menu.
internal static partial class LaunchMenu
{
    // One replay entry: its text on a board starting at left.
    private static (int Left, int Top, int Width, int Height) ReplayArea(string label, int left) =>
        (left, QUIT_Y - TITLE_ASCENT - PANEL_PADDING, Overlay.TextWidth(label, TITLE_SCALE) + PANEL_PADDING * 2, TITLE_ASCENT + PANEL_PADDING * 2 + Overlay.GLYPH_SIZE);

    /// The description of option number (for tests).
    internal static string DescriptionOf(int number) => OPTIONS[number].Description;

    // The x position of the Quit entry: its text ends at the right margin.
    private static int QuitLeft() => QUIT_RIGHT - Overlay.TextWidth(QUIT_LABEL, TITLE_SCALE);

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
