using FaeryTale.Game.Startup;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The journal as an overlay on the 640 x 400 grid: the inventory's slab of stone with the newest
/// things people said first, each wrapped to fit, as many as the slab has room for.
public static class JournalPicture
{
    private const uint PANEL_SEED = 0x7b21;
    private const uint CLOSE_SEED = 0x4e07;

    private const int TITLE_SCALE = 2;
    private const int TEXT_SCALE = 1;
    private const int GLYPH_ASCENT = 7;
    private const int TITLE_X = 28;
    private const int TITLE_BASELINE = 14;

    // The words start this far below the top of the slab, each line this far under the last, and the
    // slab has room until this far from its top.
    private const int FIRST_LINE_BELOW_TOP = 52;
    private const int LINE_HEIGHT = 11;
    private const int ENTRY_GAP = 5;
    private const int LAST_LINE_BELOW_TOP = 290;
    private const int TEXT_LEFT = 28;
    private const int TEXT_COLUMNS = 69;
    private const string CONTINUATION_INDENT = "  ";
    private const string EMPTY_TEXT = "Nobody has told you anything yet. What people say is kept here.";

    private static Image? panel;
    private static Image? close;

    /// The screen for the things heard so far.
    public static Overlay Of(string[] entries)
    {
        Overlay overlay = new();

        overlay.Images.Add(Panel());
        AddHeader(overlay, entries.Length);

        if (entries.Length == 0)
        {
            MenuStyle.AddText(overlay, EMPTY_TEXT, (TEXT_LEFT, InventoryLayout.PANEL_TOP + FIRST_LINE_BELOW_TOP), MenuStyle.DESCRIPTION_COLOR, TEXT_SCALE);

            return overlay;
        }

        AddEntries(overlay, entries);

        return overlay;
    }

    // The title, how many things are kept, and the close button.
    private static void AddHeader(Overlay overlay, int count)
    {
        int baseline = InventoryLayout.PANEL_TOP + TITLE_BASELINE + GLYPH_ASCENT;
        string kept = $"{count} kept";

        MenuStyle.AddText(overlay, "Journal", (TITLE_X, baseline), MenuStyle.TITLE_COLOR, TITLE_SCALE);
        MenuStyle.AddText(overlay, kept, (InventoryLayout.CloseCorner.X - 24 - Overlay.TextWidth(kept, TEXT_SCALE), baseline), MenuStyle.HEADING_COLOR, TEXT_SCALE);

        (int x, int y) = InventoryLayout.CloseCorner;

        overlay.Images.Add(Close() with { X = x, Y = y });
        MenuStyle.AddText(overlay, "X", (x + (InventoryLayout.CLOSE_WIDTH - Overlay.GLYPH_SIZE) / 2, y + (InventoryLayout.CLOSE_HEIGHT - Overlay.GLYPH_SIZE) / 2 + GLYPH_ASCENT), MenuStyle.TITLE_COLOR, TEXT_SCALE);
    }

    // The newest entry first, each in the lines it wraps to, until the slab is full.
    private static void AddEntries(Overlay overlay, string[] entries)
    {
        int y = InventoryLayout.PANEL_TOP + FIRST_LINE_BELOW_TOP;
        int last = InventoryLayout.PANEL_TOP + LAST_LINE_BELOW_TOP;

        for (int index = entries.Length - 1; index >= 0; index--)
        {
            List<string> lines = LaunchMenu.Wrap(entries[index], TEXT_COLUMNS);

            if (y + lines.Count * LINE_HEIGHT > last)
            {
                return;
            }

            for (int line = 0; line < lines.Count; line++)
            {
                MenuStyle.AddText(overlay, (line == 0 ? "" : CONTINUATION_INDENT) + lines[line], (TEXT_LEFT, y), index == entries.Length - 1 ? MenuStyle.TITLE_COLOR : MenuStyle.DESCRIPTION_COLOR, TEXT_SCALE);
                y += LINE_HEIGHT;
            }

            y += ENTRY_GAP;
        }
    }

    private static Image Panel() =>
        panel ??= new Image(InventoryLayout.PANEL_LEFT, InventoryLayout.PANEL_TOP, InventoryLayout.PANEL_WIDTH, InventoryLayout.PANEL_HEIGHT, Stone.Slab(InventoryLayout.PANEL_WIDTH, InventoryLayout.PANEL_HEIGHT, PANEL_SEED));

    private static Image Close() =>
        close ??= new Image(0, 0, InventoryLayout.CLOSE_WIDTH, InventoryLayout.CLOSE_HEIGHT, Stone.Slab(InventoryLayout.CLOSE_WIDTH, InventoryLayout.CLOSE_HEIGHT, CLOSE_SEED));
}
