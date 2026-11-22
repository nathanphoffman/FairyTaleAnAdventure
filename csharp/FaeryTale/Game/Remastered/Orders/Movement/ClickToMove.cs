using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// Click to move (Remastered and Reimagined): a left click on the play field makes the hero walk
/// there along a route that goes around trees, walls and deep water.
///
/// The route is planned by WalkRoutes. The hero is not moved directly: each frame the walk
/// direction is set towards the next point on the route, so the hero uses the normal walking
/// code and all its rules. The arrow keys, the compass, fighting and riding take over at once and
/// cancel the route. The parts are in the Walk folder.
public static class ClickToMove
{
    /// Where the walk is heading (a world position) and whether the hero can stand there, while a
    /// route is being followed.
    public static ((int X, int Y) World, bool Walkable)? Marker(Data.Game game) => Active(game) && !ClickToAttack.Active ? (game.Walk.Planning.DoorPoint ?? game.Walk.Route.Goal, game.Walk.Route.GoalWalkable) : null;

    /// The classic buttons the walk in progress is for (Talk to a person, Take an item, Look).
    public static PanelButtons WalkingFor(Data.Game game) =>
        (game.Walk.Errands.PersonToTalkTo is not null ? PanelButtons.Talk : 0) | (game.Walk.Errands.ItemToTake is not null ? PanelButtons.Take : 0) | (game.Walk.Errands.LookWhenThere ? PanelButtons.Look : 0);

    /// Whether a route is being followed.
    public static bool Active(Data.Game game) => game.Walk.Route.Points.Count > 0 || game.Walk.Planning.DoorPoint is not null;

    /// Forgets the route and any attack order (the hero stops).
    public static void CancelAll(Data.Game game)
    {
        Cancel(game);
        ClickToAttack.Clear();
    }

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
        game.Ui.PathDirection = WalkFollowing.NOT_WALKING;

        if (game.Experience == Experience.Original)
        {
            return;
        }

        Conversation.CloseIfLost(game);

        if (InputHandler.TakeClick() is { } click && game.Ui.ViewStatus == WalkClicks.VIEW_NORMAL)
        {
            WalkClicks.Click(game, output, click);
        }

        if (ClickToAttack.Update(game))
        {
            return;
        }

        if (!Active(game))
        {
            return;
        }

        if (WalkFollowing.PlayerSteers(game) || game.World.Region != game.Walk.Planning.StartRegion)
        {
            Cancel(game);

            return;
        }

        WalkFollowing.Follow(game);
        PersonWalks.OpenConversationWhenNear(game);
        LookWalks.LookIfArrived(game, output);
        ItemWalks.TakeItemWhenThere(game, output);
    }
}
