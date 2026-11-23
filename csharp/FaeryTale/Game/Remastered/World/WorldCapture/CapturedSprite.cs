using FaeryTale.Game.Data;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Remastered;

/// One character or object as the new world drawer sees it: which picture, and where in the world.
public sealed class CapturedSprite
{
    /// Which actor it is (the slot, and 1 for the weapon pass of an actor drawn in two).
    public int Slot;
    public int Pass;
    public byte Race;
    public sbyte Kind;

    /// The sprite sheet (a copy of where it is in memory) and the picture in it.
    public SeqInfo Sheet;
    public short Inum;

    /// The world position of the picture's top left corner, how many of its lines to draw, and how
    /// many lines at the top to leave out.
    public int Left;
    public int Top;
    public int Rows;
    public int SkipRows;

    /// The actor's own position, and how far it moved in the last tick (the previous position minus
    /// this one): drawing it that much back gives the smooth movement between two ticks.
    public int ActorX;
    public int ActorY;
    public double DeltaX;
    public double DeltaY;

    /// The line of the world the actor stands on, for hiding it behind scenery.
    public int GroundY;

    /// Drawn over everything, whatever scenery is in front.
    public bool InFront;

    public Shape? Actor;
}
