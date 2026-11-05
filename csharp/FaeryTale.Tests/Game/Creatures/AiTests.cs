using FaeryTale.Game.Creatures.Ai;
using FaeryTale.Game.Data;

namespace FaeryTale.Tests;

public class AiTests
{
    private const short ATTACK_MODE = 1;

    private static EnemyGoals.Situation Calm() => new(ActorState.STILL, 0, 10, 0, 0, 0);

    [Fact]
    public void EveryoneFleesWhenTheHeroIsDeadAndNobodyLeads()
    {
        Assert.Equal(ActorGoal.FLEE, EnemyGoals.AdjustedMode(1, Calm() with { HeroState = ActorState.DEAD }));
    }

    [Fact]
    public void EveryoneFollowsTheLeaderWhenTheHeroFalls()
    {
        Assert.Equal(ActorGoal.FOLLOWER, EnemyGoals.AdjustedMode(1, Calm() with { HeroState = ActorState.FALL, LeaderSlot = 3 }));
    }

    [Fact]
    public void TheNearlyDeadFleeAndTheHealthyKeepTheirMode()
    {
        Assert.Equal(ActorGoal.FLEE, EnemyGoals.AdjustedMode(1, Calm() with { Vitality = EnemyGoals.NEARLY_DEAD - 1 }));
        Assert.Equal(1, EnemyGoals.AdjustedMode(1, Calm() with { Vitality = EnemyGoals.NEARLY_DEAD }));
    }

    [Fact]
    public void StrangersInASpecialZoneFleeButTheZonesOwnRaceStays()
    {
        EnemyGoals.Situation zone = Calm() with { ExtentType = EnemyGoals.SPECIAL_XTYPE + 1, ZoneRace = 6 };

        Assert.Equal(ActorGoal.FLEE, EnemyGoals.AdjustedMode(1, zone with { Race = 5 }));
        Assert.Equal(1, EnemyGoals.AdjustedMode(1, zone with { Race = 6 }));
        Assert.Equal(1, EnemyGoals.AdjustedMode(1, zone with { Race = 5, ExtentType = EnemyGoals.SPECIAL_XTYPE }));
    }

    [Fact]
    public void ScreenRangeIsExclusive()
    {
        Assert.True(EnemyGoals.OnScreen(EnemyGoals.SCREEN_RANGE - 1, EnemyGoals.SCREEN_RANGE - 1));
        Assert.False(EnemyGoals.OnScreen(EnemyGoals.SCREEN_RANGE, 0));
    }

    private static EnemyTactics.TacticFacts Enemy(sbyte weapon, short vitality) => new(0, weapon, vitality, false);

    [Fact]
    public void AnUnarmedEnemyWandersAimlesslyAndAHealthySwordsmanPursues()
    {
        Assert.Equal((ActorGoal.CONFUSED, (short)ActorTactic.RANDOM), EnemyTactics.PickTactic(Enemy(0, 20), ATTACK_MODE, 0, 0, () => false));
        Assert.Equal((ATTACK_MODE, (short)ActorTactic.PURSUE), EnemyTactics.PickTactic(Enemy(1, 20), ATTACK_MODE, 0, 0, () => true));
    }

    [Fact]
    public void AWeakEnemyEvadesOnlyWhenTheCoinSaysSoAndAStrongOneNeverFlipsIt()
    {
        Assert.Equal(ActorTactic.EVADE, EnemyTactics.PickTactic(Enemy(1, EnemyTactics.WEAK - 1), ATTACK_MODE, 0, 0, () => true).Tactic);
        Assert.Equal(ActorTactic.PURSUE, EnemyTactics.PickTactic(Enemy(1, EnemyTactics.WEAK - 1), ATTACK_MODE, 0, 0, () => false).Tactic);

        EnemyTactics.PickTactic(Enemy(1, EnemyTactics.WEAK), ATTACK_MODE, 0, 0, () => throw new InvalidOperationException("coin flipped"));
    }

    [Fact]
    public void AnArcherBacksOffWhenCloseShootsInRangeElseClosesIn()
    {
        EnemyTactics.TacticFacts archer = Enemy(EnemyGoals.BOW_BIT, 20);

        Assert.Equal(ActorTactic.BACKUP, EnemyTactics.PickTactic(archer, ActorGoal.ARCHER1, 39, 29, () => false).Tactic);
        Assert.Equal(ActorTactic.SHOOT, EnemyTactics.PickTactic(archer, ActorGoal.ARCHER1, 40, 69, () => false).Tactic);
        Assert.Equal(ActorTactic.PURSUE, EnemyTactics.PickTactic(archer, ActorGoal.ARCHER1, 70, 70, () => false).Tactic);
    }

    [Fact]
    public void ASpiderGuardsTheEggsWhileTheTurtleHasThem()
    {
        EnemyTactics.TacticFacts spider = new(EnemyTactics.SPIDER_EGGS_RACE, 1, 20, true);

        Assert.Equal(10, EnemyTactics.PickTactic(spider, ATTACK_MODE, 0, 0, () => false).Tactic);
    }

    [Fact]
    public void StrikeRules()
    {
        short near = EnemyTactics.STRIKE_BASE - ATTACK_MODE;

        Assert.Equal(EnemyTactics.Hostile.Strike, EnemyTactics.HostileChoice(0, 1, 10, ATTACK_MODE, (short)(near - 1), (short)(near - 1)));
        Assert.Equal(EnemyTactics.Hostile.Tactic, EnemyTactics.HostileChoice(0, 1, 10, ATTACK_MODE, near, 5));

        // Bows never strike.
        Assert.Equal(EnemyTactics.Hostile.Tactic, EnemyTactics.HostileChoice(0, EnemyGoals.BOW_BIT, 10, ATTACK_MODE, 1, 1));

        short reach = EnemyTactics.DKNIGHT_STRIKE;
        Assert.Equal(EnemyTactics.Hostile.Strike, EnemyTactics.HostileChoice(EnemyTactics.DKNIGHT, 1, 10, ATTACK_MODE, (short)(reach - 1), (short)(reach - 1)));
        Assert.Equal(EnemyTactics.Hostile.KnightGuard, EnemyTactics.HostileChoice(EnemyTactics.DKNIGHT, 1, 10, ATTACK_MODE, reach, 5));
        Assert.Equal(EnemyTactics.Hostile.Tactic, EnemyTactics.HostileChoice(EnemyTactics.DKNIGHT, 1, 0, ATTACK_MODE, reach, 5));
    }
}
