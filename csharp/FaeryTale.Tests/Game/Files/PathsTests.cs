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
        string direction = Path.Combine(Path.GetTempPath(), $"fta_paths_{Environment.ProcessId}");
        Directory.CreateDirectory(Path.Combine(direction, "Fonts"));
        File.WriteAllText(Path.Combine(direction, "Fonts", "Amber"), "x");

        try
        {
            Assert.Equal(Path.Combine(direction, "Fonts", "Amber"), Paths.FindIgnoreCase(direction, "fonts/amber"));
            Assert.Null(Paths.FindIgnoreCase(direction, "fonts/missing"));
        }
        finally
        {
            Directory.Delete(direction, true);
        }
    }
}
