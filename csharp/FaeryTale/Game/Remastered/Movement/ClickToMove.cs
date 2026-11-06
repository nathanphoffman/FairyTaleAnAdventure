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

    /// Walking into a closed door opens it first and takes a few frames; give up pushing after this many.
    private const int MAX_PUSH_FRAMES = 90;

    // Terrain the planner avoids: deep water, and the wading types cost extra so a dry way wins.
    private const int DEEP_WATER = 5;
    private const int WADING_FIRST = 3;
    private const int WADING_LAST = 4;
    private const int WADING_COST = 60;

    /// Probes for terrain are this far below the hero's position (as in the collision check).
    private const int FEET_OFFSET = 2;
    private const short NOT_WALKING = 9;
    private const sbyte VIEW_NORMAL = 0;

    private static readonly List<(int X, int Y)> route = [];
    private static (int X, int Y) goal;
    private static (ushort X, ushort Y) lastPosition;
    private static int stuckFrames;
    private static int replans;
    private static bool goalWalkable;

    // The click was on water: the hero may swim through deep water to get there (as with the keys).
    private static bool swimming;

    // A click on a person: the actor to walk up to before the choices of the conversation appear.
    private static int? personToTalkTo;

    // A click on something to pick up: the actor to walk to, and the one to take now that the walk
    // ended near it (set when the walk ends, used on the next step).
    private static int? itemToTake;
    private static int? arrivedToTake;

    // A shift click: look around when the walk is over (arrivedToLook is that moment).
    private static bool lookWhenThere;
    private static bool arrivedToLook;

    // A click on a door: where to push towards once the route ends, how many frames have been spent
    // pushing, and the region the walk started in (the walk is over when the hero is somewhere else).
    private static (int X, int Y)? doorPoint;
    private static int pushFrames;
    private static ushort startRegion;

    /// Where the walk is heading (a world position) and whether the hero can stand there, while a
    /// route is being followed.
    public static ((int X, int Y) World, bool Walkable)? Marker => Active ? (doorPoint ?? goal, goalWalkable) : null;

    /// Whether a route is being followed.
    public static bool Active => route.Count > 0 || doorPoint is not null;

    /// Forgets the route (the hero stops).
    public static void Cancel()
    {
        route.Clear();
        doorPoint = null;
        personToTalkTo = null;
        itemToTake = null;
        lookWhenThere = false;
    }

    /// Called every frame after the keyboard and mouse were read. Takes a new click, and while a
    /// route is active sets the hero's walk direction to follow it.
    public static void Drive(Data.Game game, IOutput output)
    {
        game.Ui.PathWalking = false;
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

        if (!Active)
        {
            return;
        }

        if (PlayerSteers(game) || game.World.Region != startRegion)
        {
            Cancel();

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
            return swimming ? WADING_COST : -1;
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
        if (Conversation.TakeButtonClick(game, output, click))
        {
            return;
        }

        (int X, int Y) target = PointerWorld.OfClick(game, click);

        if (InputHandler.ShiftHeld)
        {
            WalkToLook(game, output, target);
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
        Cancel();

        if (ItemTargets.IsCloseEnough(game, number))
        {
            TakeItems.TakeFromActor(game, output, number);

            return;
        }

        Shape item = game.Actors.List[number];

        StartRoute(game, (item.AbsX, item.AbsY), null);
        itemToTake = Active ? number : null;

        if (!Active && ItemTargets.IsCloseEnough(game, number))
        {
            TakeItems.TakeFromActor(game, output, number);
        }
    }

    // Takes the object walked to once the hero is near it, or when the walk ends short of it.
    private static void TakeItemWhenThere(Data.Game game, IOutput output)
    {
        if (itemToTake is { } number && ItemTargets.IsCloseEnough(game, number))
        {
            Cancel();
            TakeIfStillThere(game, output, number);
        }
        else if (arrivedToTake is { } arrived)
        {
            arrivedToTake = null;
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
        Cancel();
        StartRoute(game, target, null);

        if (Active)
        {
            lookWhenThere = true;
        }
        else
        {
            ItemsMenu.LookAround(game, output);
        }
    }

    // Looks around once a walk that was meant to end with a look is over.
    private static void LookIfArrived(Data.Game game, IOutput output)
    {
        if (arrivedToLook)
        {
            arrivedToLook = false;
            ItemsMenu.LookAround(game, output);
        }
    }

    // Walks towards a person, or opens the conversation at once when already near.
    private static void WalkToPerson(Data.Game game, int number)
    {
        Cancel();

        if (PersonTargets.IsCloseEnough(game, number))
        {
            Conversation.Open(game, number);

            return;
        }

        Shape person = game.Actors.List[number];

        Plan(game, (person.AbsX, person.AbsY));
        personToTalkTo = Active ? number : null;

        if (!Active && PersonTargets.IsInTalkRange(game, number))
        {
            Conversation.Open(game, number);
        }
    }

    // Once the hero is near the person walked to, stops and opens the conversation.
    private static void OpenConversationWhenNear(Data.Game game)
    {
        if (personToTalkTo is not { } number || !PersonTargets.IsCloseEnough(game, number))
        {
            return;
        }

        Cancel();
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

        personToTalkTo = null;
        itemToTake = null;
        lookWhenThere = false;
        swimming = IsWater(game, target);
        doorPoint = door is { } found ? DoorTargets.EntryPoint(game, found) : null;
        goal = door is { } doorToApproach ? DoorTargets.ApproachPoint(game, doorToApproach) : target;
        goalWalkable = doorPoint is not null || CellCost(game, target.X, target.Y) >= 0;
        startRegion = game.World.Region;
        replans = 0;
        stuckFrames = 0;
        pushFrames = 0;
        PlanFrom(game, hero);
    }

    // Plans the route to the goal from the hero's present position.
    private static void PlanFrom(Data.Game game, Shape hero)
    {
        int cell = PathFinder.CELL;
        (int X, int Y) goalCell = (RoundDivide(goal.X - hero.AbsX, cell), RoundDivide(goal.Y - hero.AbsY, cell));

        route.Clear();

        foreach ((int x, int y) in PathFinder.Find(goalCell, (cx, cy) => CellCost(game, hero.AbsX + cx * cell, hero.AbsY + cy * cell)))
        {
            route.Add((hero.AbsX + x * cell, hero.AbsY + y * cell));
        }

        lastPosition = (hero.AbsX, hero.AbsY);
    }

    // The player is steering by hand: keys, the compass, the fight button, or riding something.
    private static bool PlayerSteers(Data.Game game) =>
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

        while (route.Count > 0 && Math.Abs(route[0].X - hero.AbsX) <= REACH && Math.Abs(route[0].Y - hero.AbsY) <= REACH)
        {
            route.RemoveAt(0);
        }

        if (route.Count == 0)
        {
            Push(game, hero);

            return;
        }

        if (IsStuck(game, hero))
        {
            return;
        }

        game.Ui.PathDirection = DirectionTo(route[0].X - hero.AbsX, route[0].Y - hero.AbsY);
        game.Ui.CompassLitDirection = game.Ui.PathDirection;
        game.Ui.PathWalking = true;
    }

    // At the end of a route to a door: keeps walking into the door, which opens when bumped and lets
    // the hero through. Gives up after a while.
    private static void Push(Data.Game game, Shape hero)
    {
        if (doorPoint is not { } door || pushFrames++ >= MAX_PUSH_FRAMES)
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
        int? person = personToTalkTo;
        int? item = itemToTake;
        bool look = lookWhenThere;

        Cancel();
        arrivedToLook = look;
        arrivedToTake = item is { } itemNumber && ItemTargets.IsCloseEnough(game, itemNumber) ? item : null;

        if (person is { } number && PersonTargets.IsInTalkRange(game, number))
        {
            Conversation.Open(game, number);
        }
    }

    // Counts frames without progress. After a while the route is planned again; after a few
    // attempts the walk is given up. Returns true when the route was dropped.
    private static bool IsStuck(Data.Game game, Shape hero)
    {
        stuckFrames = (hero.AbsX, hero.AbsY) == lastPosition ? stuckFrames + 1 : 0;
        lastPosition = (hero.AbsX, hero.AbsY);

        if (stuckFrames < STUCK_FRAMES)
        {
            return false;
        }

        stuckFrames = 0;
        replans++;

        if (replans > MAX_REPLANS)
        {
            EndWalk(game);

            return true;
        }

        PlanFrom(game, hero);

        return route.Count == 0;
    }

    // Divides rounding to the nearest whole number (halves away from zero).
    private static int RoundDivide(int value, int by) => (int)Math.Round((double)value / by, MidpointRounding.AwayFromZero);
}
