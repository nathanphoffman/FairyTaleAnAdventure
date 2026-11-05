using FaeryTale.Game.Data;
using FaeryTale.Game.Land;

namespace FaeryTale.Tests;

public class SurvivalTests
{
    private const sbyte STILL = ActorState.STILL;

    private static (Hero Hero, Shape Body) HeroWithBody(short hunger, short fatigue, short vitality) =>
        (new Hero { Hunger = hunger, Fatigue = fatigue }, new Shape { Vitality = vitality });

    // A hero with an empty pack, standing in a calm place on a safe-check frame.
    private static (Survival.SurvivalFacts Facts, Hero Hero, Shape Body) CalmHero() =>
        (new Survival.SurvivalFacts(128, 3, false, false, false, true),
            new Hero { Brother = 1, X = 1234, Y = 5678 },
            new Shape { Vitality = 50, State = STILL });

    [Fact]
    public void HungerMessagesAtTheFirstTwoThresholds()
    {
        (Hero hero, Shape body) = HeroWithBody(Hunger.HUNGRY, 0, 50);
        Assert.Equal([0], Hunger.HungerEffects(hero, body).Events);

        (hero, body) = HeroWithBody(60, 0, 50);
        Assert.Equal([1], Hunger.HungerEffects(hero, body).Events);
    }

    [Fact]
    public void NothingHappensBetweenEightsWhenOnlyALittleHungry()
    {
        (Hero hero, Shape body) = HeroWithBody(41, 0, 50);
        Hunger.HungerReport report = Hunger.HungerEffects(hero, body);

        Assert.Empty(report.Events);
        Assert.False(report.RedrawStats);
        Assert.Equal(50, body.Vitality);
    }

    [Fact]
    public void ATiredHeroLosesVitalityWithoutAMessageUntilStarving()
    {
        (Hero hero, Shape body) = HeroWithBody(88, Hunger.MUCH_FATIGUE + 1, 50);
        Hunger.HungerReport report = Hunger.HungerEffects(hero, body);

        Assert.Equal((48, true), (body.Vitality, report.RedrawStats));
        Assert.Empty(report.Events);

        (hero, body) = HeroWithBody(96, Hunger.MUCH_FATIGUE + 1, 50);
        report = Hunger.HungerEffects(hero, body);
        Assert.Equal(48, body.Vitality);
        Assert.Equal([2], report.Events);
    }

    [Fact]
    public void AWeakExhaustedHeroCollapsesAsleep()
    {
        (Hero hero, Shape body) = HeroWithBody(80, Hunger.COLLAPSE_FATIGUE + 1, Hunger.WEAK_VITALITY);

        Assert.Equal([12], Hunger.HungerEffects(hero, body).Events);
        Assert.Equal(ActorState.SLEEP, body.State);
    }

    [Fact]
    public void AWeakStarvingHeroFaintsResettingHunger()
    {
        (Hero hero, Shape body) = HeroWithBody(Hunger.FAINT_HUNGER + 8 - Hunger.FAINT_HUNGER % 8, 0, Hunger.WEAK_VITALITY);

        Assert.Equal([24], Hunger.HungerEffects(hero, body).Events);
        Assert.Equal((ActorState.SLEEP, Hunger.FAINT_RESET), (body.State, hero.Hunger));
    }

    [Fact]
    public void ACalmFrameNotesTheSafeSpotAddingHunger()
    {
        (Survival.SurvivalFacts facts, Hero hero, Shape body) = CalmHero();
        var steps = Survival.SurvivalStepFor(facts, hero, body);

        Assert.Equal(((ushort)3, (ushort)1234, (ushort)5678), (hero.SafeR, hero.SafeX, hero.SafeY));
        Assert.Equal(((short)1, (short)1), (hero.Hunger, hero.Fatigue));
        Assert.Empty(steps);
    }

    [Fact]
    public void DangerStopsTheSafeSpotButNotTheHunger()
    {
        for (int danger = 0; danger < 4; danger++)
        {
            (Survival.SurvivalFacts facts, Hero hero, Shape body) = CalmHero();

            facts = danger switch
            {
                0 => facts with { ActorsOnScreen = true },
                1 => facts with { ActorsLoading = true },
                2 => facts with { WitchOnScreen = true },
                _ => facts,
            };

            if (danger == 3)
            {
                hero.SafeFlag = 1;
            }

            Survival.SurvivalStepFor(facts, hero, body);
            Assert.Equal(0, hero.SafeR);
            Assert.Equal(1, hero.Hunger);
        }
    }

    [Fact]
    public void AHungryHeroEatsFruitAtTheSafeSpot()
    {
        (Survival.SurvivalFacts facts, Hero hero, Shape body) = CalmHero();
        hero.Stuff[24] = 2;
        hero.Hunger = 40;

        var steps = Survival.SurvivalStepFor(facts, hero, body);

        Assert.Equal([new Survival.SurvivalStep.Event(37)], steps);
        Assert.Equal(40 - 30 + 1, hero.Hunger);
        Assert.Equal(1, hero.Stuff[24]);
    }

    [Fact]
    public void MusicIsPickedEveryEighthFrameOutsideBattle()
    {
        (Survival.SurvivalFacts facts, Hero hero, Shape body) = CalmHero();

        Assert.Equal([new Survival.SurvivalStep.PickMusic()], Survival.SurvivalStepFor(facts with { Clock = 8, Battle = false }, hero, body));
        Assert.Empty(Survival.SurvivalStepFor(facts with { Clock = 8, Battle = true }, hero, body));
    }

    [Fact]
    public void HungerMessagesComeInTheOriginalOrder()
    {
        (Survival.SurvivalFacts facts, Hero hero, Shape body) = CalmHero();
        facts = facts with { Clock = 0 };
        hero.Hunger = Hunger.HUNGRY - 1;

        Assert.Equal([new Survival.SurvivalStep.Event(0)], Survival.SurvivalStepFor(facts, hero, body));

        // Damage is shown (stats) before the starving messages, and the tired one comes last.
        hero.Hunger = 103;
        hero.Fatigue = Survival.TIRED_AT - 1;

        Assert.Equal(
            [new Survival.SurvivalStep.RedrawStats(), new Survival.SurvivalStep.Event(2), new Survival.SurvivalStep.Event(3)],
            Survival.SurvivalStepFor(facts, hero, body));
        Assert.Equal(48, body.Vitality);
    }

    [Fact]
    public void NoHungerWhileAsleepOrDead()
    {
        (Survival.SurvivalFacts facts, Hero hero, Shape body) = CalmHero();

        body.State = ActorState.SLEEP;
        Survival.SurvivalStepFor(facts, hero, body);
        Assert.Equal(0, hero.Hunger);

        body.State = STILL;
        body.Vitality = 0;
        Survival.SurvivalStepFor(facts, hero, body);
        Assert.Equal(0, hero.Hunger);
    }

    [Fact]
    public void SurvivalSchedule()
    {
        Assert.Equal([0, 128, 256, 384], Enumerable.Range(0, 385).Where(c => Hunger.HungerDue((ushort)c, 10, STILL)));
        Assert.False(Hunger.HungerDue(128, 0, STILL));
        Assert.False(Hunger.HungerDue(128, 10, ActorState.SLEEP));
        Assert.True(Survival.MoodDue(8, false) && Survival.MoodDue(0, false));
        Assert.False(Survival.MoodDue(9, false) || Survival.MoodDue(8, true));
        Assert.True(Hunger.EatsFruit(31, 1) && !Hunger.EatsFruit(30, 1) && !Hunger.EatsFruit(50, 0));
    }
}
