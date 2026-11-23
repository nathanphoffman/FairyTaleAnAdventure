using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Remastered;

/// Smarter enemies (Remastered and Reimagined). The original steers an enemy straight at its target
/// and, when blocked, only tries a step to either side, so enemies jam against walls and corners
/// while the hero walks round them. Here an enemy whose straight way is blocked by the terrain is
/// aimed at the next point of a route found with PathFinder instead. The rest of the enemy's
/// behaviour (when it attacks, flees or waits) is unchanged, and so is the way it walks.
public static class EnemyPathing
{
    /// An enemy this close to its target walks straight at it (it is in fighting range).
    private const int DIRECT_DISTANCE = 24;

    /// How far apart the points are that are tested for a clear straight line.
    private const int LINE_STEP = 6;

    /// A route point counts as reached when the enemy is this close.
    private const int REACH = 5;

    /// A route is planned again after this many frames (the hero moves), or when the target has
    /// moved this far from where it was.
    private const int ROUTE_LIFETIME = 45;
    private const int TARGET_MOVE_LIMIT = 40;

    /// The first point of a route must be near the enemy, or the route belongs to another enemy that
    /// used the slot before.
    private const int MAX_FIRST_POINT_DISTANCE = 3 * PathFinder.CELL;

    // Terrain that stops an enemy for good: deep water. (Shallow water is walked through.)
    private const int DEEP_WATER = 5;
    private const int WADING_COST = 40;
    private const int FEET_OFFSET = 2;

    /// The enemy slot number the routes are kept for.
    private const int SLOT_COUNT = 24;

    private sealed class Route
    {
        public List<(int X, int Y)> Points = [];
        public (int X, int Y) Target;
        public int Age;
    }

    private static readonly Route?[] routes = new Route?[SLOT_COUNT];

    /// Forgets all routes (a new game starts).
    public static void Reset() => Array.Clear(routes);

    /// Where an enemy should aim to reach a target: the target itself when the way is clear, else
    /// the next point of a route round the obstacles.
    public static (int X, int Y) AimPoint(Data.Game game, int slot, (int X, int Y) target)
    {
        Shape actor = game.Actors.List[slot];

        if (game.Experience == Experience.Original || slot >= SLOT_COUNT || !IsWalkingEnemy(actor)
            || DistanceTo(actor, target) < DIRECT_DISTANCE || IsLineClear(game, actor, target))
        {
            if (slot < SLOT_COUNT)
            {
                routes[slot] = null;
            }

            return target;
        }

        Route route = RouteFor(game, slot, actor, target);

        return route.Points.Count > 0 ? route.Points[0] : target;
    }

    // Wraiths float through walls and need no route.
    private static bool IsWalkingEnemy(Shape actor) =>
        actor.Kind == ActorKind.ENEMY && actor.Race != Creatures.Interaction.Blocking.RACE_WRAITH;

    private static int DistanceTo(Shape actor, (int X, int Y) target) =>
        Math.Max(Math.Abs(actor.AbsX - target.X), Math.Abs(actor.AbsY - target.Y));

    // Whether the terrain lets the enemy walk straight to the target: probed every LINE_STEP pixels.
    private static bool IsLineClear(Data.Game game, Shape actor, (int X, int Y) target)
    {
        int steps = DistanceTo(actor, target) / LINE_STEP;

        for (int step = 1; step <= steps; step++)
        {
            int x = actor.AbsX + (target.X - actor.AbsX) * step / steps;
            int y = actor.AbsY + (target.Y - actor.AbsY) * step / steps;

            if (MapLookup.ProbeCollision(game, x, y) != 0)
            {
                return false;
            }
        }

        return true;
    }

    // The enemy's route, planned afresh when there is none, it is old, the target has moved, or it
    // does not start near the enemy. Reached points are dropped.
    private static Route RouteFor(Data.Game game, int slot, Shape actor, (int X, int Y) target)
    {
        Route route = routes[slot] ??= new Route { Target = target, Age = ROUTE_LIFETIME };

        route.Age++;

        while (route.Points.Count > 0 && DistanceTo(actor, route.Points[0]) <= REACH)
        {
            route.Points.RemoveAt(0);
        }

        bool stale = route.Age >= ROUTE_LIFETIME
            || Math.Max(Math.Abs(route.Target.X - target.X), Math.Abs(route.Target.Y - target.Y)) > TARGET_MOVE_LIMIT
            || (route.Points.Count > 0 && DistanceTo(actor, route.Points[0]) > MAX_FIRST_POINT_DISTANCE);

        if (stale || route.Points.Count == 0)
        {
            Plan(game, route, actor, target);
        }

        return route;
    }

    private static void Plan(Data.Game game, Route route, Shape actor, (int X, int Y) target)
    {
        int cell = PathFinder.CELL;
        (int X, int Y) goalCell = (RoundDivide(target.X - actor.AbsX, cell), RoundDivide(target.Y - actor.AbsY, cell));

        route.Points = PathFinder.Find(goalCell, (cx, cy) => CellCost(game, actor.AbsX + cx * cell, actor.AbsY + cy * cell))
            .Select(point => (actor.AbsX + point.X * cell, actor.AbsY + point.Y * cell))
            .ToList();
        route.Target = target;
        route.Age = 0;
    }

    // What entering a position costs an enemy: blocked by terrain or deep water, open, or wading.
    // Other actors are left out on purpose: they move, and the enemy's own walking deals with them.
    private static int CellCost(Data.Game game, int x, int y)
    {
        if (MapLookup.ProbeCollision(game, x, y) != 0)
        {
            return -1;
        }

        int terrain = MapLookup.TerrainAt(game, x, y + FEET_OFFSET);

        return terrain == DEEP_WATER ? -1 : terrain == 0 ? 0 : WADING_COST;
    }

    private static int RoundDivide(int value, int by) => (int)Math.Round((double)value / by, MidpointRounding.AwayFromZero);
}
