using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// Following the route of a click-to-move walk: steering along it, pushing into doors, and giving up when stuck.
internal static class WalkFollowing
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

    internal const short NOT_WALKING = 9;

    // Steers towards the next route point, dropping points that were reached and planning again
    // when the hero is stuck.
    internal static void Follow(Data.Game game)
    {
        Shape hero = game.Actors.List[0];

        while (game.Walk.Route.Points.Count > 0 && Math.Abs(game.Walk.Route.Points[0].X - hero.AbsX) <= REACH && Math.Abs(game.Walk.Route.Points[0].Y - hero.AbsY) <= REACH)
        {
            game.Walk.Route.Points.RemoveAt(0);
        }

        if (game.Walk.Route.Points.Count == 0)
        {
            if (!WalkRoutes.ContinueTowardsGoal(game, hero))
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

    // At the end of a route to a door: keeps walking into the door, which opens when bumped and lets
    // the hero through. Gives up after a while.
    internal static void Push(Data.Game game, Shape hero)
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
    internal static void EndWalk(Data.Game game)
    {
        int? person = game.Walk.Errands.PersonToTalkTo;
        int? item = game.Walk.Errands.ItemToTake;
        bool look = game.Walk.Errands.LookWhenThere;

        ClickToMove.Cancel(game);
        game.Walk.Errands.ArrivedToLook = look;
        game.Walk.Errands.ArrivedToTake = item is { } itemNumber && ItemTargets.IsCloseEnough(game, itemNumber) ? item : null;

        if (person is { } number && PersonTargets.IsInTalkRange(game, number))
        {
            Conversation.Open(game, number);
        }
    }

    // Counts frames without progress. After a while the route is planned again; after a few
    // attempts the walk is given up. Returns true when the route was dropped.
    internal static bool IsStuck(Data.Game game, Shape hero)
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

        WalkRoutes.PlanFrom(game, hero);

        return game.Walk.Route.Points.Count == 0;
    }

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

    // The player is steering by hand: keys, the compass, the fight button, or riding something.
    internal static bool PlayerSteers(Data.Game game) =>
        InputHandler.UserDirection(game.Ui) < NOT_WALKING
        || game.Ui.FightKeyHeld != 0
        || InputHandler.JoystickFire
        || game.Hero.Riding != 0
        || (InputHandler.MouseButtons & InputHandler.RIGHT_BUTTON) != 0;
}
