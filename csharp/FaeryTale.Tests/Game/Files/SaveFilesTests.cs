using FaeryTale.Game.Files;

namespace FaeryTale.Tests;

public class SaveFilesTests
{
    // A save written by the Rust game (walked right, then saved to slot A).
    private static readonly string RUST_SAVE = Path.Combine(AppContext.BaseDirectory, "Fixtures", "rust_save_A.faery");

    [Fact]
    public void NumbersAreStoredBigEndian()
    {
        Transfer transfer = Transfer.ForSaving();
        ushort word = 0x1234;
        byte single = 0x56;
        short signed = -2;

        transfer.Word(ref word);
        transfer.Byte(ref single);
        transfer.Word(ref signed);

        Assert.Equal(new byte[] { 0x12, 0x34, 0x56, 0xff, 0xfe }, transfer.Finish());
    }

    [Fact]
    public void ReadingGivesBackWhatWasWritten()
    {
        Transfer transfer = Transfer.ForLoading([0x12, 0x34, 0x56, 0xff, 0xfe]);
        ushort word = 0;
        byte single = 0;
        short signed = 0;

        transfer.Word(ref word);
        transfer.Byte(ref single);
        transfer.Word(ref signed);

        Assert.Equal((0x1234, 0x56, -2), (word, single, signed));
    }

    [Fact]
    public void AFileThatEndsEarlyLeavesTheRestAlone()
    {
        Transfer transfer = Transfer.ForLoading([0x12]);
        ushort word = 0x5678;

        transfer.Word(ref word);

        Assert.Equal(0x1278, word);
    }

    [Fact]
    public void ASaveFromTheRustGameLoadsAndSavesBackIdentically()
    {
        byte[] original = File.ReadAllBytes(RUST_SAVE);
        FaeryTale.Game.Data.Game game = new();

        GameTransfer.TransferAll(game, Transfer.ForLoading(original));
        Transfer again = Transfer.ForSaving();
        GameTransfer.TransferAll(game, again);

        Assert.Equal(original, again.Finish());
    }

    [Fact]
    public void TheRustSaveHasTheHeroOnTheFirstBrother()
    {
        FaeryTale.Game.Data.Game game = new();

        GameTransfer.TransferAll(game, Transfer.ForLoading(File.ReadAllBytes(RUST_SAVE)));

        Assert.Equal(1, game.Hero.Brother);
        Assert.Equal(0x4a14, game.World.MapX);
        Assert.True(game.Actors.Active > 0);
    }

    [Fact]
    public void SlotsAreNamedByLetter()
    {
        Assert.Equal("A.faery", SaveFiles.NameForSlot(0));
        Assert.Equal("C.faery", SaveFiles.NameForSlot(2));
    }

    [Fact]
    public void WritingAndReadingUseTheSavesFolder()
    {
        string folder = Path.Combine(Path.GetTempPath(), $"fta_saves_{Environment.ProcessId}");
        Environment.SetEnvironmentVariable("FTA_SAVES", folder);

        try
        {
            FaeryTale.Game.Data.Game game = new();
            game.Hero.Luck = 77;
            SaveFiles.Write(game, "B.faery");

            FaeryTale.Game.Data.Game loaded = new();
            SaveFiles.Read(loaded, "b.FAERY");

            Assert.Equal(77, loaded.Hero.Luck);
        }
        finally
        {
            Environment.SetEnvironmentVariable("FTA_SAVES", null);
            Directory.Delete(folder, true);
        }
    }
}
