using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class QueuedUseAndHealthBarTests
{
    private const int SWORD_SLOT = 2;
    private const int DIRK_SLOT = 0;
    private const int SWORD_WEAPON = 3;
    private const uint NOW = 1000;

    [Fact]
    public void QueuedWeaponIsTakenInHandWhenRun()
    {
        FaeryTale.Game.Data.Game game = new() { Experience = Experience.Remastered };

        game.Hero.Stuff[SWORD_SLOT] = 1;
        game.Hero.Stuff[DIRK_SLOT] = 1;
        QueuedUse.Toggle(DIRK_SLOT, false);
        QueuedUse.Toggle(SWORD_SLOT, false);

        Assert.True(QueuedUse.Contains(SWORD_SLOT));
        Assert.False(QueuedUse.Contains(DIRK_SLOT));

        QueuedUse.Run(game, new Recorder());

        Assert.Equal(SWORD_WEAPON, game.Actors.List[0].Weapon);
        Assert.False(QueuedUse.Contains(SWORD_SLOT));
    }

    [Fact]
    public void ClickingAQueuedItemAgainTakesItOut()
    {
        QueuedUse.Toggle(SWORD_SLOT, false);
        QueuedUse.Toggle(SWORD_SLOT, false);

        Assert.False(QueuedUse.Contains(SWORD_SLOT));
    }

    [Fact]
    public void HealthBarShowsOnlyAfterAHitAndGoesWhenEnemyDies()
    {
        FaeryTale.Game.Data.Game game = new() { Experience = Experience.Remastered };
        Shape enemy = game.Actors.List[1];

        enemy.Kind = ActorKind.ENEMY;
        enemy.State = ActorState.WALKING;
        enemy.Vitality = 20;
        game.Actors.Drawn = 2;
        EnemyHealthBars.Enabled = true;

        Assert.Empty(EnemyHealthBars.Update(game, NOW));

        enemy.Vitality = 5;
        List<EnemyHealthBars.Bar> bars = EnemyHealthBars.Update(game, NOW + 1);

        Assert.Equal(0.25, Assert.Single(bars).Fraction);

        enemy.Vitality = 0;

        Assert.Empty(EnemyHealthBars.Update(game, NOW + 2));
    }
}

public class ClickToShootTests
{
    [Theory]
    [InlineData(50, 0, 0, 3)]
    [InlineData(-50, 2, 2, 7)]
    [InlineData(0, 40, 0, 5)]
    [InlineData(30, 30, 0, 4)]
    [InlineData(-30, 30, 0, 6)]
    public void NearestLineFindsTheDirectionAndGap(int dx, int dy, int offset, int direction)
    {
        (double gap, short facing) = ClickToShoot.NearestLine(dx, dy);

        Assert.Equal(offset, gap, 3);
        Assert.Equal(direction, facing);
    }
}
