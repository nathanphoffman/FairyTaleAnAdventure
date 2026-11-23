namespace FaeryTale.Game.Data;

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
