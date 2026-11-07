namespace FaeryTale.Platform;

/// Key remapping from controls.cfg (or the file named by FTA_CONTROLS). Each line is
/// "Key = Action" where both are SDL key names, for example "W = Up". # starts a comment.
public static class KeyRemap
{
    private const string CONFIG_VARIABLE = "FTA_CONTROLS";
    private const string DEFAULT_CONFIG = "controls.cfg";

    private static readonly Dictionary<string, string> REMAP = [];

    // While the controls screen waits for a key to bind, the next key pressed is kept here and does
    // not reach the game. The window thread sets it and the game thread reads it.
    private static volatile bool capturing;
    private static volatile string? captured;

    private static string ConfigPath => Environment.GetEnvironmentVariable(CONFIG_VARIABLE) ?? DEFAULT_CONFIG;

    /// The key name the game should see when the key with this name is pressed.
    public static string Apply(string name) => REMAP.GetValueOrDefault(name, name);

    /// Reads the config file. No file is normal.
    public static void Load()
    {
        string path = ConfigPath;

        if (!File.Exists(path))
        {
            return;
        }

        foreach (string line in File.ReadAllLines(path))
        {
            if (ParseLine(line) is { } pair)
            {
                REMAP[pair.From] = pair.To;
            }
            else if (line.Split('#')[0].Trim().Length > 0)
            {
                Console.Error.WriteLine($"{path}: cannot read line '{line}'");
            }
        }
    }

    /// The key the player gave to a game key (the name the game sees, like "Up" or "I"), or null when it
    /// still has only its own.
    public static string? KeyFor(string gameKey) => REMAP.FirstOrDefault(pair => pair.Value == gameKey).Key;

    /// Makes the key press the game key from now on (and gives up the key it had before), and saves.
    public static void Bind(string key, string gameKey)
    {
        foreach (string old in REMAP.Where(pair => pair.Value == gameKey).Select(pair => pair.Key).ToList())
        {
            REMAP.Remove(old);
        }

        REMAP[key] = gameKey;
        Save();
    }

    /// Takes every key the player set back to the keys the game came with, and saves.
    public static void ResetAll()
    {
        REMAP.Clear();
        Save();
    }

    /// Waits for the next key press and keeps it (see TakeCaptured).
    public static void StartCapture()
    {
        captured = null;
        capturing = true;
    }

    /// Gives up waiting for a key.
    public static void StopCapture() => capturing = false;

    /// Called with each key pressed: keeps it and returns true while a capture is waiting (the key is
    /// then not for the game).
    public static bool Offer(string name)
    {
        if (!capturing)
        {
            return false;
        }

        captured = name;
        capturing = false;

        return true;
    }

    /// The key that was captured, once (null while none has been pressed).
    public static string? TakeCaptured()
    {
        string? name = captured;

        captured = null;

        return name;
    }

    // Writes the keys the player set in the format Load reads, so they stay for the next run.
    private static void Save()
    {
        List<string> lines = ["# Keys set in the game's Controls screen. Format: Key = Action (SDL key names)."];

        lines.AddRange(REMAP.Select(pair => $"{pair.Key} = {pair.Value}"));

        try
        {
            File.WriteAllLines(ConfigPath, lines);
        }
        catch (IOException problem)
        {
            Console.Error.WriteLine($"{ConfigPath}: cannot save the keys: {problem.Message}");
        }
    }

    /// Splits "Key = Action  # comment" into its two names. Null for blank, comment and
    /// malformed lines.
    internal static (string From, string To)? ParseLine(string line)
    {
        string[] sides = line.Split('#')[0].Trim().Split('=', 2);

        if (sides.Length != 2 || sides[0].Trim().Length == 0 || sides[1].Trim().Length == 0)
        {
            return null;
        }

        return (sides[0].Trim(), sides[1].Trim());
    }
}
