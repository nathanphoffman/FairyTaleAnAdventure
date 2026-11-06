using FaeryTale.Game.Data;
using FaeryTale.Game.Ui;

namespace FaeryTale.Tests;

public class SearchBodyTests
{
    private static Shape Body(sbyte kind = ActorKind.ENEMY)
    {
        Shape body = new();

        (body.Kind, body.State, body.Vitality, body.Weapon) = (kind, ActorState.DEAD, 0, 2);

        return body;
    }

    [Fact]
    public void ADeadBodyThatHasNotBeenSearchedCanBeTaken()
    {
        Assert.True(TakeItems.CanTake(Body()));
        Assert.True(TakeItems.CanTake(Body(ActorKind.SETFIG)));
    }

    [Fact]
    public void ASearchedBodyCannotBeTakenAgain()
    {
        Shape body = Body();

        // Searching sets the weapon to -1.
        body.Weapon = -1;

        Assert.False(TakeItems.CanTake(body));
    }

    [Fact]
    public void ALivingOrDyingEnemyIsNotABody()
    {
        Shape enemy = Body();

        enemy.State = ActorState.WALKING;
        Assert.False(TakeItems.CanTake(enemy));

        enemy.State = ActorState.DYING;
        Assert.False(TakeItems.CanTake(enemy));
    }
}
