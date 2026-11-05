namespace FaeryTale.Game.Data;

/// A rectangle of the world with a special meaning, for example where a kind of monster appears
/// (struct extent). Etype says what it is; V1-V3 are extra values for that type.
public struct Extent
{
    public ushort X1;
    public ushort Y1;
    public ushort X2;
    public ushort Y2;
    public byte Etype;
    public byte V1;
    public byte V2;
    public byte V3;
}

/// An item or person standing in the world (struct object). ObStat is its state: 0 does not
/// exist, 1 on the ground, 2 taken, 3 a person, 4 a dead person, 5 hidden until looked for.
public struct WorldObject
{
    public ushort Xc;
    public ushort Yc;
    public sbyte ObId;
    public sbyte ObStat;
}

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

/// A kind of person (setfig): which sprite file, the first image, and whether they can talk.
public sealed class SetFigEntry
{
    public sbyte CfileEntry;
    public sbyte ImageBase;
    public sbyte CanTalk;
}

/// One lit part of the compass: where it is in the bitmap and how big (struct compvars).
public sealed class CompassPart
{
    public byte Xrect;
    public byte Yrect;
    public byte Xsize;
    public byte Ysize;
}

/// The disk blocks one region needs (struct need): four image banks, two terrain tables, the
/// sector data, the region map and the character set. The game also keeps one of these for what is
/// loaded now, so it can be changed in place; Copy makes an independent one.
public sealed class Need
{
    public ushort[] Image = new ushort[4];
    public ushort Terra1;
    public ushort Terra2;
    public ushort Sector;
    public ushort Region;
    public ushort Setchar;

    public Need Copy() => new()
    {
        Image = (ushort[])Image.Clone(),
        Terra1 = Terra1,
        Terra2 = Terra2,
        Sector = Sector,
        Region = Region,
        Setchar = Setchar,
    };
}
