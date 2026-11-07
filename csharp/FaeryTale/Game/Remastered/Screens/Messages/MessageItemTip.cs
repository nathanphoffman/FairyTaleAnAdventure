using FaeryTale.Game.Reimagined;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The box that says what an item is when the pointer rests on its name in the message log (the
/// same words as the inventory and the hotbar give), drawn on the 640 x 400 grid above the line.
public static class MessageItemTip
{
    // The message lines in the text panel: the log's first pixel across, the baseline of its newest
    // line and the gap between lines (in the panel's own lines), and the grid row of the panel's top
    // (143 lines of 2 rows, with the message box raised 4 lines).
    private const int TEXT_LEFT = 16;
    private const int NEWEST_BASELINE = 42;
    private const int LINE_HEIGHT = 10;
    private const int PANEL_TOP_ROW = 278;
    private const int ROWS_PER_LINE = 2;

    // A letter reaches this many lines above its baseline and the underline this many below it.
    private const int ASCENT_LINES = 7;
    private const int DESCENT_LINES = 3;

    private const int PADDING = 6;
    private const int GAP = 4;
    private const int LINE_ADVANCE = 11;
    private const int GLYPH_ASCENT = 7;
    private const byte DARKNESS = 215;

    /// The box for the item name under the pointer (a place of the grid); empty when there is none.
    /// advanceOf says how far across each character of the log's font reaches.
    public static Overlay Of(Data.Game game, MessageHistory history, Func<byte, int> advanceOf, (int X, int Y) pointer)
    {
        Overlay overlay = new();

        if (SlotAt(game, history, advanceOf, pointer) is { } hit)
        {
            Add(overlay, game, hit.Slot, hit.Left, hit.Top);
        }

        return overlay;
    }

    /// The item whose name is under the place of the grid, with where its name starts and its top.
    public static (int Slot, int Left, int Top)? SlotAt(Data.Game game, MessageHistory history, Func<byte, int> advanceOf, (int X, int Y) place)
    {
        IReadOnlyList<byte[]> window = history.Window();

        for (int index = 0; index < window.Count; index++)
        {
            int baseline = NEWEST_BASELINE - (window.Count - 1 - index) * LINE_HEIGHT;
            int top = PANEL_TOP_ROW + (baseline - ASCENT_LINES) * ROWS_PER_LINE;
            int bottom = PANEL_TOP_ROW + (baseline + DESCENT_LINES) * ROWS_PER_LINE;

            if (place.Y < top || place.Y >= bottom)
            {
                continue;
            }

            foreach (ItemWord word in ItemWords.Find(game, window[index]))
            {
                int left = TEXT_LEFT + WidthOf(window[index].AsSpan(0, word.Start), advanceOf);

                if (place.X >= left && place.X < left + WidthOf(window[index].AsSpan(word.Start, word.Length), advanceOf))
                {
                    return (word.Slot, left, top);
                }
            }
        }

        return null;
    }

    private static int WidthOf(ReadOnlySpan<byte> text, Func<byte, int> advanceOf)
    {
        int width = 0;

        foreach (byte character in text)
        {
            width += advanceOf(character);
        }

        return width;
    }

    // The box above the line: the name, then what the item does when it has something to say.
    private static void Add(Overlay overlay, Data.Game game, int slot, int left, int top)
    {
        string name = InfernoSkull.NameOf(game, slot);
        string detail = ItemStats.For(slot);
        int lines = detail.Length > 0 ? 2 : 1;
        int width = Math.Max(Overlay.TextWidth(name, 1), Overlay.TextWidth(detail, 1)) + 2 * PADDING;
        int height = 2 * PADDING + (lines - 1) * LINE_ADVANCE + GLYPH_ASCENT;
        int x = Math.Clamp(left, 0, Math.Max(0, Overlay.WIDTH - width));
        int y = top - GAP - height;

        overlay.Shades.Add(new Shade(x, y, width, height, DARKNESS));
        MenuStyle.AddText(overlay, name, (x + PADDING, y + PADDING + GLYPH_ASCENT), MenuStyle.TITLE_COLOR, 1);

        if (detail.Length > 0)
        {
            MenuStyle.AddText(overlay, detail, (x + PADDING, y + PADDING + LINE_ADVANCE + GLYPH_ASCENT), MenuStyle.DESCRIPTION_COLOR, 1);
        }
    }
}
