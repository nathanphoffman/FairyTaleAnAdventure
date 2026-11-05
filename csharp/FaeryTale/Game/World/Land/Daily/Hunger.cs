using FaeryTale.Game.Data;

namespace FaeryTale.Game.Land;

/// What hunger and tiredness do to the hero: messages, lost vitality, collapsing and fainting.
public static class Hunger
{
    /// What hunger and fatigue did this time: the event messages to show (in order) and whether the
    /// stats line needs redrawing.
    internal sealed record HungerReport(List<int> Events, bool RedrawStats);

    internal const short HUNGRY = 35;
    internal const short STARVING_AT = 90;
    internal const short WEAK_VITALITY = 5;
    internal const short MUCH_FATIGUE = 160;
    internal const short COLLAPSE_FATIGUE = 170;
    internal const short FAINT_HUNGER = 140;
    internal const short FAINT_RESET = 130;

    private const short VERY_HUNGRY = 60;
    private const short MUCH_HUNGER = 100;
    private const short HUNGER_STEP_MASK = 7;

    /// Hungry enough to eat fruit at the safe spot.
    private const short EATS_FRUIT_AT = 30;

    // Hunger, fatigue and the safe spot are checked when the clock is a multiple of 128.
    private const ushort SAFE_CHECK_MASK = 127;

    // Event messages for hunger and tiredness.
    private const int EVENT_HUNGRY = 0;
    private const int EVENT_VERY_HUNGRY = 1;
    private const int EVENT_STARVING_WARNING = 2;
    private const int EVENT_COLLAPSE = 12;
    private const int EVENT_FAINT = 24;

    /// Whether the hunger, fatigue and safe-spot checks run on this clock value.
    internal static bool SafeCheckDue(ushort clock) => (clock & SAFE_CHECK_MASK) == 0;

    /// The hero is hungrier and more tired on a safe-check frame while alive and awake.
    internal static bool HungerDue(ushort clock, short vitality, sbyte state) =>
        SafeCheckDue(clock) && vitality != 0 && state != ActorState.SLEEP;

    /// Hungry enough to eat a piece of fruit at the safe spot (only when one is carried).
    internal static bool EatsFruit(short hungry, byte fruit) => hungry > EATS_FRUIT_AT && fruit != 0;

    /// The messages and damage that come with hunger and fatigue. Every 8 points of hunger the
    /// hero loses vitality when very hungry or tired, and when weak either collapses from fatigue
    /// or faints from hunger (falling asleep).
    internal static HungerReport HungerEffects(Hero hero, Shape body)
    {
        HungerReport report = new([], false);

        if (hero.Hunger == HUNGRY)
        {
            report.Events.Add(EVENT_HUNGRY);
        }
        else if (hero.Hunger == VERY_HUNGRY)
        {
            report.Events.Add(EVENT_VERY_HUNGRY);
        }
        else if ((hero.Hunger & HUNGER_STEP_MASK) == 0)
        {
            return StepEffects(hero, body);
        }

        return report;
    }

    // The effects of a hunger step (every 8th point): damage and warnings while strong enough,
    // collapsing or fainting when weak.
    private static HungerReport StepEffects(Hero hero, Shape body)
    {
        List<int> events = [];
        bool redraw = false;

        if (body.Vitality > WEAK_VITALITY)
        {
            if (hero.Hunger > MUCH_HUNGER || hero.Fatigue > MUCH_FATIGUE)
            {
                body.Vitality -= 2;
                redraw = true;
            }

            if (hero.Hunger > STARVING_AT)
            {
                events.Add(EVENT_STARVING_WARNING);
            }
        }
        else if (hero.Fatigue > COLLAPSE_FATIGUE)
        {
            events.Add(EVENT_COLLAPSE);
            body.State = ActorState.SLEEP;
        }
        else if (hero.Hunger > FAINT_HUNGER)
        {
            events.Add(EVENT_FAINT);
            hero.Hunger = FAINT_RESET;
            body.State = ActorState.SLEEP;
        }

        return new HungerReport(events, redraw);
    }
}
