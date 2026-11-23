using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Video: drawing the message text and item names.
public sealed partial class Video
{
    // Draws the lines of the history the window shows now, the newest at the bottom, and leaves the
    // text position at the end of the bottom line.
    private void DrawMessageWindow()
    {
        Rect area = new(TEXT_X_MIN, TEXT_Y_MIN, TEXT_X_MAX - TEXT_X_MIN + 1, TEXT_Y_MAX - TEXT_Y_MIN + 1);
        IReadOnlyList<byte[]> window = History.Window();

        shownScroll = History.Scrolled;
        Screen.Scroll(PortId.Text, area, (0, area.Height));

        for (int index = 0; index < window.Count; index++)
        {
            int rowsFromBottom = window.Count - 1 - index;

            Screen.MoveTo(PortId.Text, TEXT_X_MIN, NEW_LINE_Y - rowsFromBottom * LINE_HEIGHT);
            DrawMessageText(PortId.Text, window[index]);
        }
    }

    /// How far across a character of the message font reaches.
    public int MessageAdvance(byte character) => Screen.AdvanceOf(PortId.Text, character);

    // Writes message text at the pen. In Remastered and Reimagined the names of items are written
    // in dark brown and underlined, so they stand out as things the pointer can ask about.
    private void DrawMessageText(PortId port, ReadOnlySpan<byte> text)
    {
        if (!Capture.Enabled || Capture.Game is not { } game)
        {
            Screen.Text(port, text);

            return;
        }

        int done = 0;

        foreach (ItemWord word in ItemWords.Find(game, text))
        {
            Screen.Text(port, text[done..word.Start]);
            DrawItemName(port, text.Slice(word.Start, word.Length));
            done = word.Start + word.Length;
        }

        Screen.Text(port, text[done..]);
    }

    // One item name: dark brown with a line under it, then the pen goes back as it was.
    private void DrawItemName(PortId port, ReadOnlySpan<byte> name)
    {
        Port current = Screen.GetPort(port);
        (byte color, int left, int baseline) = (current.Pen.Fg, current.Pen.X, current.Pen.Y);

        Screen.SetAPen(port, ITEM_NAME_PEN);
        Screen.Text(port, name);
        Screen.RectFill(port, (left, baseline + UNDERLINE_OFFSET), (current.Pen.X - 1, baseline + UNDERLINE_OFFSET));
        Screen.SetAPen(port, color);
    }

    public void SetTextPosition(int x, int y)
    {
        History.OnMessageLine = y == NEW_LINE_Y;
        Screen.MoveTo(Screen.CurrentPort, x, y);
    }

    public void SelectMessageArea() => Screen.CurrentPort = PortId.Text;
}
