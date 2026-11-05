using FaeryTale.Game.Files;

namespace FaeryTale.Tests;

public class PathsTests
{
    [Fact]
    public void DiskDevicePrefixIsRemoved()
    {
        Assert.Equal("A.faery", Paths.StripDiskDevice("df1:A.faery"));
        Assert.Equal("", Paths.StripDiskDevice("DF0:"));
        Assert.Null(Paths.StripDiskDevice("image"));
        Assert.Null(Paths.StripDiskDevice("dh0:file"));
    }

    [Fact]
    public void FindsFilesIgnoringCase()
    {
        string dir = Path.Combine(Path.GetTempPath(), $"fta_paths_{Environment.ProcessId}");
        Directory.CreateDirectory(Path.Combine(dir, "Fonts"));
        File.WriteAllText(Path.Combine(dir, "Fonts", "Amber"), "x");

        try
        {
            Assert.Equal(Path.Combine(dir, "Fonts", "Amber"), Paths.FindIgnoreCase(dir, "fonts/amber"));
            Assert.Null(Paths.FindIgnoreCase(dir, "fonts/missing"));
        }
        finally
        {
            Directory.Delete(dir, true);
        }
    }
}
