using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;

namespace FaeryTale.Game.Remastered;

/// Click to attack (Remastered and Reimagined): a click on an enemy makes the hero walk up to it
/// and fight it until it is dead, another click on an enemy changes who he fights, and any other
/// click or the keys end it. The hero uses the normal fighting rules: while the order stands, the
/// hero turns towards the enemy and holds the fight button down.
public static class ClickToAttack
{
    // How close the hero stands to an enemy to fight it by hand, and how far away he shoots a bow or
    // a wand (which fly straight along the way he faces).
    private const int MELEE_DISTANCE = 18;
    private const int RANGED_DISTANCE = 120;

    // The route to a moving enemy is worked out again this often (in frames).
    private const int REPLAN_FRAMES = 12;

    private const sbyte WEAPON_BOW = 4;
    private const sbyte WEAPON_WAND = 5;

    // Where the middle of an enemy is above its feet.
    private const int BODY_HEIGHT = 14;

    private static int? target;
    private static int frames;

    /// Whether an enemy is being attacked.
    public static bool Active => target is not null;

    /// Orders an attack on an enemy (an actor number).
    public static void Order(int enemy)
    {
        target = enemy;
        frames = 0;
    }

    /// Ends the order.
    public static void Clear() => target = null;

    /// The world position of the middle of the enemy being attacked, to draw the target on.
    public static (int X, int Y)? TargetAt(Data.Game game)
    {
        if (target is not { } number || !EnemyTargets.IsAttackable(game, number))
        {
            return null;
        }

        Shape enemy = game.Actors.List[number];

        return (enemy.AbsX, enemy.AbsY - BODY_HEIGHT);
    }

    /// Called every frame. Ends the order when the enemy is dead or the player takes over; fights
    /// when the enemy is in reach (returns true: the hero is busy); otherwise keeps a route to the
    /// enemy so that the walk goes on.
    public static bool Update(Data.Game game)
    {
        if (target is not { } number)
        {
            return false;
        }

        if (!EnemyTargets.IsAttackable(game, number) || ClickToMove.PlayerSteers(game))
        {
            Clear();
            ClickToMove.Cancel();

            return false;
        }

        Shape hero = game.Actors.List[0];
        Shape enemy = game.Actors.List[number];

        if (Blocking.ActorDistance(game, number, 0) <= ReachOf(hero))
        {
            Fight(game, hero, enemy);

            return true;
        }

        Approach(game, enemy);

        return false;
    }

    // How far the hero's weapon takes him from an enemy.
    private static int ReachOf(Shape hero) => hero.Weapon is WEAPON_BOW or WEAPON_WAND ? RANGED_DISTANCE : MELEE_DISTANCE;

    // Stands and fights, turned towards the enemy.
    private static void Fight(Data.Game game, Shape hero, Shape enemy)
    {
        short direction = ClickToMove.DirectionTo(enemy.AbsX - hero.AbsX, enemy.AbsY - hero.AbsY);

        ClickToMove.Cancel();

        if (direction < ClickToMove.NOT_WALKING)
        {
            game.Ui.PathDirection = direction;
            game.Ui.CompassLitDirection = direction;
        }

        game.Ui.PathFighting = true;
    }

    // Keeps a route to the enemy, worked out again now and then as the enemy moves.
    private static void Approach(Data.Game game, Shape enemy)
    {
        if (!ClickToMove.Active || frames++ % REPLAN_FRAMES == 0)
        {
            ClickToMove.RouteTo(game, (enemy.AbsX, enemy.AbsY));
        }
    }
}
