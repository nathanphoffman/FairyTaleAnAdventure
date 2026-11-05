using FaeryTale.Game.Data;

namespace FaeryTale.Game.Creatures.Interaction;

/// Carrying out an enemy's tactic: pursuing, following, evading, wandering, shooting or backing up.
public static class Tactics
{
    // How an enemy shooting at the hero lines up.
    private const short SHOOT_CLOSE = 8;
    private const short SHOOT_DIAGONAL_BELOW = 5;
    private const short SHOOT_DIAGONAL_ABOVE = 7;

    // SteerActor modes.
    private const int COURSE_TOWARD = 0;
    private const int COURSE_WOBBLE_CLOSE = 2;
    private const int COURSE_AWAY = 3;
    private const int COURSE_STRAIGHT = 4;
    private const int COURSE_TURN_ONLY = 5;

    // An actor picks a new heading one frame in 8 (one in 4 for those that attack harder).
    private const int ACTS_ONE_IN_8 = 7;
    private const int ACTS_ONE_IN_4 = 3;
    private const int FACING_MASK = 7;

    /// The followers aim this far below the one they follow.
    private const int FOLLOW_OFFSET = 20;

    // Where snakes go to find the eggs.
    private const int EGG_X = 23087;
    private const int EGG_Y = 5667;

    /// Runs one step of enemy index's tactic. Most tactics only act on some ticks (1 in 8, or 1 in
    /// 4 for clever enemies), which makes them look less robotic.
    ///
    /// - PURSUE: head toward the hero. - SHOOT: line up with the hero (same row, column or
    ///   diagonal) and aim, else approach. - RANDOM: wander. - BUMBLE_SEEK: head toward the hero
    ///   without smart steering. - BACKUP: head away from the hero. - FOLLOW: stay near the
    ///   leader. - EVADE: keep away from another enemy. - EGG_SEEK: go to the turtle eggs.
    public static void RunTactic(Data.Game game, int index, int tactic)
    {
        bool actsNow = (game.Rng.Next() & ACTS_ONE_IN_8) == 0;
        Shape actor = game.Actors.List[index];

        actor.Tactic = (sbyte)tactic;

        if (actor.Goal == ActorGoal.ATTACK2)
        {
            actsNow = (game.Rng.Next() & ACTS_ONE_IN_4) == 0;
        }

        int heroX = game.Hero.X;
        int heroY = game.Hero.Y;

        switch (tactic)
        {
            case ActorTactic.PURSUE:
                SteerIf(game, actsNow, index, heroX, heroY, COURSE_TOWARD);
                break;

            case ActorTactic.SHOOT:
                EngageTarget(game, index, actor, heroX, heroY);
                break;

            case ActorTactic.RANDOM:
                Wander(game, actor, actsNow);
                break;

            case ActorTactic.BUMBLE_SEEK:
                SteerIf(game, actsNow, index, heroX, heroY, COURSE_STRAIGHT);
                break;

            case ActorTactic.BACKUP:
                SteerIf(game, actsNow, index, heroX, heroY, COURSE_AWAY);
                break;

            case ActorTactic.FOLLOW:
                FollowLeader(game, index, actor, actsNow);
                break;

            case ActorTactic.EVADE:
                EvadeNeighbour(game, index, actsNow);
                break;

            case ActorTactic.EGG_SEEK:
                SteerIf(game, actsNow, index, EGG_X, EGG_Y, COURSE_TOWARD);
                actor.State = ActorState.WALKING;
                break;
        }
    }

    // Steers actor index toward a point (or however mode says) if this is one of its active ticks.
    private static void SteerIf(Data.Game game, bool actsNow, int index, int x, int y, int mode)
    {
        if (actsNow)
        {
            Steering.SteerActor(game, index, x, y, mode);
        }
    }

    // An archer lines up with the hero (same row, column or diagonal) and aims, otherwise approaches.
    private static void EngageTarget(Data.Game game, int index, Shape actor, int heroX, int heroY)
    {
        Shape hero = game.Actors.List[0];
        short xDifference = (short)Math.Abs((int)(short)(actor.AbsX - hero.AbsX)); // wraps like the original at -32768
        short yDifference = (short)Math.Abs((int)(short)(actor.AbsY - hero.AbsY));
        bool linedUp = xDifference < SHOOT_CLOSE
            || yDifference < SHOOT_CLOSE
            || (xDifference > yDifference - SHOOT_DIAGONAL_BELOW && xDifference < yDifference + SHOOT_DIAGONAL_ABOVE);

        if ((game.Rng.Next() & 1) != 0 && linedUp)
        {
            Steering.SteerActor(game, index, heroX, heroY, COURSE_TURN_ONLY);

            if (actor.State < ActorState.SHOOT1)
            {
                actor.State = ActorState.SHOOT1;
            }
        }
        else
        {
            Steering.SteerActor(game, index, heroX, heroY, COURSE_TOWARD);
        }
    }

    // Wander: now and then turn a random way, and keep walking.
    private static void Wander(Data.Game game, Shape actor, bool actsNow)
    {
        if (actsNow)
        {
            actor.Facing = (sbyte)(game.Rng.Next() & FACING_MASK);
        }

        actor.State = ActorState.WALKING;
    }

    // Stay near the leader (an enemy cannot follow itself, so it wanders instead).
    private static void FollowLeader(Data.Game game, int index, Shape actor, bool actsNow)
    {
        int leader = game.Actors.Leader;

        if (index == leader)
        {
            actor.Tactic = ActorTactic.RANDOM;
        }

        if (actsNow)
        {
            SteerTowardActor(game, index, leader, COURSE_TOWARD);
        }
    }

    // Keep away from another enemy (the one before the last, or the one at twice this slot number).
    private static void EvadeNeighbour(Data.Game game, int index, bool actsNow)
    {
        int other = index == game.Actors.Active ? index - 1 : index + index;

        if (actsNow)
        {
            SteerTowardActor(game, index, other, COURSE_WOBBLE_CLOSE);
        }
    }

    // Steers actor index toward (a little below) actor target.
    private static void SteerTowardActor(Data.Game game, int index, int target, int mode)
    {
        Shape other = game.Actors.List[target];

        Steering.SteerActor(game, index, other.AbsX, other.AbsY + FOLLOW_OFFSET, mode);
    }
}
