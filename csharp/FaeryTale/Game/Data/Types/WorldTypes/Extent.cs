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
