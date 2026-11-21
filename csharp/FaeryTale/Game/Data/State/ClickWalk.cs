namespace FaeryTale.Game.Data;

/// Everything click to move (Remastered and Reimagined) remembers between frames, kept in three
/// small parts: the route being followed, what to do on arriving, and how the walk was planned.
public sealed class ClickWalk
{
    public readonly WalkRoute Route = new();
    public readonly WalkErrands Errands = new();
    public readonly WalkPlanning Planning = new();
}

/// The route being followed and how the hero is getting along it.
public sealed class WalkRoute
{
    /// The world points still to walk to, nearest first.
    public readonly List<(int X, int Y)> Points = [];

    public (int X, int Y) Goal;
    public bool GoalWalkable;

    /// The click was on water: the hero may swim through deep water to get there (as with the keys).
    public bool Swimming;

    public (ushort X, ushort Y) LastPosition;
    public int StuckFrames;
}

/// What the hero does on arriving: talk to a person, take an item, or look around.
public sealed class WalkErrands
{
    /// A click on a person: the actor to walk up to before the choices of the conversation appear.
    public int? PersonToTalkTo;

    /// A click on something to pick up: the actor to walk to.
    public int? ItemToTake;

    /// The item to take now that the walk ended near it (set when the walk ends, used on the next step).
    public int? ArrivedToTake;

    /// A shift click: look around when the walk is over (ArrivedToLook is that moment).
    public bool LookWhenThere;
    public bool ArrivedToLook;
    public (int X, int Y) LookSpot;
}

/// How the walk was planned: the door to push through, the legs of a far goal, and the retries.
public sealed class WalkPlanning
{
    /// A click on a door: where to push towards once the route ends, and how many frames have been
    /// spent pushing.
    public (int X, int Y)? DoorPoint;
    public int PushFrames;

    /// A far goal is reached in legs: how many have been planned, and where the present one began.
    public int Legs;
    public (int X, int Y) LegStart;

    /// The region the walk started in (the walk is over when the hero is somewhere else).
    public ushort StartRegion;
    public int Replans;
}
