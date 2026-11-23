using FaeryTale.Game.Input;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Video: printing lines into the message area.
public sealed partial class Video
{
    /// A line of text in the message area; in Remastered and Reimagined a line that is too wide wraps
    /// onto more lines by itself.
    public void PrintBytes(ReadOnlySpan<byte> text)
    {
        if (!Capture.Enabled)
        {
            PrintLine(text);

            return;
        }

        foreach (byte[] line in MessageWrapper.Wrap(text, TEXT_X_MAX - TEXT_X_MIN + 1, character => Screen.AdvanceOf(Screen.CurrentPort, character)))
        {
            PrintLine(line);
        }
    }

    // The old lines scroll up by one text line (10 pixels) inside the message area.
    private void PrintLine(ReadOnlySpan<byte> text)
    {
        ShowNewestMessages();

        Rect area = new(TEXT_X_MIN, TEXT_Y_MIN, TEXT_X_MAX - TEXT_X_MIN + 1, TEXT_Y_MAX - TEXT_Y_MIN + 1);

        Screen.Scroll(Screen.CurrentPort, area, (0, LINE_HEIGHT));
        TextRoutines.MoveTo(this, TEXT_X_MIN, NEW_LINE_Y);
        DrawMessageText(Screen.CurrentPort, text);

        if (Capture.Enabled)
        {
            History.StartLine(text);
        }
    }

    public void PrintContBytes(ReadOnlySpan<byte> text)
    {
        ShowNewestMessages();

        if (History.OnMessageLine)
        {
            DrawMessageText(Screen.CurrentPort, text);
        }
        else
        {
            Screen.Text(Screen.CurrentPort, text);
        }

        if (Capture.Enabled)
        {
            History.Append(text);
        }
    }

    public void PrintNumber(int value, int length)
    {
        byte[] text = TextRoutines.FormatNumber((uint)value);
        int from = Math.Max(NUMBER_FIELD - length, 0);

        ShowNewestMessages();
        Screen.Text(Screen.CurrentPort, text.AsSpan(from));

        if (Capture.Enabled)
        {
            History.Append(text.AsSpan(from));
        }
    }

    /// Scrolls the message window the way the player asked (mouse wheel, Page Up and Page Down).
    /// Called every frame; the classic game never asks.
    public void ShowAskedMessageScroll()
    {
        if (!Capture.Enabled)
        {
            return;
        }

        History.ScrollBy(MessageScrollRequests.Take());

        if (MessageScrollBar.DragTarget(History, InputHandler.GridPointer, InputHandler.LeftButtonDown) is { } dragged)
        {
            History.ScrollBy(dragged - History.Scrolled);
        }

        if (shownScroll != History.Scrolled)
        {
            DrawMessageWindow();
        }
    }

    // Before anything new is printed the window goes back to the newest lines.
    private void ShowNewestMessages()
    {
        if (!Capture.Enabled)
        {
            return;
        }

        History.ScrollToNewest();

        if (shownScroll != 0)
        {
            DrawMessageWindow();
        }
    }
}
