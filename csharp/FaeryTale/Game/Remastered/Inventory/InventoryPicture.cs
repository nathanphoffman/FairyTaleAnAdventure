using FaeryTale.Game.Data;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The inventory screen as an overlay on the 640 x 400 grid: a slab of stone with the sections of
/// items on it, each item in a square edged like a checkerboard, an item that is not carried dim, the
/// weapon in hand edged in gold, the close button, and a line that says what the square under the
/// pointer is. The panel and the squares are made once.
public static class InventoryPicture
{
    private const uint PANEL_SEED = 0x6a3f;
    private const uint CELL_SEED = 0x2c91;
    private const uint CLOSE_SEED = 0x4e07;

    // A square's checkerboard edge: each little square this big, in two greys, and the gold ring of
    // the weapon in hand just inside it.
    private const int EDGE = 4;
    private const int RING = 2;
    private const uint EDGE_LIGHT = 0xff9a_9aa2;
    private const uint EDGE_DARK = 0xff5a_5a62;
    private const uint GOLD = 0xffe0_b040;

    // How much of its colour an item not carried keeps (out of 100).
    private const int DIM_PERCENT = 30;

    private const int TITLE_SCALE = 2;
    private const int TEXT_SCALE = 1;
    private const int GLYPH_ASCENT = 7;
    private const int TITLE_X = 28;
    private const int TITLE_BASELINE = 14;
    private const int SECTION_TITLE_BASELINE = 9;
    private const int COUNT_INSET = 3;
    private const uint DETAIL_BACKGROUND = 0xff2a_2a30;

    private static Image? panel;
    private static Image? close;
    private static Image? litClose;
    private static Image? detailBar;
    private static readonly Dictionary<(bool Lit, bool Equipped), Image> cells = [];
    private static readonly Dictionary<int, Image> dimmed = [];

    /// The screen for what the hero carries; pictureOf gives the picture of an object by number, and
    /// pointer is where the mouse is on the grid.
    public static Overlay Of(Data.Game game, Func<int, SpriteImage?> pictureOf, (int X, int Y) pointer)
    {
        Overlay overlay = new();
        InventoryCell? hovered = InventoryLayout.CellAt(pointer);

        overlay.Images.Add(Panel());
        AddHeader(overlay, game, InventoryLayout.CloseContains(pointer));

        foreach (InventoryTitle title in InventoryLayout.TITLES)
        {
            MenuStyle.AddText(overlay, title.Text, (title.X, title.Y + SECTION_TITLE_BASELINE), MenuStyle.HEADING_COLOR, TEXT_SCALE);
        }

        foreach (InventoryCell cell in InventoryLayout.CELLS)
        {
            AddCell(overlay, game, pictureOf, cell, hovered is { } over && over.Item.Slot == cell.Item.Slot);
        }

        AddDetail(overlay, game, hovered);

        return overlay;
    }

    // The title, the gold the hero has, and the close button.
    private static void AddHeader(Overlay overlay, Data.Game game, bool closeLit)
    {
        MenuStyle.AddText(overlay, "Inventory", (TITLE_X, InventoryLayout.PANEL_TOP + TITLE_BASELINE + GLYPH_ASCENT), MenuStyle.TITLE_COLOR, TITLE_SCALE);

        string gold = $"Gold {game.Hero.Wealth}";
        int goldLeft = InventoryLayout.CloseCorner.X - 24 - Overlay.TextWidth(gold, TEXT_SCALE);

        MenuStyle.AddText(overlay, gold, (goldLeft, InventoryLayout.PANEL_TOP + TITLE_BASELINE + GLYPH_ASCENT), MenuStyle.HEADING_COLOR, TEXT_SCALE);

        (int x, int y) = InventoryLayout.CloseCorner;
        Image button = closeLit ? LitClose() : Close();

        overlay.Images.Add(button with { X = x, Y = y });
        MenuStyle.AddText(overlay, "X", (x + (InventoryLayout.CLOSE_WIDTH - Overlay.GLYPH_SIZE) / 2, y + (InventoryLayout.CLOSE_HEIGHT - Overlay.GLYPH_SIZE) / 2 + GLYPH_ASCENT), closeLit ? MenuStyle.HOVER_COLOR : MenuStyle.TITLE_COLOR, TEXT_SCALE);
    }

    // One item's square, with its picture centred in it and, when there are several, how many.
    private static void AddCell(Overlay overlay, Data.Game game, Func<int, SpriteImage?> pictureOf, InventoryCell cell, bool lit)
    {
        int count = InventoryActions.CountOf(game, cell.Item.Slot);
        bool equipped = InventoryActions.IsEquipped(game, cell.Item.Slot);

        overlay.Images.Add(CellPicture(lit, equipped) with { X = cell.X, Y = cell.Y });

        if (ItemIcons.Of(cell.Item.Slot, pictureOf) is { } icon)
        {
            Image shown = count > 0 ? icon : Dim(cell.Item.Slot, icon);

            overlay.Images.Add(shown with { X = cell.X + (InventoryLayout.CELL_WIDTH - icon.Width) / 2, Y = cell.Y + (InventoryLayout.CELL_HEIGHT - icon.Height) / 2 });
        }

        if (count > 1)
        {
            string text = $"x{count}";

            MenuStyle.AddText(overlay, text, (cell.X + InventoryLayout.CELL_WIDTH - COUNT_INSET - Overlay.TextWidth(text, TEXT_SCALE), cell.Y + InventoryLayout.CELL_HEIGHT - COUNT_INSET - 1), MenuStyle.TITLE_COLOR, TEXT_SCALE);
        }
    }

