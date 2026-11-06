using FaeryTale.Platform;

namespace FaeryTale.Game.Remastered;

/// One square of the hotbar: which item of the pack it shows, whether it is a spell (else a weapon), and
/// where its top left corner is in the window (pixels).
public readonly record struct HotbarSlot(int Item, bool IsSpell, int X, int Y);

/// Where the hotbar goes for a window of some size: the size of a square, the board of wood under the text
/// panel that the bars grow out of, and the middle line the squares sit on.
public sealed record HotbarGeometry(int SlotSize, int Scale, int BoardLeft, int BoardRight, int BoardTop, int WindowWidth, int WindowHeight, int CenterY);

/// The hotbar (Remastered and Reimagined): the wood under the text panel grows out to the left and the
/// right, the weapons the hero carries on the left and the magic stones on the right, each in a square
/// with a thick black border. The drawing and the clicks both ask here, so a square is where it is
/// drawn. It is all in window pixels, and needs the room beside the panel; a small window has none.
public static class HotbarLayout
{
    /// Room between squares and round them on the wood, and the sizes a square can be.
    public const int GAP = 4;
    public const int PAD = 6;
    public const int MIN_SLOT = 28;
    public const int MAX_SLOT = 60;

    /// The most spells a hero can carry, and how many squares go across in a row on each side (so the
    /// squares can be big): the stones are in two rows of up to four, the weapons two rows of up to three.
    public const int SPELL_SLOTS = 7;
    public const int SPELL_COLUMNS = 4;
    public const int WEAPON_COLUMNS = 3;

    // How the text panel sits in the window (as NativeComposer lays it out: the 640 x 400 grid is scaled
    // by a whole number, centred across and at the bottom), and where the middle of the menu is.
    private const int GRID_WIDTH = 640;
    private const int GRID_HEIGHT = 400;
    private const int ROWS_PER_LINE = 2;
    private const int MENU_MIDDLE_LINES = 24;

    /// The geometry for a window of this size, or null when there is not room beside the panel.
    public static HotbarGeometry? For(int width, int height)
    {
        int scale = Math.Max(1, Math.Min(width / GRID_WIDTH, height / GRID_HEIGHT));
        int gridLeft = (width - GRID_WIDTH * scale) / 2;
        int gridTop = height - GRID_HEIGHT * scale;
        int boardLeft = gridLeft - PanelLayout.PAD_SIDE * scale;
        int boardRight = gridLeft + (GRID_WIDTH + PanelLayout.COMPASS_SHIFT + PanelLayout.PAD_SIDE) * scale;
        int room = Math.Min(boardLeft, width - boardRight);
        int slot = Math.Min(MAX_SLOT, (room - GAP - 2 * PAD) / SPELL_COLUMNS - GAP);

        if (slot < MIN_SLOT)
        {
            return null;
        }

        int boardTop = gridTop + (PanelLayout.PANEL_FIRST_LINE - PanelLayout.TOP_PAD_LINES) * ROWS_PER_LINE * scale;
        int centerY = gridTop + (PanelLayout.PANEL_FIRST_LINE + MENU_MIDDLE_LINES) * ROWS_PER_LINE * scale;

        return new HotbarGeometry(slot, scale, boardLeft, boardRight, boardTop, width, height, centerY);
    }

    /// How wide the wood is for a bar of some number of squares laid out in rows of columns (0 for none).
    public static int StripWidth(HotbarGeometry geometry, int count, int columns) =>
        count == 0 ? 0 : Math.Min(count, columns) * (geometry.SlotSize + GAP) + GAP + 2 * PAD;

    /// Every square: the weapons in rows from the left edge of the wood on the left of the panel, the
    /// spells in rows from the left edge of the wood on its right, the rows one above the other round
    /// the middle line.
    public static List<HotbarSlot> Slots(HotbarGeometry geometry, IReadOnlyList<int> weapons, IReadOnlyList<int> spells)
    {
        List<HotbarSlot> slots = [];
        int weaponsLeft = geometry.BoardLeft - StripWidth(geometry, weapons.Count, WEAPON_COLUMNS) + PAD + GAP;
        int spellsLeft = geometry.BoardRight + PAD + GAP;

        AddGroup(slots, geometry, weapons, false, (weaponsLeft, WEAPON_COLUMNS));
        AddGroup(slots, geometry, spells, true, (spellsLeft, SPELL_COLUMNS));

        return slots;
    }

    // One group of squares in rows of columns, starting at a left edge and centred up and down.
    private static void AddGroup(List<HotbarSlot> slots, HotbarGeometry geometry, IReadOnlyList<int> items, bool isSpell, (int Left, int Columns) place)
    {
        int rows = (items.Count + place.Columns - 1) / place.Columns;
        int top = geometry.CenterY - (rows * geometry.SlotSize + (rows - 1) * GAP) / 2;

        for (int index = 0; index < items.Count; index++)
        {
            int x = place.Left + index % place.Columns * (geometry.SlotSize + GAP);
            int y = top + index / place.Columns * (geometry.SlotSize + GAP);

            slots.Add(new HotbarSlot(items[index], isSpell, x, y));
        }
    }

    /// The square a window pixel is on, if any.
    public static HotbarSlot? SlotAt(HotbarGeometry geometry, IReadOnlyList<HotbarSlot> slots, int pixelX, int pixelY)
    {
        foreach (HotbarSlot slot in slots)
        {
            if (pixelX >= slot.X && pixelX < slot.X + geometry.SlotSize && pixelY >= slot.Y && pixelY < slot.Y + geometry.SlotSize)
            {
                return slot;
            }
        }

        return null;
    }
}
