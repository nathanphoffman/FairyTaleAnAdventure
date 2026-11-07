using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Reimagined;

namespace FaeryTale.Tests;

public class InfernoSkullTests
{
    private const int HERO_SPOT = 1000;

    // The hero faces east (direction 3) with an enemy of the given distance ahead of him.
    private static FaeryTale.Game.Data.Game GameWithEnemyAhead(int enemyDistance)
    {
        FaeryTale.Game.Data.Game game = new() { Experience = Experience.Reimagined };

        game.Actors.List[0].AbsX = HERO_SPOT;
        game.Actors.List[0].AbsY = HERO_SPOT;
        game.Actors.List[0].Facing = 3;
        game.Actors.List[1].AbsX = (ushort)(HERO_SPOT + enemyDistance);
        game.Actors.List[1].AbsY = HERO_SPOT - 10;
        game.Actors.List[1].Kind = ActorKind.ENEMY;
        game.Actors.List[1].State = ActorState.WALKING;
        game.Actors.List[1].Vitality = 20;
        game.Actors.Active = 2;

        return game;
    }

    [Fact]
    public void TheFireballBurstsOnAnEnemyAndLeavesFlames()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyAhead(40);

        InfernoSkull.Cast(game);

        for (int frame = 0; frame < 10 && game.Inferno.Fireball is not null; frame++)
        {
            InfernoSkull.Advance(game, new Recorder());
        }

        Assert.Null(game.Inferno.Fireball);
        Assert.NotEmpty(game.Inferno.Flames);
    }

    [Fact]
    public void TheFlamesGoOutAfterAWhile()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyAhead(40);

        InfernoSkull.Cast(game);

        for (int frame = 0; frame < 100; frame++)
        {
            InfernoSkull.Advance(game, new Recorder());
        }

        Assert.False(game.Inferno.Active);
    }

    [Fact]
    public void ABurstAlsoHurtsEnemiesNearbyThatItDidNotTouch()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyAhead(40);

        game.Actors.List[2].AbsX = HERO_SPOT + 60;
        game.Actors.List[2].AbsY = HERO_SPOT + 20;
        game.Actors.List[2].Kind = ActorKind.ENEMY;
        game.Actors.List[2].State = ActorState.WALKING;
        game.Actors.List[2].Vitality = 100;
        game.Actors.Active = 3;
        InfernoSkull.Cast(game);

        for (int frame = 0; frame < 10 && game.Inferno.Fireball is not null; frame++)
        {
            InfernoSkull.Advance(game, new Recorder());
        }

        Assert.True(game.Actors.List[2].Vitality < 100);
    }
}
