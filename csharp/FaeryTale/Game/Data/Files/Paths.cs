namespace FaeryTale.Game.Files;

/// Mapping the original file names to real paths.
public static class Paths
{
    private const string DATA_DIR_VAR = "FTA_DATA";
    private const string DEFAULT_DATA_DIR = "faery-tale-amiga/game";
    private const string SAVES_DIR_VAR = "FTA_SAVES";
    private const string DEFAULT_SAVES_DIR = "saves";
    private const string SAVE_EXTENSION = ".faery";

    // A disk device name is "df" plus a unit digit, like "df1".
    private const string DISK_DEVICE_PREFIX = "df";
    private const int DISK_DEVICE_NAME_LENGTH = 3;

    /// Maps an original file name to a real path, or null if it is not there. forWrite allows
    /// names that do not exist yet.
    public static string? Resolve(string name, bool forWrite)
    {
        string? afterDevice = StripDiskDevice(name);

        if (afterDevice is not null)
        {
            return afterDevice.Length == 0 ? SaveDir() : ResolveInSaves(afterDevice, forWrite);
        }

        // Game saves (A.faery ...) go to the saves directory, not the data directory.
        if (name.EndsWith(SAVE_EXTENSION, StringComparison.OrdinalIgnoreCase))
        {
            return ResolveInSaves(name, forWrite);
        }

        // Falls back to the plain data path only when the caller is going to write the file.
        return FindIgnoreCase(DataDir(), name) ?? (forWrite ? Path.Combine(DataDir(), name) : null);
    }

    /// The part after "df1:" if name starts with a disk device, else null.
    public static string? StripDiskDevice(string name)
    {
        int colon = name.IndexOf(':');

        if (colon != DISK_DEVICE_NAME_LENGTH)
        {
            return null;
        }

        bool isDisk = name.StartsWith(DISK_DEVICE_PREFIX, StringComparison.OrdinalIgnoreCase);

        return isDisk ? name[(colon + 1)..] : null;
    }

    /// Finds name under baseDir, matching each path component case-insensitively.
    public static string? FindIgnoreCase(string baseDir, string name)
    {
        string current = baseDir;

        foreach (string part in name.Split('/', StringSplitOptions.RemoveEmptyEntries))
        {
            string exact = Path.Combine(current, part);

            if (Path.Exists(exact))
            {
                current = exact;

                continue;
            }

            string? found = FindEntryIgnoreCase(current, part);

            if (found is null)
            {
                return null;
            }

            current = found;
        }

        return current;
    }

    private static string? FindEntryIgnoreCase(string directory, string part)
    {
        if (!Directory.Exists(directory))
        {
            return null;
        }

        return Directory.EnumerateFileSystemEntries(directory)
            .FirstOrDefault(entry => string.Equals(Path.GetFileName(entry), part, StringComparison.OrdinalIgnoreCase));
    }

    // Finds (or, for writing, makes room for) name in the saves directory.
    private static string? ResolveInSaves(string name, bool forWrite)
    {
        string directory = SaveDir();

        if (!forWrite)
        {
            return FindIgnoreCase(directory, name);
        }

        Directory.CreateDirectory(directory);

        return Path.Combine(directory, name);
    }

    private static string DataDir() => Environment.GetEnvironmentVariable(DATA_DIR_VAR) ?? DEFAULT_DATA_DIR;

    private static string SaveDir() => Environment.GetEnvironmentVariable(SAVES_DIR_VAR) ?? DEFAULT_SAVES_DIR;
}
