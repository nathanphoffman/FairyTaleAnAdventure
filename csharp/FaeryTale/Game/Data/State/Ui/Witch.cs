namespace FaeryTale.Game.Data;

/// The witch and her spinning beam.
public sealed class Witch
{
    public sbyte Direction;
    public sbyte OnScreen;
    public byte Angle;

    /// Which side of the beam's two edges the hero is on.
    public short Side1;
    public short Side2;
}
