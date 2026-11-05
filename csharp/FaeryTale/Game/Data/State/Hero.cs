namespace FaeryTale.Game.Data;

/// The hero: where they are, their stats, and what they carry or ride.
public sealed class Hero
{
    /// How many items a brother can carry: 35 slots that are saved, and one more (the last) that
    /// the pick-up code uses to count the arrows in a quiver.
    public const int STUFF_SLOTS = 36;

    /// The slots of the item counts that are saved.
    public const int SAVED_STUFF_SLOTS = 35;

    public ushort X;
    public ushort Y;
    public short Sector;
    public ushort Place;
    public short Brave;
    public short Luck;
    public short Kindness;
    public short Wealth;
    public short Hunger;
    public short Fatigue;
    public short Riding;
    public short Flying;
    public short Brother;
    public byte[] Julstuff = new byte[STUFF_SLOTS];
    public byte[] Kevstuff = new byte[STUFF_SLOTS];
    public byte[] Philstuff = new byte[STUFF_SLOTS];
    public sbyte SafeFlag;
    public ushort SafeR;
    public ushort SafeX;
    public ushort SafeY;
    public byte GoodFairyTimer;
    public short PrincessesRescued;

    /// The item counts of the brother who is playing (brothers are numbered from 1).
    public byte[] Stuff => Brother switch
    {
        2 => Philstuff,
        3 => Kevstuff,
        _ => Julstuff,
    };
}
