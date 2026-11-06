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

    // A bow shoots when the fight is let go, so the fight is held while the bow is drawn and let go for
    // a moment to loose the arrow, over and over. A wand fires when the fight starts, so it is let go
    // for a while between shots. (Frames: each is a fiftieth of a second.)
    private const int BOW_PERIOD = 16;
    private const int BOW_HOLD = 14;
    private const int WAND_PERIOD = 14;
    private const int WAND_HOLD = 2;

    // Whether a shot has a clear line is looked at every this many pixels, leaving out the last few
    // next to the enemy (who stands in the way of himself).
    private const int SIGHT_STEP = 8;
    private const int SIGHT_END_MARGIN = 12;
    private const int ARROW_SLOT = 8;

    private const sbyte WEAPON_BOW = 4;
    private const sbyte WEAPON_WAND = 5;

    // Where the middle of an enemy is above its feet.
    private const int BODY_HEIGHT = 14;

    private static int? target;
    private static int frames;
    private static int shotFrames;

    /// Whether an enemy is being attacked.
    public static bool Active => target is not null;

    /// Orders an attack on an enemy (an actor number).
    public static void Order(int enemy)
    {
        target = enemy;
        frames = 0;
        shotFrames = 0;
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

        bool ranged = IsRanged(hero);

        if (ranged && hero.Weapon == WEAPON_BOW && game.Hero.Stuff[ARROW_SLOT] == 0)
        {
            Clear();
            ClickToMove.Cancel();

            return false;
        }

        if (Blocking.ActorDistance(game, number, 0) <= ReachOf(hero) && (!ranged || HasClearShot(game, hero, enemy)))
        {
            Fight(game, hero, enemy);

            return true;
        }

        Approach(game, enemy);

        return false;
    }

    // A bow or a wand shoots, the rest of the weapons are swung.
    private static bool IsRanged(Shape hero) => hero.Weapon is WEAPON_BOW or WEAPON_WAND;

    // How far the hero's weapon takes him from an enemy.
    private static int ReachOf(Shape hero) => IsRanged(hero) ? RANGED_DISTANCE : MELEE_DISTANCE;

    /// Whether the fight is held down at a frame of a run of shots (a frame count from the first). A
    /// weapon that is swung is held all the time.
    internal static bool ShotHeld(sbyte weapon, int frame) => weapon switch
    {
        WEAPON_BOW => frame % BOW_PERIOD < BOW_HOLD,
        WEAPON_WAND => frame % WAND_PERIOD < WAND_HOLD,
        _ => true,
    };

    // Whether nothing solid lies between the hero and the enemy (a shot goes in a straight line).
    private static bool HasClearShot(Data.Game game, Shape hero, Shape enemy)
    {
        int dx = enemy.AbsX - hero.AbsX;
        int dy = enemy.AbsY - hero.AbsY;
        int steps = (Math.Max(Math.Abs(dx), Math.Abs(dy)) - SIGHT_END_MARGIN) / SIGHT_STEP;

        for (int step = 1; step <= steps; step++)
        {
            int x = hero.AbsX + dx * step * SIGHT_STEP / Math.Max(Math.Abs(dx), Math.Abs(dy));
            int y = hero.AbsY + dy * step * SIGHT_STEP / Math.Max(Math.Abs(dx), Math.Abs(dy));

            if (Blocking.BlockedAt(game, (short)x, (short)y, 0) != 0)
            {
                return false;
            }
        }

        return true;
    }

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

        game.Ui.PathFighting = ShotHeld(hero.Weapon, shotFrames++);
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
