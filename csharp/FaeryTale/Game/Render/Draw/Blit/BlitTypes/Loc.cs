using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// A place in an area: Pos bytes from its start (for a plane, from the first visible byte, so it
/// can reach a little before it).
public readonly record struct Loc(Area Area, long Pos)
{
    /// A place that is not used.
    public static readonly Loc NONE = new(Area.Queue, 0);

    /// The place bytes further on.
    public Loc Add(long bytes) => new(Area, Pos + bytes);
}
