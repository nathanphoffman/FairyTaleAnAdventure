using System.Text;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// The journal (Remastered and Reimagined): everything people have told the hero, kept in the order it
/// was heard, so a hint is not lost when the message log scrolls past it. Tab opens and closes it, or
/// the X closes it. Like the inventory it is drawn over the game, which keeps running, and a click on it
/// is used up. It is kept for the session; a new game starts with an empty one.
public static class Journal
{
    private static readonly List<string> entries = [];
    private static volatile bool open;

    /// Whether the journal is on the screen.
    public static bool IsOpen => open;

    /// Opens it if closed and closes it if open (the Tab key).
    public static void Toggle() => open = !open;

    /// Closes it.
    public static void Close() => open = false;

    /// What has been heard, oldest first.
    public static string[] Entries()
    {
        lock (entries)
        {
            return [.. entries];
        }
    }

    /// Keeps something that was said (message text, where % is the brother's name), unless the same
    /// words are in the journal already.
    public static void Record(Data.Game game, ReadOnlySpan<byte> speech)
    {
        string text = Encoding.Latin1.GetString(speech).Replace("%", Messages.BrotherName(game)).Replace('\r', ' ').Trim();

        lock (entries)
        {
            if (text.Length > 0 && !entries.Contains(text))
            {
                entries.Add(text);
            }
        }
    }

    /// Forgets everything (a new game starts).
    public static void Reset()
    {
        lock (entries)
        {
            entries.Clear();
        }

        open = false;
    }

    /// Does what a click at a place on the overlay grid means. Returns true when the click was on the
    /// journal (it is used up).
    public static bool TryClick((int X, int Y) place)
    {
        if (!open || !InventoryLayout.PanelContains(place))
        {
            return false;
        }

        if (InventoryLayout.CloseContains(place))
        {
            Close();
        }

        return true;
    }
}
