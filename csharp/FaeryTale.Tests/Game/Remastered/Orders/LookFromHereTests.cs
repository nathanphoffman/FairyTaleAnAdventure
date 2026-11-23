using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class LookFromHereTests
{
    private static Shape HeroAt(int positionX, int positionY)
    {
        Shape hero = new();

        (hero.AbsX, hero.AbsY) = ((ushort)positionX, (ushort)positionY);

        return hero;
    }

    [Fact]
    public void ASpotWithinHalfTheLookReachIsLookedAtFromWhereTheHeroStands()
    {
        Shape hero = HeroAt(1000, 1000);

        Assert.True(LookWalks.LooksFromHere(hero, (1000, 1000)));
        Assert.True(LookWalks.LooksFromHere(hero, (1020, 980)));
    }

    [Fact]
    public void AFurtherSpotIsWalkedTo()
    {
        Shape hero = HeroAt(1000, 1000);

        Assert.False(LookWalks.LooksFromHere(hero, (1021, 1000)));
        Assert.False(LookWalks.LooksFromHere(hero, (1000, 960)));
    }
}
