namespace FaeryTale.Game.Remastered;

/// Scrolling asked for by the window thread (mouse wheel, Page Up and Page Down), carried out by
/// the game thread at the top of its next frame.
public static class MessageScrollRequests
{
    private static readonly object padlock = new();
    private static int pending;

    /// Asks to scroll back (positive) or forward (negative) by lines.
    public static void Add(int lines)
    {
        lock (padlock)
        {
            pending += lines;
        }
    }

    /// The lines asked for since the last call.
    public static int Take()
    {
        lock (padlock)
        {
            int asked = pending;

            pending = 0;

            return asked;
        }
    }
}
