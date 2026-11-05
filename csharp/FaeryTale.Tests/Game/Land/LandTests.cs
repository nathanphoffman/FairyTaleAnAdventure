using FaeryTale.Game.Data;
using FaeryTale.Game.Land;

namespace FaeryTale.Tests;

public class LandTests
{
    private const sbyte STILL = ActorState.STILL;
    private const sbyte DEAD = ActorState.DEAD;

    [Fact]
    public void KeysMustMatchUnlessNoneIsNeeded()
    {
        Assert.NotNull(KeyDoors.FindOpen(64, 360, KeyDoors.GREEN));
        Assert.Null(KeyDoors.FindOpen(64, 360, 0));
        Assert.Null(KeyDoors.FindOpen(64, 360, KeyDoors.GOLD));

        // A wooden door needs no key: walking into it (key 0) opens it.
        Assert.Equal(125, KeyDoors.FindOpen(120, 360, 0)!.New1);

        // The same door tile in another map set is a different door.
        Assert.Equal(124, KeyDoors.FindOpen(64, 280, KeyDoors.GREY)!.New1);
        Assert.Null(KeyDoors.FindOpen(64, 999, KeyDoors.GREEN));
    }

    [Fact]
    public void OpenTileLayouts()
    {
        Assert.Equal([(10, 5, (byte)123), (11, 5, (byte)124)], KeyDoors.OpenChanges(KeyDoors.FindOpen(64, 360, KeyDoors.GREEN)!, 10, 5));
        Assert.Equal([(10, 5, (byte)127)], KeyDoors.OpenChanges(KeyDoors.FindOpen(122, 360, 0)!, 10, 5));
        Assert.Equal([(10, 5, (byte)154), (10, 4, (byte)155)], KeyDoors.OpenChanges(KeyDoors.FindOpen(128, 240, KeyDoors.WHITE)!, 10, 5));
        Assert.Equal([(10, 5, (byte)27), (9, 5, (byte)26)], KeyDoors.OpenChanges(KeyDoors.FindOpen(25, 680, KeyDoors.GOLD)!, 10, 5));
        Assert.Equal([(10, 5, (byte)133), (11, 5, (byte)134), (12, 5, (byte)135)], KeyDoors.OpenChanges(KeyDoors.FindOpen(136, 800, KeyDoors.GOLD)!, 10, 5));
        Assert.Equal(
            [(10, 5, (byte)85), (10, 4, (byte)87), (11, 5, (byte)86), (11, 4, (byte)88)],
            KeyDoors.OpenChanges(KeyDoors.FindOpen(165, 800, KeyDoors.GREEN)!, 10, 5));
    }

    [Fact]
    public void ATiredHeroFallsAsleepAfterThirtyFramesOnABed()
    {
        short wait = 0;
        Bed.BedAction last = Bed.BedAction.Nothing;

        for (int frame = 1; frame <= Bed.BED_WAIT; frame++)
        {
            (wait, last) = Bed.BedStep(wait, true, Bed.TOO_RESTED);
            Assert.Equal(frame == Bed.BED_WAIT, last == Bed.BedAction.FallAsleep);
        }
    }

    [Fact]
    public void ARestedHeroIsToldHeIsNotTiredAndLeavingTheBedResetsTheCount()
    {
        Assert.Equal(Bed.BedAction.NotTired, Bed.BedStep(Bed.BED_WAIT - 1, true, Bed.TOO_RESTED - 1).Action);
        Assert.Equal(((short)0, Bed.BedAction.Nothing), Bed.BedStep(20, false, 100));
    }

