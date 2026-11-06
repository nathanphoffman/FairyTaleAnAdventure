using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The red squares round the classic buttons in use. The buttons are in two columns of boxes in the
/// text panel; where they are on the 640 x 400 overlay grid is fixed by how the panel is drawn.
public static class PanelButtonOutline
{
    // The boxes: 48 wide and 16 tall, in two columns, one every 18 rows, the first at row 290.
    private const int LEFT_COLUMN_X = 430;
    private const int COLUMN_SPACING = 52;
    private const int FIRST_ROW_Y = 290;
    private const int ROW_SPACING = 18;
    private const int BOX_WIDTH = 48;
    private const int BOX_HEIGHT = 16;

    // Two boxes in a row.
    private const int COLUMNS = 2;

    // The square is this far round the box and this thick.
    private const int MARGIN = 3;
    private const int THICKNESS = 2;
    private const uint RED = 0xffe0_2020;

    // The menu entries of the buttons, and the menus they are on.
    private const int TALK_ENTRY = 2;
    private const int TAKE_ENTRY = 6;
    private const int LOOK_ENTRY = 7;
    private const int ITEMS_MENU = 0;
    private const int FIRST_NON_MAIN_MENU = 8;

    /// The squares for the buttons that are in use, as an overlay on the grid (empty when none is
    /// shown on the panel now).
    public static Overlay Of(Data.Game game, PanelButtons lit)
    {
        Overlay overlay = new();

        AddIf(overlay, game, lit.HasFlag(PanelButtons.Talk), TALK_ENTRY, game.Ui.CurrentMenu < FIRST_NON_MAIN_MENU);
        AddIf(overlay, game, lit.HasFlag(PanelButtons.Take), TAKE_ENTRY, game.Ui.CurrentMenu == ITEMS_MENU);
        AddIf(overlay, game, lit.HasFlag(PanelButtons.Look), LOOK_ENTRY, game.Ui.CurrentMenu == ITEMS_MENU);

        return overlay;
    }

    // Adds the square for a menu entry when it is wanted and the entry is on the panel now.
    private static void AddIf(Overlay overlay, Data.Game game, bool wanted, int entry, bool menuShowsIt)
    {
        int slot = Array.IndexOf(game.Ui.SlotEntries, (sbyte)entry);

        if (wanted && menuShowsIt && slot >= 0)
        {
            int x = LEFT_COLUMN_X + slot % COLUMNS * COLUMN_SPACING;
            int y = FIRST_ROW_Y + slot / COLUMNS * ROW_SPACING;

            overlay.Images.Add(Square(x - MARGIN, y - MARGIN));
        }
    }

    // A hollow square picture with its top left corner at a place on the grid.
    private static Image Square(int left, int top)
    {
        int width = BOX_WIDTH + 2 * MARGIN;
        int height = BOX_HEIGHT + 2 * MARGIN;
        uint[] pixels = new uint[width * height];

        for (int y = 0; y < height; y++)
        {
            for (int x = 0; x < width; x++)
            {
                bool onRing = x < THICKNESS || y < THICKNESS || x >= width - THICKNESS || y >= height - THICKNESS;

                pixels[y * width + x] = onRing ? RED : 0;
            }
        }

        return new Image(left, top, width, height, pixels);
    }
}
