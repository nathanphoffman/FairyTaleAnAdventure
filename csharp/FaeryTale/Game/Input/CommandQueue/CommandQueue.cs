namespace FaeryTale.Game;

/// Holds the command a test script asked for until the top of the game's next frame.
/// A later command replaces an earlier one that was not run yet.
public static class CommandQueue
{
    private static readonly object padlock = new();
    private static Command? pending;

    public static void Queue(Command command)
    {
        lock (padlock)
        {
            pending = command;
        }
    }

    /// The queued command, which is then removed, or null if there is none.
    public static Command? Take()
    {
        lock (padlock)
        {
            Command? taken = pending;
            pending = null;

            return taken;
        }
    }
}
