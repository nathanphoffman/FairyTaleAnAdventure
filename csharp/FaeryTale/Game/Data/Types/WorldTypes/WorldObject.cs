namespace FaeryTale.Game.Data;

/// An item or person standing in the world (struct object). ObStat is its state: 0 does not
/// exist, 1 on the ground, 2 taken, 3 a person, 4 a dead person, 5 hidden until looked for.
public struct WorldObject
{
    public ushort Xc;
    public ushort Yc;
    public sbyte ObId;
    public sbyte ObStat;
}
