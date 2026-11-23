namespace FaeryTale.Game.Data;

/// The monster encounter being set up and how dangerous the place is.
public sealed class EncounterState
{
    public byte Number;
    public ushort Kind;
    public ushort X;
    public ushort Y;
    public byte DangerLevel;
    public short MixFlag;
    public short WeaponType;
}
