using FaeryTale.Game.Creatures.Revive;
using FaeryTale.Game.Data;

namespace FaeryTale.Tests;

public class ReviveTests
{
    [Fact]
    public void ANewBrotherStartsCleanAtTheFirstSafeSpot()
    {
        Hero hero = new();
        World world = new() { SecretTimer = 5, LightTimer = 6, FreezeTimer = 7, Region = 9 };

        Brothers.ResetForNextBrother(hero, world);

        Assert.Equal((0, 0, 0), (world.SecretTimer, world.LightTimer, world.FreezeTimer));
        Assert.Equal(Brothers.FIRST_REGION, world.Region);
        Assert.Equal((Brothers.FIRST_SAFE_SPOT_X, Brothers.FIRST_SAFE_SPOT_Y, Brothers.FIRST_REGION), (hero.SafeX, hero.SafeY, hero.SafeR));
    }

    [Fact]
    public void TheHeroReappearsAtTheSafeSpotWithTheCameraCentred()
    {
        Hero hero = new() { SafeX = 1000, SafeY = 2000, SafeR = 5 };
        Shape body = new();
        World world = new();

        SafeSpot.MoveToSafeSpot(hero, body, world);

        Assert.Equal((1000, 2000, 1000, 2000), (hero.X, hero.Y, body.AbsX, body.AbsY));
        Assert.Equal((1000 - SafeSpot.CAMERA_OFFSET_X, 2000 - SafeSpot.CAMERA_OFFSET_Y), (world.MapX, world.MapY));
        Assert.Equal(5, world.NewRegion);
    }

    [Fact]
    public void ARevivedHeroIsHealthyRestedInTheMorning()
    {
        Hero hero = new() { Brave = 40, Hunger = 90, Fatigue = 80 };
        Shape body = new() { State = 15, Environ = 3 };
        World world = new() { DayClock = 100 };

        SafeSpot.RestoreStats(hero, body, world);

        Assert.Equal((25, ActorState.STILL, 0, 255), (body.Vitality, body.State, body.Environ, body.Race));
        Assert.Equal((0, 0), (hero.Hunger, hero.Fatigue));
        Assert.Equal((SafeSpot.START_DAYNIGHT, SafeSpot.START_LIGHT), (world.DayClock, world.LightLevel));
    }

    [Fact]
    public void TheNextBrotherStartsWithHisOwnStatsHoldingOnlyADirk()
    {
        Bro next = new() { Brave = 20, Luck = 35, Kind = 15, Wealth = 15 };
        Hero hero = new() { Brother = 1 };
        Array.Fill(hero.Philstuff, (byte)7);
        Shape body = new();

        Brothers.StartNextBrother(hero, body, next);

        Assert.Equal((20, 35, 15, 15), (hero.Brave, hero.Luck, hero.Kindness, hero.Wealth));
        Assert.Equal(2, hero.Brother);
        Assert.Equal(1, body.Weapon);
        Assert.Equal(1, hero.Philstuff[0]);
        Assert.All(hero.Philstuff[1..Brothers.GOLDBASE], count => Assert.Equal(0, count));
        Assert.Equal(7, hero.Philstuff[Brothers.GOLDBASE]); // what is past the item slots is left alone
    }
}
