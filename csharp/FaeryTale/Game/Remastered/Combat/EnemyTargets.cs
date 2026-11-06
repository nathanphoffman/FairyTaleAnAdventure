using FaeryTale.Game.Data;

namespace FaeryTale.Game.Remastered;

/// Finding the enemies under the mouse that can be attacked, and whether an enemy still can be.
public static class EnemyTargets
{
    // An enemy's picture is about 32 pixels wide and high, standing on the actor's position.
    private const int HALF_WIDTH = 14;
    private const int HEIGHT_ABOVE_FEET = 32;
    private const int DEPTH_BELOW_FEET = 8;

    /// The actor number of the enemy that can be attacked at a world position, if there is one.
    public static int? EnemyAt(Data.Game game, (int X, int Y) world)
    {
        for (int number = 1; number < game.Actors.Drawn; number++)
        {
            Shape enemy = game.Actors.List[number];

            if (IsAttackable(enemy) && PictureHolds(enemy, world))
            {
                return number;
            }
        }

        return null;
    }

    /// Whether an actor number is an enemy that is still alive (the slot may have been given to
    /// something else, or the enemy killed).
    public static bool IsAttackable(Data.Game game, int number) =>
        number > 0 && number < game.Actors.Drawn && IsAttackable(game.Actors.List[number]);

    /// Whether an actor is an enemy that is alive.
    public static bool IsAttackable(Shape actor) =>
        actor.Kind == ActorKind.ENEMY && actor.State != ActorState.DEAD && actor.State != ActorState.DYING && actor.Vitality > 0;

    // Whether a world position is inside the box of an enemy's picture.
    private static bool PictureHolds(Shape enemy, (int X, int Y) world) =>
        Math.Abs(world.X - enemy.AbsX) <= HALF_WIDTH
        && world.Y >= enemy.AbsY - HEIGHT_ABOVE_FEET
        && world.Y <= enemy.AbsY + DEPTH_BELOW_FEET;
}
