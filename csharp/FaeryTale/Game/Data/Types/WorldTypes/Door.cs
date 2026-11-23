namespace FaeryTale.Game.Data;

/// A door between the outdoors and an inside (struct door): Xc1/Yc1 is where it is outside,
/// Xc2/Yc2 where it leads inside. Kind is the sort of door (bit 0 set = horizontal) and Secs says
/// which map section it joins.
public sealed class Door
{
    public ushort Xc1;
    public ushort Yc1;
    public ushort Xc2;
    public ushort Yc2;
    public sbyte Kind;
    public sbyte Secs;
}
