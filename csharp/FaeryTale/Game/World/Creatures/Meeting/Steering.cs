using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Game.Creatures;

/// Steering an actor toward or away from a target (the enemy-steering routine of the 68000
/// assembly): turn the actor and start it walking.
public static class Steering
{
    // Steering modes.
    public const int STEER_STRAIGHT = 4;
    public const int STEER_TURN_ONLY = 5;
    public const int STEER_OFFSET = 6;
    private const int STEER_WOBBLE_FAR = 1;
    private const int STEER_WOBBLE_NEAR = 2;
    private const int STEER_AWAY = 3;

    // The wobble starts when the target is closer than this (in mode 1 and in mode 2).
    private const short WOBBLE_DISTANCE_FAR = 40;
    private const short WOBBLE_DISTANCE_NEAR = 30;

    // The steering mode is the low byte of the argument; directions are 0-7.
    private const int MODE_MASK = 0xff;
    private const int FACING_MASK = 7;

    // The table is indexed by 4 - 3 * (vertical sign) - (horizontal sign).
    private const int TABLE_CENTER = 4;
    private const int TABLE_ROW_WIDTH = 3;

    // An axis is ignored when it is less than half the other (a shift of one).
    private const int IGNORE_SHIFT = 1;

    /// Direction number for each of the 9 combinations of horizontal and vertical steering.
    private static readonly byte[] DIRECTION_TABLE = [0, 1, 2, 7, 9, 3, 6, 5, 4];
    private const int NO_DIRECTION = 9;

    /// What the steering decides for an actor: stop, or face a direction.
    internal readonly record struct Course(bool Stop, int Facing);

    /// Points an actor toward (or away from) a target and sets it walking (see Steer for modes).
    /// The random number is only drawn when the actor moves, as in the original, so recorded games
    /// stay in step.
    public static void SteerActor(Data.Game game, int objectSlot, int targetX, int targetY, int mode)
    {
        mode &= MODE_MASK;
        Shape actor = game.Actors.List[objectSlot];

        // Remastered: an enemy whose straight way is blocked is aimed at the next point of a route.
        if (mode <= STEER_WOBBLE_NEAR)
        {
            (targetX, targetY) = EnemyPathing.AimPoint(game, objectSlot, (targetX, targetY));
        }

        short xDifference = mode == STEER_OFFSET ? (short)targetX : (short)(actor.AbsX - targetX);
        short yDifference = mode == STEER_OFFSET ? (short)targetY : (short)(actor.AbsY - targetY);

        Course course = Steer(xDifference, yDifference, mode, () => (game.Rng.Next() & 2) != 0);

        if (course.Stop)
        {
            actor.State = ActorState.STILL;

            return;
        }

        actor.Facing = (sbyte)course.Facing;

        if (mode != STEER_TURN_ONLY)
        {
            actor.State = ActorState.WALKING;
        }
    }

    /// Works out the direction toward (or away from) a target. xDifference/yDifference are actor
    /// minus target (or the offset itself in mode 6). Modes: 0 toward, 1 and 2 toward with a
    /// sideways wobble when close, 3 away, 4 straight toward (no smart steering), 5 turn only,
    /// 6 the offset is given.
    ///
    /// Method: take the sign of each axis, then ignore an axis if the other is more than twice as
    /// far (so mostly-horizontal targets give a pure left/right move, not a diagonal). The two
    /// signs pick one of 8 directions from a table; if both are zero the actor is on the target
    /// and stops. The wobble adds or subtracts one direction step at random when the target is close.
    internal static Course Steer(short xDifference, short yDifference, int mode, Func<bool> randomBit)
    {
        ushort xAbs = (ushort)Math.Abs((int)xDifference);
        ushort yAbs = (ushort)Math.Abs((int)yDifference);
        int xDirection = Math.Sign(xDifference);
        int yDirection = Math.Sign(yDifference);

        if (mode != STEER_STRAIGHT)
        {
            (xDirection, yDirection) = AlongTheLongAxis(xDirection, yDirection, xAbs, yAbs);
        }

        int deviation = Wobble(mode, (short)(xAbs + yAbs));

        if (mode == STEER_AWAY)
        {
            xDirection = -xDirection;
            yDirection = -yDirection;
        }

        int facing = DIRECTION_TABLE[TABLE_CENTER - yDirection * TABLE_ROW_WIDTH - xDirection];

        if (facing == NO_DIRECTION)
        {
            return new Course(true, 0);
        }

        facing += randomBit() ? deviation : -deviation;

        return new Course(false, facing & FACING_MASK);
    }

    // An axis less than half as far as the other is ignored, so the actor goes straight along the
    // long one. Compared as signed values: the shifted magnitude is at most half the other axis.
    private static (int X, int Y) AlongTheLongAxis(int xDirection, int yDirection, ushort xAbs, ushort yAbs)
    {
        int y = (short)yAbs < (short)(xAbs >> IGNORE_SHIFT) ? 0 : yDirection;
        int x = (short)xAbs < (short)(yAbs >> IGNORE_SHIFT) ? 0 : xDirection;

        return (x, y);
    }

    // The sideways wobble (one direction step) of modes 1 and 2 when the target is close, else 0.
    private static int Wobble(int mode, short close) =>
        (mode == STEER_WOBBLE_FAR && close < WOBBLE_DISTANCE_FAR) || (mode == STEER_WOBBLE_NEAR && close < WOBBLE_DISTANCE_NEAR) ? 1 : 0;
}
