using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class LookFromHereTests
{
    private static Shape HeroAt(int x, int y)
    {
        Shape hero = new();

        (hero.AbsX, hero.AbsY) = ((ushort)x, (ushort)y);

        return hero;
    }

    [Fact]
    public void ASpotWithinHalfTheLookReachIsLookedAtFromWhereTheHeroStands()
    {
        Shape hero = HeroAt(1000, 1000);

        Assert.True(ClickToMove.LooksFromHere(hero, (1000, 1000)));
        Assert.True(ClickToMove.LooksFromHere(hero, (1020, 980)));
    }

    [Fact]
    public void AFurtherSpotIsWalkedTo()
    {
        Shape hero = HeroAt(1000, 1000);

        Assert.False(ClickToMove.LooksFromHere(hero, (1021, 1000)));
        Assert.False(ClickToMove.LooksFromHere(hero, (1000, 960)));
    }
}
