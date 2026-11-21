using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// Click to move (Remastered and Reimagined): a left click on the play field makes the hero walk
/// there along a route that goes around trees, walls and deep water.
///
/// The route is planned by PathFinder. The hero is not moved directly: each frame the walk
/// direction is set towards the next point on the route, so the hero uses the normal walking
/// code and all its rules. The arrow keys, the compass, fighting and riding take over at once and
/// cancel the route.
public static class ClickToMove
{
    /// A route point counts as reached when the hero is this close (world pixels).
    private const int REACH = 5;

    /// Closer than this on one axis, the hero does not steer along that axis.
    private const int AXIS_DEAD_ZONE = 2;

    /// The hero standing still this many frames on a route means something is in the way.
    private const int STUCK_FRAMES = 20;
    private const int MAX_REPLANS = 3;

    // A far goal takes several routes; the walk is over once the hero is this close to it (world
    // pixels), and no more than this many routes are planned.
    private const int LEG_END_TOLERANCE = 40;
    private const int MAX_LEGS = 8;

    /// Walking into a closed door opens it first and takes a few frames; give up pushing after this many.
    private const int MAX_PUSH_FRAMES = 90;

    // Terrain the planner avoids: deep water, and the wading types cost extra so a dry way wins.
    private const int DEEP_WATER = 5;
    private const int WADING_FIRST = 3;
    private const int WADING_LAST = 4;
    private const int WADING_COST = 60;

    /// Probes for terrain are this far below the hero's position (as in the collision check).
    private const int FEET_OFFSET = 2;
    internal const short NOT_WALKING = 9;
    private const sbyte VIEW_NORMAL = 0;

    /// Where the walk is heading (a world position) and whether the hero can stand there, while a
    /// route is being followed.
    public static ((int X, int Y) World, bool Walkable)? Marker(Data.Game game) => Active(game) && !ClickToAttack.Active ? (game.Walk.Planning.DoorPoint ?? game.Walk.Route.Goal, game.Walk.Route.GoalWalkable) : null;

    /// The classic buttons the walk in progress is for (Talk to a person, Take an item, Look).
    public static PanelButtons WalkingFor(Data.Game game) =>
        (game.Walk.Errands.PersonToTalkTo is not null ? PanelButtons.Talk : 0) | (game.Walk.Errands.ItemToTake is not null ? PanelButtons.Take : 0) | (game.Walk.Errands.LookWhenThere ? PanelButtons.Look : 0);

    /// Whether a route is being followed.
    public static bool Active(Data.Game game) => game.Walk.Route.Points.Count > 0 || game.Walk.Planning.DoorPoint is not null;

    /// While the game is paused a click still gives an order (a walk, or an attack on an enemy) that
    /// the hero carries out when the game goes on, or queues a hotbar item. Remastered and Reimagined only.
    public static void GiveOrdersWhilePaused(Data.Game game, IOutput output)
    {
        if (game.Experience == Experience.Original || game.Ui.ViewStatus != VIEW_NORMAL || InputHandler.TakeClick() is not { } click)
        {
            return;
        }

        // A click on the hotbar queues that item for when the game goes on.
        if (Hotbar.TryClick(game, output, click, WorldView.Current.Width, WorldView.Current.Height))
        {
            return;
        }

        (int X, int Y) target = PointerWorld.OfClick(game, click);

        if (click.Native && EnemyTargets.EnemyAt(game, target) is { } enemy)
        {
            Cancel(game);
            ClickToAttack.Order(enemy);
        }
        else
        {
            ClickToAttack.Clear();
            Plan(game, target);
        }
    }

    /// Forgets the route and any attack order (the hero stops).
    public static void CancelAll(Data.Game game)
    {
        Cancel(game);
        ClickToAttack.Clear();
    }

    /// Plans a walk to a world position without ending an attack order (the attack goes on after the
    /// enemy).
    internal static void RouteTo(Data.Game game, (int X, int Y) goalPoint) => StartRoute(game, goalPoint, null);

    /// Forgets the route (the hero stops).
    public static void Cancel(Data.Game game)
    {
        game.Walk.Route.Points.Clear();
        game.Walk.Planning.DoorPoint = null;
        game.Walk.Errands.PersonToTalkTo = null;
        game.Walk.Errands.ItemToTake = null;
        game.Walk.Errands.LookWhenThere = false;
    }