    [Fact]
    public void AFrameOfTheDay()
    {
        Clock.DayTick tick = Clock.DayStep(1999, false, 0, 50, 0, STILL);
        Assert.Equal(new Clock.DayTick(2000, 50, false, 1, null, false), tick);

        Assert.Equal(((short)4, (int?)29), (Clock.DayStep(7999, false, 3, 50, 0, STILL).Period, Clock.DayStep(7999, false, 3, 50, 0, STILL).PeriodEvent));
        Assert.Null(Clock.DayStep(8000, false, 4, 50, 0, STILL).PeriodEvent);
        Assert.True(Clock.DayStep(100, false, 0, 50, 0, STILL).LampOn);
        Assert.Equal(500, Clock.DayStep(500, true, 0, 50, 0, STILL).Clock);

        Assert.True(Clock.DayStep(1023, false, 0, 10, 0, STILL).Heal);
        Assert.False(Clock.DayStep(1023, false, 0, 10, 0, DEAD).Heal);
        Assert.False(Clock.DayStep(1024, false, 0, 10, 0, STILL).Heal);
    }

    [Fact]
    public void ClockCountsWrapsFreezes()
    {
        Assert.Equal(1, Clock.ClockStep(0, false));
        Assert.Equal(500, Clock.ClockStep(500, true));
        Assert.Equal(Clock.DAY_LENGTH, Clock.ClockStep(Clock.DAY_LENGTH - 1, false));
        Assert.Equal(0, Clock.ClockStep(Clock.DAY_LENGTH, false));
    }

    [Fact]
    public void LightFollowsTheSun()
    {
        Assert.Equal(0, Clock.LightLevel(0));
        Assert.Equal(0, Clock.LightLevel(39));
        Assert.Equal(1, Clock.LightLevel(40));
        Assert.Equal(299, Clock.LightLevel(300 * 40 - 1));
        Assert.Equal(300, Clock.LightLevel(300 * 40));
        Assert.Equal(299, Clock.LightLevel(301 * 40));
        Assert.Equal(1, Clock.LightLevel(23999));
    }

    [Fact]
    public void DayPeriodsWithMessages()
    {
        Assert.Equal(0, Clock.DayPeriod(0));
        Assert.Equal(0, Clock.DayPeriod(1999));
        Assert.Equal(1, Clock.DayPeriod(2000));
        Assert.Equal(11, Clock.DayPeriod(23999));

        var said = Enumerable.Range(0, 12).Select(p => (p, Clock.PeriodEvent((short)p))).Where(pair => pair.Item2 is not null).ToList();
        Assert.Equal([(0, (int?)28), (4, 29), (6, 30), (9, 31)], said);
    }

    [Fact]
    public void HealingEvery1024Ticks()
    {
        Assert.True(Clock.HealDue(0, 10, 0, STILL));
        Assert.True(Clock.HealDue(1024, 10, 0, STILL));
        Assert.False(Clock.HealDue(1023, 10, 0, STILL));
        Assert.False(Clock.HealDue(1024, 15, 0, STILL));
        Assert.True(Clock.HealDue(1024, 15, 4, STILL));
        Assert.False(Clock.HealDue(1024, 10, 0, DEAD));
    }

    [Fact]
    public void SleepStepsFastWaking()
    {
        int drawn = 0;
        Func<bool> draw = () =>
        {
            drawn++;

            return true;
        };

        Assert.Equal(((ushort)163, (short)49, false), Clock.SleepStep(100, 50, false, draw));
        Assert.Equal(0, drawn);
        Assert.True(Clock.SleepStep(100, 1, false, () => false).Wake);
        Assert.True(Clock.SleepStep(9000, 20, false, () => false).Wake);
        Assert.False(Clock.SleepStep(9000, 40, false, () => false).Wake);
        Assert.False(Clock.SleepStep(8000, 20, false, () => false).Wake);

        Assert.True(Clock.SleepStep(100, 50, true, draw).Wake);
        Assert.Equal(1, drawn);

        Clock.SleepStep(100, 1, true, draw);
        Assert.Equal(1, drawn);
        Assert.Equal(0, Clock.SleepStep(0, 0, false, () => false).Tired);
    }
}
