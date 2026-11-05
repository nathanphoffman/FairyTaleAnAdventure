using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Creatures.Interaction;

/// What blocks an actor's way (terrain or another actor), taking a step, and finding the nearest
/// actor.
public static class Blocking
{
    /// Value BlockedAt returns when another actor is in the way.
    private const int BLOCKED_BY_ACTOR = 16;

    // Collision box around an actor.
    private const int BLOCK_HALF_WIDTH = 11;
    private const int BLOCK_HALF_HEIGHT = 9;

    /// The raft is always slot 1; rafts and other carriers do not block movement.
    private const int RAFT_SLOT = 1;

    /// The wraith race floats, so it ignores terrain.
    internal const byte RACE_WRAITH = 2;

    /// Terrain types that never stop the hero.
    private static readonly int[] SOFT_TERRAIN = [8, 9];

    /// Index of the "still" object type that always counts when looking for the nearest figure.
    private const sbyte ALWAYS_COUNTED_OBJECT = 0x1d;

    // The diagonal is approximated as 5/7 of the sum of the two offsets.
    private const int DIAGONAL_NUMERATOR = 5;
    private const int DIAGONAL_DENOMINATOR = 7;

    /// What blocks actor index at (x, y): a terrain type, 16 for another actor, or 0 if clear.
    ///
    /// Terrain is checked first (wraiths float, so they ignore it; the hero ignores terrain types
    /// 8 and 9). Then every other actor is checked: it blocks if it is within 10 pixels sideways
    /// and 8 pixels up or down. Dead actors and the raft can be walked over.
    public static int BlockedAt(Data.Game game, short x, short y, short index)
    {
        Shape me = game.Actors.List[index];

        if (me.Kind != ActorKind.ENEMY || me.Race != RACE_WRAITH)
        {
            int terrain = MapLookup.ProbeCollision(game, x, y);

            if (index == 0 && SOFT_TERRAIN.Contains(terrain))
            {
                terrain = 0;
            }

            if (terrain != 0)
            {
                return terrain;
            }
        }

        for (int other = 0; other < game.Actors.Active; other++)
        {
            if (BlockedByActor(game.Actors, x, y, index, other))
            {
                return BLOCKED_BY_ACTOR;
            }
        }

        return 0;
    }

    /// Moves an actor dist in direction dir if nothing blocks it. Returns 1 if it moved.
    public static int StepFigure(Data.Game game, short figure, short dir, short dist)
    {
        Shape actor = game.Actors.List[figure];
        ushort x = (ushort)MapSteps.StepX(actor.AbsX, dir, dist);
        ushort y = (ushort)MapSteps.StepY(actor.AbsY, dir, dist);

        if (BlockedAt(game, (short)x, (short)y, figure) != 0)
        {
            return 0;
        }

        actor.AbsX = x;
        actor.AbsY = y;

        return 1;
    }

    /// Finds the nearest actor to the hero closer than dist, setting NearestObject; returns the
    /// distance found. With constraint set, objects are ignored too.
    public static int FindNearestFigure(Data.Game game, sbyte constraint, short dist)
    {
        game.Actors.NearestObject = 0;

        for (int number = 1; number < game.Actors.Drawn; number++)
        {
            Shape actor = game.Actors.List[number];

            if (actor.Kind == ActorKind.OBJECTS && (constraint != 0 || actor.Index == ALWAYS_COUNTED_OBJECT))
            {
                continue;
            }

            int distance = ActorDistance(game, number, 0);

            if (distance < dist)
            {
                game.Actors.NearestObject = (short)number;
                dist = (short)distance;
            }
        }

        return dist;
    }

    /// Distance between two actors. The game avoids a square root: if one axis is more than twice
    /// the other it is used alone, otherwise (x + y) * 5 / 7 approximates the diagonal.
    public static int ActorDistance(Data.Game game, int first, int second)
    {
        Shape a = game.Actors.List[first];
        Shape b = game.Actors.List[second];
        int x = Math.Abs(a.AbsX - b.AbsX);
        int y = Math.Abs(a.AbsY - b.AbsY);

        if (x > y + y)
        {
            return x;
        }

        return y > x + x ? y : (x + y) * DIAGONAL_NUMERATOR / DIAGONAL_DENOMINATOR;
    }

    // Whether actor other stands in the way of actor index at (x, y). Dead actors and rafts can be
    // walked over.
    private static bool BlockedByActor(Actors actors, short x, short y, short index, int other)
    {
        Shape actor = actors.List[other];

        if (index == other || other == RAFT_SLOT || actor.Kind == ActorKind.CARRIER || actor.State == ActorState.DEAD)
        {
            return false;
        }

        int dx = x - actor.AbsX;
        int dy = y - actor.AbsY;

        return dx < BLOCK_HALF_WIDTH && dx > -BLOCK_HALF_WIDTH && dy < BLOCK_HALF_HEIGHT && dy > -BLOCK_HALF_HEIGHT;
    }
}
