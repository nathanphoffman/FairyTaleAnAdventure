using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The bubble of a conversation as an overlay: the face, with the three choices on wooden boards
/// to the right of it. The overlay is in units from the bubble's top left corner (see BubbleLayout).
public static class ConversationPicture
{
    private const uint BOARD_SEED = 0x7a1c;
    private const int TEXT_SCALE = 1;

    /// The baseline of a line of text is this far below the top of its glyph box.
    private const int GLYPH_ASCENT = 7;

    private static Image[]? boards;

    /// The face with its tail, then each choice as a board, lit when hovered (null for none).
    public static Overlay Draw(Image face, int? hovered)
    {
        Overlay overlay = new();

        overlay.Images.Add(FacePicture.WithTail(face));

        for (int index = 0; index < Conversation.CHOICES.Length; index++)
        {
            AddButton(overlay, index, index == hovered);
        }

        return overlay;
    }

    // A board and its label, centred on it.
    private static void AddButton(Overlay overlay, int index, bool lit)
    {
        string label = Conversation.CHOICES[index];
        int left = BubbleLayout.ButtonLeft;
        int top = BubbleLayout.ButtonTop(index);
        Image board = Boards()[lit ? 1 : 0];

        overlay.Images.Add(board with { X = left, Y = top });

        int textLeft = left + (BubbleLayout.BUTTON_WIDTH - Overlay.TextWidth(label, TEXT_SCALE)) / 2;
        int baseline = top + (BubbleLayout.BUTTON_HEIGHT - Overlay.GLYPH_SIZE) / 2 + GLYPH_ASCENT;

        MenuStyle.AddText(overlay, label, (textLeft, baseline), lit ? MenuStyle.HOVER_COLOR : MenuStyle.TITLE_COLOR, TEXT_SCALE);
    }

    // The plain and the lit board, made once.
    private static Image[] Boards() =>
        boards ??= MenuStyle.Board((0, 0, BubbleLayout.BUTTON_WIDTH, BubbleLayout.BUTTON_HEIGHT), BOARD_SEED);
}
