using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Land;

/// The survival tick: noting the safe spot, picking the music and applying hunger once a frame.
public static class Survival
{
    /// What the survival tick needs to know about the world.
    internal sealed record SurvivalFacts(ushort Clock, ushort Region, bool ActorsOnScreen, bool ActorsLoading, bool WitchOnScreen, bool Battle);

    /// What the survival tick wants done, in order.
    internal abstract record SurvivalStep
    {
        internal sealed record Event(int Message) : SurvivalStep;

        internal sealed record RedrawStats : SurvivalStep;

        internal sealed record PickMusic : SurvivalStep;
    }

    // The music mood is picked every 8th tick.
    private const ushort MOOD_PERIOD_MASK = 7;

    private const int FRUIT_SLOT = 24;
    private const short FRUIT_FOOD = 30;
    private const int EVENT_ATE_FRUIT = 37;
    internal const short TIRED_AT = 70;
    private const int EVENT_TIRED = 3;
    private const int EVENT_STARVING = 4;

    /// Notes a safe spot, picks the music, and makes the hero hungrier and more tired.
    public static void SurvivalTick(Data.Game game, IOutput output)
    {
        SurvivalFacts facts = new(
            game.World.DayClock,
            game.World.Region,
            game.Actors.OnScreen != 0,
            game.Actors.Loading != 0,
            game.Witch.OnScreen != 0,
            game.Actors.BattleFlag != 0);

        foreach (SurvivalStep step in SurvivalStepFor(facts, game.Hero, game.Actors.List[0]))
        {
            switch (step)
            {
                case SurvivalStep.Event message:
                    TextRoutines.ShowEvent(game, output, message.Message);
                    break;

                case SurvivalStep.RedrawStats:
                    TextRoutines.QueuePrint(game.Ui, PrintRequest.VITALITY);
                    break;

                case SurvivalStep.PickMusic:
                    Music.SetMood(game, 0);
                    break;
            }
        }
    }

    internal static bool MoodDue(ushort clock, bool battle) => (clock & MOOD_PERIOD_MASK) == 0 && !battle;

    /// One frame of the survival rules: remembers a safe spot (eating fruit if hungry), then picks
    /// the music, then makes the hero hungrier and more tired. body is the hero's actor.
    internal static List<SurvivalStep> SurvivalStepFor(SurvivalFacts facts, Hero hero, Shape body)
    {
        List<SurvivalStep> steps = [];

        if (Hunger.SafeCheckDue(facts.Clock) && IsSafePlace(facts, hero, body))
        {
            NoteSafeSpot(facts, hero, steps);
        }

        if (MoodDue(facts.Clock, facts.Battle))
        {
            steps.Add(new SurvivalStep.PickMusic());
        }

        if (Hunger.HungerDue(facts.Clock, body.Vitality, body.State))
        {
            AddHunger(hero, body, steps);
        }

        return steps;
    }

    // A safe spot can be noted when nothing is around, the hero is on dry land and alive.
    private static bool IsSafePlace(SurvivalFacts facts, Hero hero, Shape body) =>
        !facts.ActorsOnScreen
        && !facts.ActorsLoading
        && !facts.WitchOnScreen
        && body.Environ == 0
        && hero.SafeFlag == 0
        && body.State != ActorState.DEAD;

    // Remembers where the hero stands as the safe spot, and eats fruit if hungry enough.
    private static void NoteSafeSpot(SurvivalFacts facts, Hero hero, List<SurvivalStep> steps)
    {
        hero.SafeR = facts.Region;
        hero.SafeX = hero.X;
        hero.SafeY = hero.Y;

        if (Hunger.EatsFruit(hero.Hunger, hero.Stuff[FRUIT_SLOT]))
        {
            hero.Stuff[FRUIT_SLOT]--;
            hero.Hunger -= FRUIT_FOOD;
            steps.Add(new SurvivalStep.Event(EVENT_ATE_FRUIT));
        }
    }

    // One more point of hunger and tiredness, with what that does and the warnings that come with it.
    private static void AddHunger(Hero hero, Shape body, List<SurvivalStep> steps)
    {
        hero.Hunger++;
        hero.Fatigue++;
        Hunger.HungerReport report = Hunger.HungerEffects(hero, body);

        if (report.RedrawStats)
        {
            steps.Add(new SurvivalStep.RedrawStats());
        }

        steps.AddRange(report.Events.Select(message => new SurvivalStep.Event(message)));

        if (hero.Fatigue == TIRED_AT)
        {
            steps.Add(new SurvivalStep.Event(EVENT_TIRED));
        }
        else if (hero.Hunger == Hunger.STARVING_AT)
        {
            steps.Add(new SurvivalStep.Event(EVENT_STARVING));
        }
    }
}
