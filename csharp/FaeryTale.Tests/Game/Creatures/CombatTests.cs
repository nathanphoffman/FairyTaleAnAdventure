using FaeryTale.Game.Creatures.Combat;
using FaeryTale.Game.Data;

namespace FaeryTale.Tests;

public class CombatTests
{
    private static Melee.SwingFacts Facts(bool byHero, short dist, bool frozen = false, short strength = 3) =>
        new(byHero, dist, 5, frozen, strength);

    private static int NeverRolls() => throw new InvalidOperationException("the hero does not roll");

    [Fact]
    public void DistanceIsTheLargerAxis()
    {
        Assert.Equal(5, Melee.Chebyshev(3, -5));
        Assert.Equal(7, Melee.Chebyshev(-7, 2));
        Assert.Equal(0, Melee.Chebyshev(0, 0));
    }

    [Fact]
    public void BowsAreNotSwungAndNaturalWeaponsCountAsATouch()
    {
        Assert.Null(Melee.MeleeWeapon(Melee.BOW_BIT));
        Assert.Equal((short?)1, Melee.MeleeWeapon(1));
        Assert.Equal((short?)Melee.TOUCH_WEAPON, Melee.MeleeWeapon(8));
        Assert.Equal((short?)Melee.TOUCH_WEAPON, Melee.MeleeWeapon(10));
    }

    [Fact]
    public void ReachIsLimitedAndBraverHeroesReachFurther()
    {
        Assert.Equal(Melee.MAX_REACH, Melee.LimitedReach(Melee.MAX_REACH));
        Assert.Equal(Melee.REACH_LIMIT, Melee.LimitedReach(Melee.MAX_REACH + 1));
        Assert.True(Melee.HeroReach(100) >= Melee.HeroReach(0));
        Assert.Equal(Melee.REACH_LIMIT, Melee.HeroReach(short.MaxValue / 2));
    }

    [Fact]
    public void TheHeroAlwaysHitsWithinReach()
    {
        Assert.Equal(Melee.SwingResult.Hit, Melee.SwingResultFor(Facts(true, 4), 100, NeverRolls));
        Assert.Equal(Melee.SwingResult.Whoosh, Melee.SwingResultFor(Facts(true, 5), 100, NeverRolls));
        Assert.Equal(Melee.SwingResult.Miss, Melee.SwingResultFor(Facts(true, 7), 100, NeverRolls));
    }

    [Fact]
    public void AnEnemyMustBeatTheHerosBravery()
    {
        Assert.Equal(Melee.SwingResult.Hit, Melee.SwingResultFor(Facts(false, 1), 100, () => 101));
        Assert.Equal(Melee.SwingResult.Whoosh, Melee.SwingResultFor(Facts(false, 1), 100, () => 100));
        Assert.Equal(Melee.SwingResult.Whoosh, Melee.SwingResultFor(Facts(false, 1), 100, () => 0));
    }

    [Fact]
    public void TheEnemyRollsEvenWhenOutOfReach()
    {
        int drawn = 0;

        Melee.SwingResultFor(Facts(false, 50), 100, () =>
        {
            drawn++;

            return 0;
        });

        Assert.Equal(1, drawn);
    }

    [Fact]
    public void NothingHitsWhileTimeIsFrozenAndTouchesAreSilent()
    {
        Assert.Equal(Melee.SwingResult.Whoosh, Melee.SwingResultFor(Facts(true, 1, frozen: true), 0, () => 0));
        Assert.Equal(Melee.SwingResult.Miss, Melee.SwingResultFor(Facts(true, 1, frozen: true, strength: Melee.TOUCH_WEAPON), 0, () => 0));
        Assert.Equal(Melee.SwingResult.Miss, Melee.SwingResultFor(Facts(true, 5, strength: Melee.TOUCH_WEAPON), 0, () => 0));
    }

    private static Missile Arrow(sbyte type, sbyte speed, sbyte flight) => new() { MissileType = type, Speed = speed, TimeOfFlight = flight };

    [Fact]
    public void UnusedSpentAndStoppedMissilesExpire()
    {
        Assert.True(Missiles.MissileExpired(Arrow(Missiles.MISSILE_NONE, 5, 0), 10));
        Assert.True(Missiles.MissileExpired(Arrow(Missiles.MISSILE_SPENT, 5, 0), 10));
        Assert.True(Missiles.MissileExpired(Arrow(Missiles.MISSILE_ROCK, 0, 0), 0));
    }

    [Fact]
    public void AMissileCountsFlightTimeUntilItRunsOut()
    {
        Missile missile = Arrow(Missiles.MISSILE_ROCK, 5, 0);

        Assert.False(Missiles.MissileExpired(missile, 10));
        Assert.Equal(1, missile.TimeOfFlight);
        Assert.True(Missiles.MissileExpired(Arrow(Missiles.MISSILE_ROCK, 5, Missiles.MAX_FLIGHT + 1), 10));
    }

    [Fact]
    public void MissilesRollOnlyForTheFirstSlot()
    {
        Func<int> never = () => throw new InvalidOperationException("only slot 0 rolls");

        Assert.True(Missiles.MissileConnects(3, 5, 6, 100, never));
        Assert.False(Missiles.MissileConnects(3, 6, 6, 100, never));
        Assert.True(Missiles.MissileConnects(0, 5, 6, 100, () => 101));
        Assert.False(Missiles.MissileConnects(0, 5, 6, 100, () => 100));
    }
}
