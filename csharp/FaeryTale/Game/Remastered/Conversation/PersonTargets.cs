using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Game.Remastered;

/// Finding the people (not the monsters) under the mouse that the hero can talk to, and whether the
/// hero is close enough to do it.
public static class PersonTargets
{
    // A person's picture is about 16 pixels wide and 32 high, standing on the actor's position.
    private const int HALF_WIDTH = 10;
    private const int HEIGHT_ABOVE_FEET = 30;
    private const int DEPTH_BELOW_FEET = 6;

    /// The hero walks up to this close before the choices appear: near enough that a Yell is told
    /// off, as in the original game, the same as when starting close.
    public const int CONVERSATION_DISTANCE = TalkOptions.TOO_CLOSE_TO_YELL - 1;

    /// When the route ends short of that (a counter or a table is in the way) the choices still
    /// appear if the person is inside the range Talk works in.
    public const int TALK_DISTANCE = TalkOptions.TALK_RANGE - 1;

    /// The actor number of the living person at a world position, if there is one.
    public static int? PersonAt(Data.Game game, (int X, int Y) world)
    {
        for (int number = 1; number < game.Actors.Drawn; number++)
        {
            if (IsTalkable(game.Actors.List[number]) && PictureHolds(game.Actors.List[number], world))
            {
                return number;
            }
        }

        return null;
    }

    /// Whether an actor is still a living person (the slot may have been given to someone else).
    public static bool IsTalkable(Shape actor) => actor.Kind == ActorKind.SETFIG && actor.State != ActorState.DEAD;

    /// Whether the hero stands near enough to talk to an actor.
    public static bool IsCloseEnough(Data.Game game, int actorNumber) =>
        Blocking.ActorDistance(game, actorNumber, 0) < CONVERSATION_DISTANCE;

    /// Whether the hero is within the range Talk works in, though maybe not close enough to be told
    /// off for yelling.
    public static bool IsInTalkRange(Data.Game game, int actorNumber) =>
        Blocking.ActorDistance(game, actorNumber, 0) < TALK_DISTANCE;

    // Whether a world position is inside the box of a person's picture.
    private static bool PictureHolds(Shape actor, (int X, int Y) world) =>
        Math.Abs(world.X - actor.AbsX) <= HALF_WIDTH
        && world.Y >= actor.AbsY - HEIGHT_ABOVE_FEET
        && world.Y <= actor.AbsY + DEPTH_BELOW_FEET;
}
