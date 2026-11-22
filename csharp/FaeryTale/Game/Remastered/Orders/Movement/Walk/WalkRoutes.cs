using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// Planning the route of a click-to-move walk, in legs when the goal is far.
internal static class WalkRoutes
{
    // A far goal takes several routes; the walk is over once the hero is this close to it (world
    // pixels), and no more than this many routes are planned.
    private const int LEG_END_TOLERANCE = 40;

    private const int MAX_LEGS = 8;

    // Starts a walk to a world position (or, for a door, to the way in).
    internal static void StartRoute(Data.Game game, (int X, int Y) target, int? door)
    {
        Shape hero = game.Actors.List[0];

        if (game.Hero.Riding != 0)
        {
            return;
        }

        game.Walk.Errands.PersonToTalkTo = null;
        game.Walk.Errands.ItemToTake = null;
        game.Walk.Errands.LookWhenThere = false;
        game.Walk.Route.Swimming = WalkTerrain.IsWater(game, target);
        game.Walk.Planning.DoorPoint = door is { } found ? DoorTargets.EntryPoint(game, found) : null;
        game.Walk.Route.Goal = door is { } doorToApproach ? DoorTargets.ApproachPoint(game, doorToApproach) : target;
        game.Walk.Route.GoalWalkable = game.Walk.Planning.DoorPoint is not null || WalkTerrain.CellCost(game, target.X, target.Y) >= 0;
        game.Walk.Planning.StartRegion = game.World.Region;
        game.Walk.Planning.Replans = 0;
        game.Walk.Route.StuckFrames = 0;
        game.Walk.Planning.PushFrames = 0;
        game.Walk.Planning.Legs = 0;
        game.Walk.Planning.LegStart = (hero.AbsX, hero.AbsY);
        PlanFrom(game, hero);
    }

    /// Plans a walk to a world position without ending an attack order (the attack goes on after the
    /// enemy).
    internal static void RouteTo(Data.Game game, (int X, int Y) goalPoint) => StartRoute(game, goalPoint, null);

    // Plans the route to the goal from the hero's present position.
    internal static void PlanFrom(Data.Game game, Shape hero)
    {
        int cell = PathFinder.CELL;
        (int X, int Y) goalCell = (WalkTerrain.RoundDivide(game.Walk.Route.Goal.X - hero.AbsX, cell), WalkTerrain.RoundDivide(game.Walk.Route.Goal.Y - hero.AbsY, cell));

        game.Walk.Route.Points.Clear();

        foreach ((int x, int y) in PathFinder.Find(goalCell, (cx, cy) => WalkTerrain.CellCost(game, hero.AbsX + cx * cell, hero.AbsY + cy * cell)))
        {
            game.Walk.Route.Points.Add((hero.AbsX + x * cell, hero.AbsY + y * cell));
        }

        game.Walk.Route.LastPosition = (hero.AbsX, hero.AbsY);
    }

    // A route is only planned over a limited distance (PathFinder), so a goal that is far off is reached
    // in several legs, each planned from where the last one ended. Returns true when another leg
    // was planned. The walk is over when the goal is near, the last leg went nowhere, or after
    // MAX_LEGS legs.
    internal static bool ContinueTowardsGoal(Data.Game game, Shape hero)
    {
        int distance = Math.Max(Math.Abs(game.Walk.Route.Goal.X - hero.AbsX), Math.Abs(game.Walk.Route.Goal.Y - hero.AbsY));
        bool progressed = (hero.AbsX, hero.AbsY) != game.Walk.Planning.LegStart;

        if (distance <= LEG_END_TOLERANCE || !progressed || game.Walk.Planning.Legs >= MAX_LEGS)
        {
            return false;
        }

        game.Walk.Planning.Legs++;
        game.Walk.Planning.LegStart = (hero.AbsX, hero.AbsY);
        PlanFrom(game, hero);

        return game.Walk.Route.Points.Count > 0;
    }
}
