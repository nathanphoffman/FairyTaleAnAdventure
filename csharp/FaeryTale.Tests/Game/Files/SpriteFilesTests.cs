using FaeryTale.Game.Files;

namespace FaeryTale.Tests;

public class SpriteFilesTests
{
    [Fact]
    public void TheSpriteTableHasEighteenFilesAndTheWitchIsAtBlock946()
    {
        Assert.Equal(18, SpriteFiles.All.Length);
        Assert.Equal(946, SpriteFiles.All[16].FileId);
    }
}
