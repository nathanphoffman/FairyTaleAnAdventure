using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class ItemStatsTests
{
    [Fact]
    public void EveryWeaponAndMagicStoneAndKeySaysWhatItDoes()
    {
        for (int slot = 0; slot <= 4; slot++)
        {
            Assert.NotEmpty(ItemStats.For(slot));
        }

        for (int slot = 9; slot <= 21; slot++)
        {
            Assert.NotEmpty(ItemStats.For(slot));
        }
    }

    [Fact]
    public void TheSpellTimesAreInSeconds()
    {
        Assert.Contains("15 seconds", ItemStats.For(10));
        Assert.Contains("2 seconds", ItemStats.For(14));
    }

    [Fact]
    public void AnItemWithNothingToSaySaysNothing()
    {
        Assert.Equal("", ItemStats.For(27));
    }
}
