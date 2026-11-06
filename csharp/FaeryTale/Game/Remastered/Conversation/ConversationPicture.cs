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
    private static void AddRow(Overlay overlay, Row row, int index, bool hovered)
    {
        bool lit = hovered && row.Enabled;
        int width = BubbleLayout.WidthOf(row);
        int left = BubbleLayout.ButtonLeft;
        int top = BubbleLayout.ButtonTop(index);

        overlay.Images.Add(BoardFor(width, row, lit) with { X = left, Y = top });

        int textLeft = left + (width - Overlay.TextWidth(row.Label, TEXT_SCALE)) / 2;
        int baseline = top + (BubbleLayout.BUTTON_HEIGHT - Overlay.GLYPH_SIZE) / 2 + GLYPH_ASCENT;
        uint color = !row.Enabled ? GREY_TEXT : lit ? MenuStyle.HOVER_COLOR : MenuStyle.TITLE_COLOR;

        MenuStyle.AddText(overlay, row.Label, (textLeft, baseline), color, TEXT_SCALE);
    }

    // The board for a row, made once for each look.
    private static Image BoardFor(int width, Row row, bool lit)
    {
        (int, RowKind, bool, bool) key = (width, row.Kind, lit, row.Enabled);

        if (!boards.TryGetValue(key, out Image? board))
        {
            Image plain = MenuStyle.Board((0, 0, width, BubbleLayout.BUTTON_HEIGHT), BOARD_SEED + (uint)width)[lit ? 1 : 0];

            board = row.Kind != RowKind.Item ? plain : Tint(plain, row.Enabled ? ITEM_TINT : GREY_TINT, !row.Enabled);
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
            float red = (pixel >> 16) & 0xff;
            float green = (pixel >> 8) & 0xff;
            float blue = pixel & 0xff;

            if (grey)
            {
                red = green = blue = (red + green + blue) / 3f;
            }

            pixels[index] = (pixel & 0xff00_0000) | Channel(red * tint.R) << 16 | Channel(green * tint.G) << 8 | Channel(blue * tint.B);
        }

        return picture with { Pixels = pixels };
    }

    private static uint Channel(float value) => (uint)Math.Clamp(value, 0f, 255f);
}
