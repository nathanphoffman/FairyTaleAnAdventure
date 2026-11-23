using FaeryTale.Game.Data;
using FaeryTale.Gfx;
using static FaeryTale.Gfx.MenuStyle;

namespace FaeryTale.Game.Startup;

/// LaunchMenu: where the options sit.
internal static partial class LaunchMenu
{
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
    internal static int? OptionAt(List<Block> blocks, int positionX, int positionY)
    {
        foreach ((int entry, (int Left, int Top, int Width, int Height) area) in new[] { (QUIT, QuitArea()), (BOOK, BookArea()), (PROLOGUE, PrologueArea()) })
        {
            if (positionX >= area.Left && positionX < area.Left + area.Width && positionY >= area.Top && positionY < area.Top + area.Height)
            {
                return entry;
            }
        }

        for (int index = 0; index < blocks.Count; index++)
        {
            (int blockTop, int blockBottom) = blocks[index].PanelSpan;

            if (positionX >= SIDE_MARGIN && positionX < Overlay.WIDTH - SIDE_MARGIN && positionY >= blockTop && positionY < blockBottom)
            {
                return index;
            }
        }

        return null;
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
}
