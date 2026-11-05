namespace FaeryTale.Tests;

/// Finds the original game files, which are not in the repo.
public static class TestData
{
    /// The folder holding the disk image, or null when the files are not there.
    public static string? FindGameFolder()
    {
        for (DirectoryInfo? dir = new(AppContext.BaseDirectory); dir is not null; dir = dir.Parent)
        {
            string candidate = Path.Combine(dir.FullName, "faery-tale-amiga", "game");

            if (File.Exists(Path.Combine(candidate, "image")))
            {
                return candidate;
            }
        }

        return null;
    }
}
