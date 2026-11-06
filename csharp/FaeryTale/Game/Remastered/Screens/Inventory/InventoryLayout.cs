namespace FaeryTale.Game.Remastered;

/// An item's square on the inventory screen (a place on the 640 x 400 overlay grid).
public readonly record struct InventoryCell(InventoryItem Item, int X, int Y);

/// A section's title and where it is written.
public readonly record struct InventoryTitle(string Text, int X, int Y);

/// Where everything is on the inventory screen: the stone panel above the text panel, two columns of
/// sections with a square for each item, a close button at the top right, and a line under the
/// squares that says what the square under the pointer is. The drawing and the clicks both ask
/// here, so a square is where it is drawn.
public static class InventoryLayout
{
    // The panel reaches a little above the overlay grid (the world is there), down to the text panel.
    public const int PANEL_LEFT = 10;
    public const int PANEL_TOP = -16;
    public const int PANEL_WIDTH = 620;
    public const int PANEL_HEIGHT = 300;

    public const int CELL_WIDTH = 56;
    public const int CELL_HEIGHT = 44;

    // The close button (an X) in the corner of the title bar.
    public const int CLOSE_WIDTH = 20;
    public const int CLOSE_HEIGHT = 18;
    private const int CLOSE_INSET = 26;
    private const int CLOSE_TOP_INSET = 4;

    /// The line under the squares, for what the pointer is on.
    public const int DETAIL_LEFT = 22;
    public const int DETAIL_TOP = 255;
    public const int DETAIL_WIDTH = 590;
    public const int DETAIL_HEIGHT = 26;

    // Squares are in rows of five, a little apart.
    private const int PER_ROW = 5;
    private const int PITCH_X = 58;
    private const int PITCH_Y = 46;

    // The two columns, where the first section starts, and the room a title and the gap between
    // sections take.
    private const int LEFT_COLUMN_X = 22;
    private const int RIGHT_COLUMN_X = 322;
    private const int FIRST_SECTION_Y = 20;
    private const int TITLE_HEIGHT = 12;
    private const int SECTION_GAP = 4;

    private static readonly (List<InventoryCell> Cells, List<InventoryTitle> Titles) placed = Place();

    /// Every item's square.
    public static IReadOnlyList<InventoryCell> CELLS => placed.Cells;

    /// Every section's title.
    public static IReadOnlyList<InventoryTitle> TITLES => placed.Titles;

    /// The close button's top left corner.
    public static (int X, int Y) CloseCorner => (PANEL_LEFT + PANEL_WIDTH - CLOSE_INSET, PANEL_TOP + CLOSE_TOP_INSET);

    /// Whether a place on the grid is on the panel.
    public static bool PanelContains((int X, int Y) place) =>
        place.X >= PANEL_LEFT && place.X < PANEL_LEFT + PANEL_WIDTH && place.Y >= PANEL_TOP && place.Y < PANEL_TOP + PANEL_HEIGHT;

    /// Whether a place on the grid is on the close button.
    public static bool CloseContains((int X, int Y) place) =>
        place.X >= CloseCorner.X && place.X < CloseCorner.X + CLOSE_WIDTH && place.Y >= CloseCorner.Y && place.Y < CloseCorner.Y + CLOSE_HEIGHT;

    /// The square a place on the grid is on, if any.
    public static InventoryCell? CellAt((int X, int Y) place)
    {
        foreach (InventoryCell cell in CELLS)
        {
            if (place.X >= cell.X && place.X < cell.X + CELL_WIDTH && place.Y >= cell.Y && place.Y < cell.Y + CELL_HEIGHT)
            {
                return cell;
            }
        }

        return null;
    }

    // Lays out both columns, section after section, a title and then the squares in rows of five.
    private static (List<InventoryCell> Cells, List<InventoryTitle> Titles) Place()
    {
        List<InventoryCell> cells = [];
        List<InventoryTitle> titles = [];

        PlaceColumn(InventorySections.LEFT, LEFT_COLUMN_X, cells, titles);
        PlaceColumn(InventorySections.RIGHT, RIGHT_COLUMN_X, cells, titles);

        return (cells, titles);
    }

    private static void PlaceColumn(InventorySection[] sections, int x, List<InventoryCell> cells, List<InventoryTitle> titles)
    {
        int y = FIRST_SECTION_Y;

        foreach (InventorySection section in sections)
        {
            titles.Add(new InventoryTitle(section.Title, x, y));
            y += TITLE_HEIGHT;

            for (int index = 0; index < section.Items.Length; index++)
            {
                cells.Add(new InventoryCell(section.Items[index], x + index % PER_ROW * PITCH_X, y + index / PER_ROW * PITCH_Y));
            }

            y += (section.Items.Length + PER_ROW - 1) / PER_ROW * PITCH_Y + SECTION_GAP;
        }
    }
}