    /// Called every frame after the keyboard and mouse were read. Takes a new click, and while a
    /// route is active sets the hero's walk direction to follow it.
    public static void Drive(Data.Game game, IOutput output)
    {
        game.Ui.PathWalking = false;
        game.Ui.PathFighting = false;
        game.Ui.PathDirection = NOT_WALKING;

        if (game.Experience == Experience.Original)
        {
            return;
        }

        Conversation.CloseIfLost(game);

        if (InputHandler.TakeClick() is { } click && game.Ui.ViewStatus == VIEW_NORMAL)
        {
            Click(game, output, click);
        }

        if (ClickToAttack.Update(game))
        {
            return;
        }

        if (!Active(game))
        {
            return;
        }

        if (PlayerSteers(game) || game.World.Region != game.Walk.Planning.StartRegion)
        {
            Cancel(game);

            return;
        }

        Follow(game);
        OpenConversationWhenNear(game);
        LookIfArrived(game, output);
        TakeItemWhenThere(game, output);
    }

    /// What entering the world position costs the planner: blocked (-1), open (0), or open but
    /// worth avoiding (wading). Uses the same blocking check as the hero's own steps.
    internal static int CellCost(Data.Game game, int x, int y)
    {
        if (Blocking.BlockedAt(game, (short)x, (short)y, 0) != 0)
        {
            return -1;
        }

        int terrain = MapLookup.TerrainAt(game, x, y + FEET_OFFSET);

        if (terrain == DEEP_WATER)
        {
            return game.Walk.Route.Swimming ? WADING_COST : -1;
        }

        return terrain is >= WADING_FIRST and <= WADING_LAST ? WADING_COST : 0;
    }

    // Whether a world position is water (wading or deep).
    private static bool IsWater(Data.Game game, (int X, int Y) place) =>
        MapLookup.TerrainAt(game, place.X, place.Y + FEET_OFFSET) is >= WADING_FIRST and <= DEEP_WATER;

    /// The walk direction (0-7, clockwise from north-west) towards a point that is (dx, dy) away,
    /// or 9 when already there.
    internal static short DirectionTo(int dx, int dy)
    {
        int stepX = Math.Abs(dx) <= AXIS_DEAD_ZONE ? 0 : Math.Sign(dx);
        int stepY = Math.Abs(dy) <= AXIS_DEAD_ZONE ? 0 : Math.Sign(dy);

        return (stepX, stepY) switch
        {
            (-1, -1) => 0,
            (0, -1) => 1,
            (1, -1) => 2,
            (1, 0) => 3,
            (1, 1) => 4,
            (0, 1) => 5,
            (-1, 1) => 6,
            (-1, 0) => 7,
            _ => NOT_WALKING,
        };
    }

    // A click in the world: a button of an open conversation, else a person to walk up to and talk
    // to, else a place (or door) to walk to. Persons and the bubble only exist in the new drawer.
    private static void Click(Data.Game game, IOutput output, InputHandler.Click click)
    {
        if (Hotbar.TryClick(game, output, click, WorldView.Current.Width, WorldView.Current.Height) || Inventory.TryClick(game, output, InputHandler.GridPointer) || Journal.TryClick(InputHandler.GridPointer) || Conversation.TakeButtonClick(game, output, click) || PanelBackButton.TryPress(game, output, InputHandler.GridPointer))
        {
            return;
        }

        (int X, int Y) target = PointerWorld.OfClick(game, click);

        ClickToAttack.Clear();

        if (InputHandler.ShiftHeld)
        {
            WalkToLook(game, output, target);
        }
        else if (click.Native && EnemyTargets.EnemyAt(game, target) is { } enemy)
        {
            Cancel(game);
            ClickToAttack.Order(enemy);
        }
        else if (click.Native && PersonTargets.PersonAt(game, target) is { } person)
        {
            WalkToPerson(game, person);
        }
        else if (click.Native && output is Video video && ItemTargets.ItemAt(video.Capture, target) is { } item)
        {
            WalkToItem(game, output, item);
        }
        else
        {
            Plan(game, target);
        }
    }

    // Walks up to something lying about and takes it, or takes it at once when already near.
    private static void WalkToItem(Data.Game game, IOutput output, int number)
    {
        Cancel(game);

        if (ItemTargets.IsCloseEnough(game, number))
        {
            TakeItems.TakeFromActor(game, output, number);

            return;
        }

        Shape item = game.Actors.List[number];

        StartRoute(game, (item.AbsX, item.AbsY), null);
        game.Walk.Errands.ItemToTake = Active(game) ? number : null;

        if (!Active(game) && ItemTargets.IsCloseEnough(game, number))
        {
            TakeItems.TakeFromActor(game, output, number);
        }
    }

    // Takes the object walked to once the hero is near it, or when the walk ends short of it.
    private static void TakeItemWhenThere(Data.Game game, IOutput output)
    {
        if (game.Walk.Errands.ItemToTake is { } number && ItemTargets.IsCloseEnough(game, number))
        {
            Cancel(game);
            TakeIfStillThere(game, output, number);
        }
        else if (game.Walk.Errands.ArrivedToTake is { } arrived)
        {
            game.Walk.Errands.ArrivedToTake = null;
            TakeIfStillThere(game, output, arrived);
        }
    }

