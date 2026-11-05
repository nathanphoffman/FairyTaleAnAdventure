using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Land;

/// The day and night clock: sleeping, the time of day, the light level, the messages when a new
/// part of the day starts, and slow healing.
public static class Clock
{
    /// What one frame of the day clock changes.
    internal readonly record struct DayTick(ushort Clock, ushort Light, bool LampOn, short Period, int? PeriodEvent, bool Heal);

    internal const ushort DAY_LENGTH = 24000;

    /// Sleeping makes time pass this much faster.
    private const ushort SLEEP_TIME_STEP = 63;

    // The hero wakes at dawn (this window) if only a little tired.
    private const ushort DAWN_START = 9000;
    private const ushort DAWN_END = 10000;
    private const short LITTLE_TIRED = 30;
    private const ushort LIGHT_PEAK = 300;
    private const ushort DARK = 40;

    // The night-time object: the lamp post at this slot of the global object list is on in the dark.
    private const int LAMP_SLOT = 5;
    private const sbyte LAMP_ON = 3;
    private const sbyte LAMP_OFF = 2;
    private const ushort PERIOD_LENGTH = 2000;

    /// Events said when a new part of the day starts: (period, event message).
    private static readonly (short Period, int Event)[] PERIOD_EVENTS = [(0, 28), (4, 29), (6, 30), (9, 31)];

    // The hero heals a little every time the clock passes a multiple of this (minus one).
    private const ushort HEAL_MASK = 0x3ff;
    private const short HEAL_BASE = 15;

    // A quarter of the bravery is added to the healing limit.
    private const short BRAVERY_DIVISOR = 4;
    private const ushort WAKE_Y_MASK = 0xffe0;

    /// Sleeping, the day clock, the light level, the messages for the time of day and slow healing.
    public static void DayNightTick(Data.Game game, IOutput output)
    {
        Shape hero = game.Actors.List[0];

        if (hero.State == ActorState.SLEEP)
        {
            SleepTick(game, hero);
        }

        DayTick tick = DayStep(game.World.DayClock, game.World.FreezeTimer != 0, game.World.DayPeriod, hero.Vitality, game.Hero.Brave, hero.State);

        game.World.DayClock = tick.Clock;
        game.World.LightLevel = tick.Light;
        game.Objects.Global[LAMP_SLOT].ObStat = tick.LampOn ? LAMP_ON : LAMP_OFF;
        game.World.DayPeriod = tick.Period;

        if (tick.PeriodEvent is { } message)
        {
            TextRoutines.ShowEvent(game, output, message);
        }

        Fading.DayFade(game, output);

        if (tick.Heal)
        {
            hero.Vitality++;
            TextRoutines.QueuePrint(game.Ui, PrintRequest.VITALITY);
        }
    }

    /// The day clock, light, lamp, part of the day and slow healing after one frame.
    internal static DayTick DayStep(ushort clock, bool frozen, short oldPeriod, short vitality, short bravery, sbyte state)
    {
        ushort newClock = ClockStep(clock, frozen);
        ushort light = LightLevel(newClock);
        short period = DayPeriod(newClock);

        return new DayTick(
            newClock,
            light,
            light < DARK,
            period,
            period != oldPeriod ? PeriodEvent(period) : null,
            HealDue(newClock, vitality, bravery, state));
    }

    /// The day clock after one frame: it stands still while time is frozen and wraps to 0 once it
    /// is past DAY_LENGTH.
    internal static ushort ClockStep(ushort clock, bool frozen)
    {
        if (frozen)
        {
            return clock;
        }

        return clock >= DAY_LENGTH ? (ushort)0 : (ushort)(clock + 1);
    }

    /// The light level for a clock value: it rises to LIGHT_PEAK at midday and falls again.
    internal static ushort LightLevel(ushort clock)
    {
        int level = clock / DARK;

        return (ushort)(level >= LIGHT_PEAK ? 2 * LIGHT_PEAK - level : level);
    }

    /// Which part of the day (0..=11) the clock is in.
    internal static short DayPeriod(ushort clock) => (short)(clock / PERIOD_LENGTH);

    /// The message said when period starts, if it has one.
    internal static int? PeriodEvent(short period) =>
        PERIOD_EVENTS.Where(entry => entry.Period == period).Select(entry => (int?)entry.Event).FirstOrDefault();

    /// Slow healing: once per HEAL_MASK + 1 clock ticks, if hurt and alive.
    internal static bool HealDue(ushort clock, short vitality, short bravery, sbyte state) =>
        (clock & HEAL_MASK) == 0 && vitality < HEAL_BASE + bravery / BRAVERY_DIVISOR && state != ActorState.DEAD;

    /// One frame of sleep: returns the new clock, the new fatigue and whether the hero wakes.
    /// fightWakes is only called (it draws a random number) when a battle is on and no other
    /// reason to wake applied.
    internal static (ushort Clock, short Tired, bool Wake) SleepStep(ushort clock, short tired, bool battle, Func<bool> fightWakes)
    {
        ushort newClock = (ushort)(clock + SLEEP_TIME_STEP);
        short newTired = tired != 0 ? (short)(tired - 1) : tired;
        bool wake = newTired == 0
            || (newTired < LITTLE_TIRED && newClock > DAWN_START && newClock < DAWN_END)
            || (battle && fightWakes());

        return (newClock, newTired, wake);
    }

    // While asleep time runs fast; the hero wakes when rested, at dawn, or when a fight starts.
    private static void SleepTick(Data.Game game, Shape hero)
    {
        (ushort clock, short tired, bool wake) = SleepStep(
            game.World.DayClock,
            game.Hero.Fatigue,
            game.Actors.BattleFlag != 0,
            () => game.Rng.Rand64() == 0);

        game.World.DayClock = clock;
        game.Hero.Fatigue = tired;

        if (wake)
        {
            hero.State = ActorState.STILL;
            hero.AbsY &= WAKE_Y_MASK;
            game.Hero.Y = hero.AbsY;
        }
    }
}
