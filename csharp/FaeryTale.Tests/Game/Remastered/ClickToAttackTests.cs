using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class ClickToAttackTests
{
    // A game with one enemy standing at a place (actor 1).
    private static FaeryTale.Game.Data.Game GameWithEnemyAt(int x, int y)
    {
        FaeryTale.Game.Data.Game game = new();
        Shape enemy = game.Actors.List[1];

        (enemy.Kind, enemy.State, enemy.Vitality, enemy.AbsX, enemy.AbsY) = (ActorKind.ENEMY, ActorState.WALKING, 10, (ushort)x, (ushort)y);
        game.Actors.Drawn = 2;

        return game;
    }

    [Fact]
    public void AnEnemyIsFoundWhereItsPictureIs()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyAt(1000, 2000);

        Assert.Equal(1, EnemyTargets.EnemyAt(game, (1005, 1975)));
        Assert.Null(EnemyTargets.EnemyAt(game, (1100, 1975)));
    }

    [Fact]
    public void ADeadOrDyingEnemyCannotBeAttacked()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyAt(1000, 2000);

        game.Actors.List[1].State = ActorState.DYING;
        Assert.False(EnemyTargets.IsAttackable(game, 1));

        game.Actors.List[1].State = ActorState.WALKING;
        game.Actors.List[1].Vitality = 0;
        Assert.False(EnemyTargets.IsAttackable(game, 1));
    }

    [Fact]
    public void AnOrderEndsWhenTheEnemyIsGone()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyAt(1000, 2000);

        ClickToAttack.Order(1);
        game.Actors.List[1].Vitality = 0;

        Assert.False(ClickToAttack.Update(game));
        Assert.False(ClickToAttack.Active);
    }

    [Fact]
    public void AHeroNextToTheEnemyFightsFacingIt()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyAt(1010, 1000);

        game.Actors.List[0].AbsX = 1000;
        game.Actors.List[0].AbsY = 1000;
        ClickToAttack.Order(1);

        Assert.True(ClickToAttack.Update(game));
        Assert.True(game.Ui.PathFighting);
        Assert.Equal(3, game.Ui.PathDirection);
        ClickToAttack.Clear();
    }
}