    private static void TakeIfStillThere(Data.Game game, IOutput output, int number)
    {
        if (ItemTargets.StillThere(game, number))
        {
            TakeItems.TakeFromActor(game, output, number);
        }
    }

    // Shift and click: walks to the spot (not through a door there) and then looks around.
    private static void WalkToLook(Data.Game game, IOutput output, (int X, int Y) target)
    {
        Cancel(game);
        game.Walk.Errands.LookSpot = target;

        // A spot within half the reach of a look is looked at from where the hero stands.
        if (LooksFromHere(game.Actors.List[0], target))
        {
            ItemsMenu.LookAround(game, output, game.Walk.Errands.LookSpot);

            return;
        }

        StartRoute(game, target, null);

        if (Active(game))
        {
            game.Walk.Errands.LookWhenThere = true;
        }
        else
        {
            ItemsMenu.LookAround(game, output, game.Walk.Errands.LookSpot);
        }
    }

    /// Whether a spot is so near the hero (within half the reach of a look) that the look is made without
    /// walking there.
    internal static bool LooksFromHere(Shape hero, (int X, int Y) target) =>
        Math.Max(Math.Abs(target.X - hero.AbsX), Math.Abs(target.Y - hero.AbsY)) <= ItemsMenu.LOOK_RANGE / 2;

    // Looks around once a walk that was meant to end with a look is over.
    private static void LookIfArrived(Data.Game game, IOutput output)
    {
        if (game.Walk.Errands.ArrivedToLook)
        {
            game.Walk.Errands.ArrivedToLook = false;
            ItemsMenu.LookAround(game, output, game.Walk.Errands.LookSpot);
        }
    }

    // Walks towards a person, or opens the conversation at once when already near.
    private static void WalkToPerson(Data.Game game, int number)
    {
        Cancel(game);

        if (PersonTargets.IsCloseEnough(game, number))
        {
            Conversation.Open(game, number);

            return;
        }

        Shape person = game.Actors.List[number];

        Plan(game, (person.AbsX, person.AbsY));
        game.Walk.Errands.PersonToTalkTo = Active(game) ? number : null;

        if (!Active(game) && PersonTargets.IsInTalkRange(game, number))
        {
            Conversation.Open(game, number);
        }
    }

    // Once the hero is near the person walked to, stops and opens the conversation.
    private static void OpenConversationWhenNear(Data.Game game)
    {
        if (game.Walk.Errands.PersonToTalkTo is not { } number || !PersonTargets.IsCloseEnough(game, number))
        {
            return;
        }

        Cancel(game);
        Conversation.Open(game, number);
    }

    // A click on a door walks to the door and the hero pushes his way through. So does a click on
    // the black outside of the room the hero is in: it means "get me out of here".
    private static void Plan(Data.Game game, (int X, int Y) target) =>
        StartRoute(game, target, DoorTargets.DoorAt(game, target) ?? DoorTargets.ExitForBlackClick(game, target));

    // Starts a walk to a world position (or, for a door, to the way in).
    private static void StartRoute(Data.Game game, (int X, int Y) target, int? door)
    {
        Shape hero = game.Actors.List[0];

        if (game.Hero.Riding != 0)
        {
            return;
        }

        game.Walk.Errands.PersonToTalkTo = null;
        game.Walk.Errands.ItemToTake = null;
        game.Walk.Errands.LookWhenThere = false;
        game.Walk.Route.Swimming = IsWater(game, target);
        game.Walk.Planning.DoorPoint = door is { } found ? DoorTargets.EntryPoint(game, found) : null;
        game.Walk.Route.Goal = door is { } doorToApproach ? DoorTargets.ApproachPoint(game, doorToApproach) : target;
        game.Walk.Route.GoalWalkable = game.Walk.Planning.DoorPoint is not null || CellCost(game, target.X, target.Y) >= 0;
        game.Walk.Planning.StartRegion = game.World.Region;
        game.Walk.Planning.Replans = 0;
        game.Walk.Route.StuckFrames = 0;
        game.Walk.Planning.PushFrames = 0;
        game.Walk.Planning.Legs = 0;
        game.Walk.Planning.LegStart = (hero.AbsX, hero.AbsY);
        PlanFrom(game, hero);
    }

