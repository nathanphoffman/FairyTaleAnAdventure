namespace FaeryTale.Platform;

/// Key remapping from controls.cfg (or the file named by FTA_CONTROLS). Each line is
/// "Key = Action" where both are SDL key names, for example "W = Up". # starts a comment.
public static class KeyRemap
{
    private const string CONFIG_VARIABLE = "FTA_CONTROLS";
    private const string DEFAULT_CONFIG = "controls.cfg";

    private static readonly Dictionary<string, string> REMAP = [];

    /// The key name the game should see when the key with this name is pressed.
    public static string Apply(string name) => REMAP.GetValueOrDefault(name, name);

    /// Reads the config file. No file is normal.
    public static void Load()
    {
        string path = Environment.GetEnvironmentVariable(CONFIG_VARIABLE) ?? DEFAULT_CONFIG;

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
