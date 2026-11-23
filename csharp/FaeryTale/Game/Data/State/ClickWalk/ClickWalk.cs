namespace FaeryTale.Game.Data;

/// Everything click to move (Remastered and Reimagined) remembers between frames, kept in three
/// small parts: the route being followed, what to do on arriving, and how the walk was planned.
public sealed class ClickWalk
{
    public readonly WalkRoute Route = new();
    public readonly WalkErrands Errands = new();
    public readonly WalkPlanning Planning = new();
}