    // Plans the route to the goal from the hero's present position.
    private static void PlanFrom(Data.Game game, Shape hero)
    {
        int cell = PathFinder.CELL;
        (int X, int Y) goalCell = (RoundDivide(game.Walk.Route.Goal.X - hero.AbsX, cell), RoundDivide(game.Walk.Route.Goal.Y - hero.AbsY, cell));

        game.Walk.Route.Points.Clear();

        foreach ((int x, int y) in PathFinder.Find(goalCell, (cx, cy) => CellCost(game, hero.AbsX + cx * cell, hero.AbsY + cy * cell)))
        {
            game.Walk.Route.Points.Add((hero.AbsX + x * cell, hero.AbsY + y * cell));
        }

        game.Walk.Route.LastPosition = (hero.AbsX, hero.AbsY);
    }

    // The player is steering by hand: keys, the compass, the fight button, or riding something.
    internal static bool PlayerSteers(Data.Game game) =>
        InputHandler.UserDirection(game.Ui) < NOT_WALKING
        || game.Ui.FightKeyHeld != 0
        || InputHandler.JoystickFire
        || game.Hero.Riding != 0
        || (InputHandler.MouseButtons & InputHandler.RIGHT_BUTTON) != 0;

    // Steers towards the next route point, dropping points that were reached and planning again
    // when the hero is stuck.
    private static void Follow(Data.Game game)
    {
        Shape hero = game.Actors.List[0];

        while (game.Walk.Route.Points.Count > 0 && Math.Abs(game.Walk.Route.Points[0].X - hero.AbsX) <= REACH && Math.Abs(game.Walk.Route.Points[0].Y - hero.AbsY) <= REACH)
        {
            game.Walk.Route.Points.RemoveAt(0);
        }

        if (game.Walk.Route.Points.Count == 0)
        {
            if (!ContinueTowardsGoal(game, hero))
            {
                Push(game, hero);
            }

            return;
        }

        if (IsStuck(game, hero))
        {
            return;
        }

        game.Ui.PathDirection = DirectionTo(game.Walk.Route.Points[0].X - hero.AbsX, game.Walk.Route.Points[0].Y - hero.AbsY);
        game.Ui.CompassLitDirection = game.Ui.PathDirection;
        game.Ui.PathWalking = true;
    }

    // A route is only planned over a limited distance (PathFinder), so a goal that is far off is reached
    // in several legs, each planned from where the last one ended. Returns true when another leg
    // was planned. The walk is over when the goal is near, the last leg went nowhere, or after
    // MAX_LEGS legs.
    private static bool ContinueTowardsGoal(Data.Game game, Shape hero)
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

    // At the end of a route to a door: keeps walking into the door, which opens when bumped and lets
    // the hero through. Gives up after a while.
    private static void Push(Data.Game game, Shape hero)
    {
        if (game.Walk.Planning.DoorPoint is not { } door || game.Walk.Planning.PushFrames++ >= MAX_PUSH_FRAMES)
        {
            EndWalk(game);

            return;
        }

        game.Ui.PathDirection = DirectionTo(door.X - hero.AbsX, door.Y - hero.AbsY);
        game.Ui.CompassLitDirection = game.Ui.PathDirection;
        game.Ui.PathWalking = game.Ui.PathDirection < NOT_WALKING;
    }

    // The route is over: a walk to a person that ended in talking range still opens the conversation.
    private static void EndWalk(Data.Game game)
    {
        int? person = game.Walk.Errands.PersonToTalkTo;
        int? item = game.Walk.Errands.ItemToTake;
        bool look = game.Walk.Errands.LookWhenThere;

        Cancel(game);
        game.Walk.Errands.ArrivedToLook = look;
        game.Walk.Errands.ArrivedToTake = item is { } itemNumber && ItemTargets.IsCloseEnough(game, itemNumber) ? item : null;

        if (person is { } number && PersonTargets.IsInTalkRange(game, number))
        {
            Conversation.Open(game, number);
        }
    }

    // Counts frames without progress. After a while the route is planned again; after a few
    // attempts the walk is given up. Returns true when the route was dropped.
    private static bool IsStuck(Data.Game game, Shape hero)
    {
        game.Walk.Route.StuckFrames = (hero.AbsX, hero.AbsY) == game.Walk.Route.LastPosition ? game.Walk.Route.StuckFrames + 1 : 0;
        game.Walk.Route.LastPosition = (hero.AbsX, hero.AbsY);

        if (game.Walk.Route.StuckFrames < STUCK_FRAMES)
        {
            return false;
        }

        game.Walk.Route.StuckFrames = 0;
        game.Walk.Planning.Replans++;

        if (game.Walk.Planning.Replans > MAX_REPLANS)
        {
            EndWalk(game);

            return true;
        }

        PlanFrom(game, hero);

        return game.Walk.Route.Points.Count == 0;
    }

    // Divides rounding to the nearest whole number (halves away from zero).
    private static int RoundDivide(int value, int by) => (int)Math.Round((double)value / by, MidpointRounding.AwayFromZero);
}
