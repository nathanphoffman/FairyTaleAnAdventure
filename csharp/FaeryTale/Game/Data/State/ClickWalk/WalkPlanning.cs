namespace FaeryTale.Game.Data;

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
