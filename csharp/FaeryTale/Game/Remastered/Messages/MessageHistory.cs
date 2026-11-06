namespace FaeryTale.Game.Remastered;

/// The lines of text the message window (the beige scroll) has shown, so the player can scroll back
/// through them (Remastered and Reimagined). The game thread adds lines; the window thread asks
/// to scroll through MessageScrollRequests and the game thread carries it out.
public sealed class MessageHistory
{
    /// How many lines the message window shows at once.
    public const int VISIBLE_LINES = 4;

    /// Older lines than this are forgotten.
    private const int MAX_LINES = 300;

    private readonly List<byte[]> lines = [];

    /// Whether text printed now continues the last line (the text position is on the message line;
    /// the status line and menus are drawn elsewhere and are not part of the history).
    public bool OnMessageLine { get; set; }

    /// How many lines back from the newest the window is scrolled (0 shows the newest).
    public int Scrolled { get; private set; }

    /// How many lines are kept.
    public int LineCount => lines.Count;

    /// How far back the window can be scrolled.
    public int MaxScroll => Math.Max(0, lines.Count - VISIBLE_LINES);

    /// A new line starts the message line.
    public void StartLine(ReadOnlySpan<byte> text)
    {
        lines.Add(text.ToArray());
        lines.RemoveRange(0, Math.Max(0, lines.Count - MAX_LINES));
        OnMessageLine = true;
    }

    /// Text that goes on at the end of the last line.
    public void Append(ReadOnlySpan<byte> text)
    {
        if (OnMessageLine && lines.Count > 0)
        {
            lines[^1] = [.. lines[^1], .. text];
        }
    }

    /// Scrolls back (positive) or forward (negative) by lines, staying inside the history. Returns
    /// whether the window moved.
    public bool ScrollBy(int count)
    {
        int next = Math.Clamp(Scrolled + count, 0, MaxScroll);
        bool moved = next != Scrolled;

        Scrolled = next;

        return moved;
    }

    /// Jumps back to the newest lines. Returns whether the window moved.
    public bool ScrollToNewest() => ScrollBy(-Scrolled);

    /// The lines the window shows now, oldest first (fewer than VISIBLE_LINES at the very start).
    public IReadOnlyList<byte[]> Window()
    {
        int end = lines.Count - Scrolled;

        return lines.GetRange(Math.Max(0, end - VISIBLE_LINES), Math.Min(VISIBLE_LINES, end));
    }
}
