using FaeryTale.Game.Files;

namespace FaeryTale.Tests;

[Collection("Game disk")]
public class DiskTests
{
    private const int BLOCK_SIZE = 512;

    [Fact]
    public void ABlockReadMatchesTheFileAndIsPaddedPastTheEnd()
    {
        string? folder = TestData.FindGameFolder();

        if (folder is null)
        {
            return;
        }

        Environment.SetEnvironmentVariable("FTA_DATA", folder);

        try
        {
            Assert.Equal(0, Disk.Open());
            byte[] raw = File.ReadAllBytes(Path.Combine(folder, "image"));
            short lastBlock = (short)(raw.Length / BLOCK_SIZE - 1);

            Assert.Equal(raw[(3 * BLOCK_SIZE)..(5 * BLOCK_SIZE)], Disk.ReadBlocks(3, 2));
            Assert.Equal(raw[(lastBlock * BLOCK_SIZE)..], Disk.ReadBlocks(lastBlock, 1)[..(raw.Length - lastBlock * BLOCK_SIZE)]);
            Assert.Equal(new byte[BLOCK_SIZE], Disk.ReadBlocks((short)(lastBlock + 5), 1));
        }
        finally
        {
            Disk.Close();
            Environment.SetEnvironmentVariable("FTA_DATA", null);
        }
    }

    [Fact]
    public void AMissingImageGivesTheNoFileError()
    {
        Environment.SetEnvironmentVariable("FTA_DATA", Path.Combine(Path.GetTempPath(), "fta_no_such_folder"));

        try
        {
            Assert.Equal(Disk.ERR_NO_FILE, Disk.Open());
        }
        finally
        {
            Environment.SetEnvironmentVariable("FTA_DATA", null);
        }
    }
}
