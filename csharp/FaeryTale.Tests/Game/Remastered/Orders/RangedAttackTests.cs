using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class RangedAttackTests
{
    private const sbyte DIRK = 1;
    private const sbyte BOW = 4;
    private const sbyte WAND = 5;

    [Fact]
    public void ABowIsHeldDrawnAndLetGoForAMomentOverAndOver()
    {
        int held = Enumerable.Range(0, 160).Count(frame => ClickToAttack.ShotHeld(BOW, frame));

        Assert.Equal(140, held);
        Assert.False(ClickToAttack.ShotHeld(BOW, 15));
        Assert.True(ClickToAttack.ShotHeld(BOW, 16));
    }

    [Fact]
    public void AWandIsFiredOnlyAtTheStartOfEachRun()
    {
        Assert.True(ClickToAttack.ShotHeld(WAND, 0));
        Assert.False(ClickToAttack.ShotHeld(WAND, 5));
        Assert.True(ClickToAttack.ShotHeld(WAND, 14));
    }

    [Fact]
    public void AWeaponThatIsSwungIsHeldAllTheTime()
    {
        Assert.All(Enumerable.Range(0, 100), frame => Assert.True(ClickToAttack.ShotHeld(DIRK, frame)));
    }
}