    // The line under the squares: the name, how many, whether it is in hand, and what a click does.
    private static void AddDetail(Overlay overlay, Data.Game game, InventoryCell? hovered)
    {
        overlay.Images.Add(DetailBar());

        string text = hovered is { } cell ? DetailText(game, cell.Item) : "Move the pointer over an item";

        MenuStyle.AddText(overlay, text, (InventoryLayout.DETAIL_LEFT + 8, InventoryLayout.DETAIL_TOP + (InventoryLayout.DETAIL_HEIGHT - Overlay.GLYPH_SIZE) / 2 + GLYPH_ASCENT), MenuStyle.DESCRIPTION_COLOR, TEXT_SCALE);
    }

    private static string DetailText(Data.Game game, InventoryItem item)
    {
        int count = InventoryActions.CountOf(game, item.Slot);
        string name = Tables.ItemList[item.Slot].Name;

        if (count == 0)
        {
            return $"{name} - you do not have one";
        }

        string state = InventoryActions.IsEquipped(game, item.Slot) ? "  (in hand)" : count > 1 ? $"  x{count}" : "";

        return $"{name}{state}   {InventoryActions.HintFor(item)}";
    }

    private static Image Panel() =>
        panel ??= new Image(InventoryLayout.PANEL_LEFT, InventoryLayout.PANEL_TOP, InventoryLayout.PANEL_WIDTH, InventoryLayout.PANEL_HEIGHT, Stone.Slab(InventoryLayout.PANEL_WIDTH, InventoryLayout.PANEL_HEIGHT, PANEL_SEED, false));

    private static Image Close() =>
        close ??= new Image(0, 0, InventoryLayout.CLOSE_WIDTH, InventoryLayout.CLOSE_HEIGHT, Stone.Slab(InventoryLayout.CLOSE_WIDTH, InventoryLayout.CLOSE_HEIGHT, CLOSE_SEED, false));

    private static Image LitClose() =>
        litClose ??= new Image(0, 0, InventoryLayout.CLOSE_WIDTH, InventoryLayout.CLOSE_HEIGHT, Stone.Slab(InventoryLayout.CLOSE_WIDTH, InventoryLayout.CLOSE_HEIGHT, CLOSE_SEED, true));

    // A dark recess for the line of words.
    private static Image DetailBar()
    {
        if (detailBar is null)
        {
            uint[] pixels = new uint[InventoryLayout.DETAIL_WIDTH * InventoryLayout.DETAIL_HEIGHT];

            Array.Fill(pixels, DETAIL_BACKGROUND);
            detailBar = new Image(InventoryLayout.DETAIL_LEFT, InventoryLayout.DETAIL_TOP, InventoryLayout.DETAIL_WIDTH, InventoryLayout.DETAIL_HEIGHT, pixels);
        }

        return detailBar;
    }

    // The square: stone inside, a checkerboard edge of two greys, and a gold ring for the weapon in hand.
    private static Image CellPicture(bool lit, bool equipped)
    {
        if (!cells.TryGetValue((lit, equipped), out Image? cell))
        {
            int width = InventoryLayout.CELL_WIDTH;
            int height = InventoryLayout.CELL_HEIGHT;
            uint[] stone = Stone.Slab(width, height, CELL_SEED, lit);
            uint[] pixels = new uint[width * height];

            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    pixels[y * width + x] = CellPixel(stone[y * width + x], x, y, equipped);
                }
            }

            cell = new Image(0, 0, width, height, pixels);
            cells[(lit, equipped)] = cell;
        }

        return cell;
    }

    // One pixel of a square: the checkerboard edge, the gold ring just inside it, else the stone.
    private static uint CellPixel(uint stone, int x, int y, bool equipped)
    {
        int depth = Math.Min(Math.Min(x, InventoryLayout.CELL_WIDTH - 1 - x), Math.Min(y, InventoryLayout.CELL_HEIGHT - 1 - y));

        if (depth < EDGE)
        {
            return (x / EDGE + y / EDGE) % 2 == 0 ? EDGE_LIGHT : EDGE_DARK;
        }

        return equipped && depth < EDGE + RING ? GOLD : stone;
    }

    // The picture dimmed, for an item the hero does not have.
    private static Image Dim(int slot, Image icon)
    {
        if (!dimmed.TryGetValue(slot, out Image? dim))
        {
            dim = icon with { Pixels = icon.Pixels.Select(DimPixel).ToArray() };
            dimmed[slot] = dim;
        }

        return dim;
    }

    // A colour with each of red, green and blue cut down (see-through pixels stay see-through).
    private static uint DimPixel(uint pixel)
    {
        uint Part(int shift) => ((pixel >> shift) & 0xff) * DIM_PERCENT / 100 << shift;

        return (pixel & 0xff00_0000) | Part(16) | Part(8) | Part(0);
    }
}
