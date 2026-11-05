namespace FaeryTale.Game;

/// The kinds of command a test script can queue (FTA_CMD).
public enum CommandKind
{
    /// Move the hero to (First, Second), like going through a door.
    Teleport,

    /// Kill the hero, who has First luck left.
    KillHero,

    /// Set inventory slot First to a count of Second.
    Give,

    /// Set a stat First (0 hunger, 1 fatigue, anything else wealth) to Second.
    SetStat,

    /// Kill every enemy that is out.
    SlayEnemies,
}

public readonly record struct Command(CommandKind Kind, int First, int Second);

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
