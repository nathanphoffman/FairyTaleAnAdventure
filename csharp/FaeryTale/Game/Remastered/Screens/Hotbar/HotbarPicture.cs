using FaeryTale.Game.Reimagined;
using FaeryTale.Game.Data;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The squares of the hotbar as an overlay in window pixels: each in a thick black border over stone,
/// the weapon in hand ringed in gold, the picture of the item (the inventory's, fitted to the square),
/// how many of a stone there are and the key that casts it, and, for the square under the pointer, a
/// box above the bar that says what it is. The squares are made once for each size.
public static class HotbarPicture
{
    private const uint STONE_SEED = 0x3d11;

    // The black border of a square and the gold ring just inside it (for the weapon in hand).
    private const int BORDER = 3;
    private const int RING = 2;
    private const uint BLACK = 0xff00_0000;
    private const uint GOLD = 0xffe0_b040;

    // The dotted white line of a queued square: this thick, in dashes this long.
    private const uint WHITE = 0xffff_ffff;
    private const int QUEUED_THICKNESS = 2;
    private const int DASH_LENGTH = 3;

    // Room between a picture and the border of its square.
    private const int ICON_MARGIN = 3;

    private const int GLYPH_ASCENT = 7;
    private const int LABEL_INSET = 2;
    private const int NAME_SCALE = 2;
    private const int TIP_PADDING = 8;
    private const int TIP_GAP = 8;
    private const int TIP_LINE_HEIGHT = 18;
    private const byte TIP_DARKNESS = 215;

    private static readonly Dictionary<(int Size, bool Lit, bool Equipped), Image> squares = [];
    private static readonly Dictionary<int, Image> queuedBorders = [];

    /// The overlay for the slots; pointer is where the mouse is in the window.
    public static Overlay Of(Data.Game game, HotbarGeometry geometry, IReadOnlyList<HotbarSlot> slots, Func<int, SpriteImage?> pictureOf, (int X, int Y) pointer)
    {
        Overlay overlay = new();
        HotbarSlot? hovered = HotbarLayout.SlotAt(geometry, slots, pointer.X, pointer.Y);

        foreach (HotbarSlot slot in slots)
        {
            AddSquare(overlay, game, geometry, slot, pictureOf, hovered is { } over && over.Item == slot.Item);
        }

        if (hovered is { } tip)
        {
            AddTip(overlay, game, geometry, tip);
        }

        return overlay;
    }

    // One square with its picture, count and key.
    private static void AddSquare(Overlay overlay, Data.Game game, HotbarGeometry geometry, HotbarSlot slot, Func<int, SpriteImage?> pictureOf, bool lit)
    {
        int size = geometry.SlotSize;
        bool equipped = !slot.IsSpell && InventoryActions.IsEquipped(game, slot.Item);

        overlay.Images.Add(Square(size, lit, equipped) with { X = slot.X, Y = slot.Y });

        if (QueuedUse.Contains(slot.Item))
        {
            overlay.Images.Add(QueuedBorder(size) with { X = slot.X, Y = slot.Y });
        }

        if (ItemIcons.Fitted(slot.Item, pictureOf, size - 2 * (BORDER + ICON_MARGIN)) is { } icon)
        {
            overlay.Images.Add(icon with { X = slot.X + (size - icon.Width) / 2, Y = slot.Y + (size - icon.Height) / 2 });
        }

        if (slot.IsSpell)
        {
            AddSpellLabels(overlay, game, geometry, slot);
        }
    }

    // The key that casts a stone in the top left corner of its square, and how many there are at the bottom right.
    private static void AddSpellLabels(Overlay overlay, Data.Game game, HotbarGeometry geometry, HotbarSlot slot)
    {
        int count = InventoryActions.CountOf(game, slot.Item);

        MenuStyle.AddText(overlay, Hotbar.KeyOf(slot.Item), (slot.X + BORDER + LABEL_INSET, slot.Y + BORDER + LABEL_INSET + GLYPH_ASCENT), MenuStyle.HEADING_COLOR, 1);

        if (count > 1)
        {
            string text = $"x{count}";

            MenuStyle.AddText(overlay, text, (slot.X + geometry.SlotSize - BORDER - LABEL_INSET - Overlay.TextWidth(text, 1), slot.Y + geometry.SlotSize - BORDER - LABEL_INSET - 1), MenuStyle.TITLE_COLOR, 1);
        }
    }

    // A box above the bar with the name of the square under the pointer and what it does.
    private static void AddTip(Overlay overlay, Data.Game game, HotbarGeometry geometry, HotbarSlot slot)
    {
        string name = InfernoSkull.NameOf(game, slot.Item);
        string detail = slot.IsSpell ? $"{Hotbar.KeyOf(slot.Item)} or click to cast. {ItemStats.For(slot.Item)}" : $"Click to equip. {ItemStats.For(slot.Item)}";
        int width = Math.Max(Overlay.TextWidth(name, NAME_SCALE), Overlay.TextWidth(detail, 1)) + 2 * TIP_PADDING;
        int left = Math.Clamp(slot.X + geometry.SlotSize / 2 - width / 2, 0, Math.Max(0, geometry.WindowWidth - width));
        int top = geometry.BoardTop - 2 * TIP_PADDING - TIP_LINE_HEIGHT - TIP_GAP;

        overlay.Shades.Add(new Shade(left, top, width, 2 * TIP_PADDING + TIP_LINE_HEIGHT + GLYPH_ASCENT, TIP_DARKNESS));
        MenuStyle.AddText(overlay, name, (left + TIP_PADDING, top + TIP_PADDING + GLYPH_ASCENT * NAME_SCALE), MenuStyle.TITLE_COLOR, NAME_SCALE);
        MenuStyle.AddText(overlay, detail, (left + TIP_PADDING, top + TIP_PADDING + TIP_LINE_HEIGHT + GLYPH_ASCENT), MenuStyle.DESCRIPTION_COLOR, 1);
    }

    // The white dotted line round a square that is queued for when the game goes on: dashes along the
    // black border (a pixel is white where its diagonal stripe, x + y, falls in an even block).
    private static Image QueuedBorder(int size)
    {
        if (!queuedBorders.TryGetValue(size, out Image? border))
        {
            uint[] pixels = new uint[size * size];

            for (int y = 0; y < size; y++)
            {
                for (int x = 0; x < size; x++)
                {
                    int depth = Math.Min(Math.Min(x, size - 1 - x), Math.Min(y, size - 1 - y));

                    pixels[y * size + x] = depth < QUEUED_THICKNESS && (x + y) / DASH_LENGTH % 2 == 0 ? WHITE : 0;
                }
            }

            border = new Image(0, 0, size, size, pixels);
            queuedBorders[size] = border;
        }

        return border;
    }

    // A square: stone inside, a thick black border, and a gold ring for the weapon in hand.
    private static Image Square(int size, bool lit, bool equipped)
    {
        if (!squares.TryGetValue((size, lit, equipped), out Image? square))
        {
            uint[] stone = lit ? Stone.LitSlab(size, size, STONE_SEED) : Stone.Slab(size, size, STONE_SEED);
            uint[] pixels = new uint[size * size];

            for (int y = 0; y < size; y++)
            {
                for (int x = 0; x < size; x++)
                {
                    int depth = Math.Min(Math.Min(x, size - 1 - x), Math.Min(y, size - 1 - y));

                    pixels[y * size + x] = depth < BORDER ? BLACK : equipped && depth < BORDER + RING ? GOLD : stone[y * size + x];
                }
            }

            square = new Image(0, 0, size, size, pixels);
            squares[(size, lit, equipped)] = square;
        }

        return square;
    }
}
