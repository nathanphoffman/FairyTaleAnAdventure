using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The bubble of a conversation as an overlay: the face, with the choices on wooden boards to the
/// right of it. The things for sale are on boards of another wood (a cool green-grey), and those
/// the hero cannot pay for are grey. The overlay is in units from the bubble's top left corner
/// (see BubbleLayout).
public static class ConversationPicture
{
    private const uint BOARD_SEED = 0x7a1c;
    private const int TEXT_SCALE = 1;

    // Where each colour byte sits in a 0xAARRGGBB pixel, and the mask for one byte.
    private const int RED_SHIFT = 16;
    private const int GREEN_SHIFT = 8;
    private const uint CHANNEL_MASK = 0xff;
    private const float MAX_CHANNEL = 255f;
    private const uint ALPHA_MASK = 0xff00_0000;
    private const float AVERAGE_OF_CHANNELS = 3f;

    /// The baseline of a line of text is this far below the top of its glyph box.
    private const int GLYPH_ASCENT = 7;

    // How each colour of the boards of the things for sale is changed: the wood's own colours times
    // these. The greyed boards get one average brightness in all three colours first.
    private static readonly (float R, float G, float B) ITEM_TINT = (0.62f, 0.95f, 0.85f);
    private static readonly (float R, float G, float B) GREY_TINT = (0.55f, 0.55f, 0.58f);
    private const uint GREY_TEXT = 0xff8c_8c8c;

    private static readonly Dictionary<(int Width, RowKind Kind, bool Lit, bool Enabled), Image> boards = [];

    /// The face with its tail, then each row as a board, lit when hovered (null for none).
    public static Overlay Draw(Image face, IReadOnlyList<Row> rows, int? hovered)
    {
        Overlay overlay = new();

        overlay.Images.Add(FacePicture.WithTail(face));

        for (int index = 0; index < rows.Count; index++)
        {
            AddRow(overlay, rows[index], index, index == hovered);
        }

        return overlay;
    }

    // A board and its words, centred on it (a greyed board is never lit).
    private static void AddRow(Overlay overlay, Row rowIndex, int index, bool hovered)
    {
        bool lit = hovered && rowIndex.Enabled;
        int width = BubbleLayout.WidthOf(rowIndex);
        int left = BubbleLayout.ButtonLeft;
        int topEdge = BubbleLayout.ButtonTop(index);

        overlay.Images.Add(BoardFor(width, rowIndex, lit) with { X = left, Y = topEdge });

        int textLeft = left + (width - Overlay.TextWidth(rowIndex.Label, TEXT_SCALE)) / 2;
        int baseline = topEdge + (BubbleLayout.BUTTON_HEIGHT - Overlay.GLYPH_SIZE) / 2 + GLYPH_ASCENT;
        uint color = !rowIndex.Enabled ? GREY_TEXT : lit ? MenuStyle.HOVER_COLOR : MenuStyle.TITLE_COLOR;  // disabled is grey, else hover or title color

        MenuStyle.AddText(overlay, rowIndex.Label, (textLeft, baseline), color, TEXT_SCALE);
    }

    // The board for a row, made once for each look.
    private static Image BoardFor(int width, Row rowIndex, bool lit)
    {
        (int, RowKind, bool, bool) key = (width, rowIndex.Kind, lit, rowIndex.Enabled);

        if (!boards.TryGetValue(key, out Image? board))
        {
            Image plain = MenuStyle.Board((0, 0, width, BubbleLayout.BUTTON_HEIGHT), BOARD_SEED + (uint)width)[lit ? 1 : 0];

            board = rowIndex.Kind != RowKind.Item ? plain : Tint(plain, rowIndex.Enabled ? ITEM_TINT : GREY_TINT, !rowIndex.Enabled);  // plain if not an item, else grey or tinted copy
            boards[key] = board;
        }

        return board;
    }

    // The picture with every colour multiplied (and, for grey, first made one brightness). The
    // alpha is kept, so the soft shadow under a board stays.
    private static Image Tint(Image picture, (float R, float G, float B) tint, bool grey)
    {
        uint[] pixels = new uint[picture.Pixels.Length];

        for (int index = 0; index < pixels.Length; index++)
        {
            uint pixel = picture.Pixels[index];
            float red = (pixel >> RED_SHIFT) & CHANNEL_MASK;  // red is bits 16-23: shift down, mask to 0-255
            float green = (pixel >> GREEN_SHIFT) & CHANNEL_MASK;  // green is bits 8-15: shift down, mask to 0-255
            float blue = pixel & CHANNEL_MASK;

            if (grey)
            {
                red = green = blue = (red + green + blue) / AVERAGE_OF_CHANNELS;
            }

            pixels[index] = (pixel & ALPHA_MASK) | Channel(red * tint.R) << RED_SHIFT | Channel(green * tint.G) << GREEN_SHIFT | Channel(blue * tint.B);  // keep alpha, then put each tinted channel back in its byte
        }

        return picture with { Pixels = pixels };
    }

    private static uint Channel(float value) => (uint)Math.Clamp(value, 0f, MAX_CHANNEL);
}
