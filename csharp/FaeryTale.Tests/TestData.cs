namespace FaeryTale.Tests;

/// Finds the original game files, which are not in the repo.
public static class TestData
{
    /// The folder holding the disk image, or null when the files are not there.
    public static string? FindGameFolder()
    {
        for (DirectoryInfo? direction = new(AppContext.BaseDirectory); direction is not null; direction = direction.Parent)
        {
            string candidate = Path.Combine(direction.FullName, "faery-tale-amiga", "game");

            if (File.Exists(Path.Combine(candidate, "image")))
            {
                return candidate;
            }
        }

        return null;
    }
}
