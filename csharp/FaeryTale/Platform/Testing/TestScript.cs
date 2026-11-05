using FaeryTale.Game;

namespace FaeryTale.Testing;

/// A key pressed (Down true) or released at a tick of 1/50 second.
public readonly record struct KeyEvent(uint Tick, string Name, bool Down);

/// A left click at a tick, at x in 320 wide lores pixels and y in screen lines.
public readonly record struct Click(uint Tick, int X, int Y);

/// A game command to queue at a tick.
public readonly record struct TimedCommand(uint Tick, Command Command);

/// Reads the scripts the test environment variables describe (FTA_INJECT, FTA_CLICK, FTA_CMD), in
/// the same formats the Rust version uses, so the scenario files work for both.
public static class TestScript
{
    /// A key press stays down this many ticks unless the script says otherwise.
    private const uint DEFAULT_HOLD_TICKS = 4;

    /// A test command has at least tick, name and one number.
    private const int MIN_COMMAND_FIELDS = 3;

    public static List<KeyEvent> KeysFromEnvironment() => ParseKeys(Environment.GetEnvironmentVariable("FTA_INJECT"));

    public static List<Click> ClicksFromEnvironment() => ParseClicks(Environment.GetEnvironmentVariable("FTA_CLICK"));

    public static List<TimedCommand> CommandsFromEnvironment() => ParseCommands(Environment.GetEnvironmentVariable("FTA_CMD"));

    /// "100:Space,200:Right:60" presses keys at those ticks, held for 4 ticks or the given number.
    public static List<KeyEvent> ParseKeys(string? text)
    {
        List<KeyEvent> events = new();

        foreach (string item in Items(text))
        {
            string[] parts = item.Split(':');

            if (parts.Length < 2 || !uint.TryParse(parts[0], out uint tick))
            {
                continue;
            }

            uint hold = parts.Length > 2 && uint.TryParse(parts[2], out uint given) ? given : DEFAULT_HOLD_TICKS;

            events.Add(new KeyEvent(tick, parts[1], true));
            events.Add(new KeyEvent(tick + hold, parts[1], false));
        }

        return events;
    }

    /// "200:225:166,260:225:184" clicks at those ticks and positions; items that are not three
    /// numbers are skipped.
    public static List<Click> ParseClicks(string? text)
    {
        List<Click> clicks = new();

        foreach (string item in Items(text))
        {
            string[] parts = item.Split(':');

            if (parts.Length >= 3
                && int.TryParse(parts[0], out int tick)
                && int.TryParse(parts[1], out int x)
                && int.TryParse(parts[2], out int y))
            {
                clicks.Add(new Click((uint)tick, x, y));
            }
        }

        return clicks;
    }

    /// "900:tp:18310:15969,1200:kill:0" queues commands at those ticks: tp, kill, give, stat, slay.
    public static List<TimedCommand> ParseCommands(string? text)
    {
        List<TimedCommand> commands = new();

        foreach (string item in Items(text))
        {
            string[] parts = item.Split(':');

            if (parts.Length < MIN_COMMAND_FIELDS || !uint.TryParse(parts[0], out uint tick))
            {
                continue;
            }

            int first = int.TryParse(parts[2], out int a) ? a : 0;
            int second = parts.Length > 3 && int.TryParse(parts[3], out int b) ? b : 0;

            if (KindFor(parts[1]) is { } kind)
            {
                commands.Add(new TimedCommand(tick, new Command(kind, first, second)));
            }
        }

        return commands;
    }

    private static string[] Items(string? text) => (text ?? "").Split(',');

    private static CommandKind? KindFor(string name) => name switch
    {
        "tp" => CommandKind.Teleport,
        "kill" => CommandKind.KillHero,
        "give" => CommandKind.Give,
        "stat" => CommandKind.SetStat,
        "slay" => CommandKind.SlayEnemies,
        _ => null,
    };
}
