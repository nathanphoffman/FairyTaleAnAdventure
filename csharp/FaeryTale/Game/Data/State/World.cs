namespace FaeryTale.Game.Data;

/// The world around the hero: the map position, region, clock, light and magic timers.
public sealed class World
{
    public ushort MapX;
    public ushort MapY;
    public ushort Region;
    public ushort NewRegion;
    public ushort TargetRegion;
    public ushort OriginX;
    public ushort OriginY;
    public ushort TileX;
    public ushort TileY;
    public ushort DayClock;
    public short DayPeriod;
    public ushort LightLevel;
    public short LightTimer;
    public short FreezeTimer;
    public short SecretTimer;

    /// The number of the extent the hero is in.
    public int Extent;
    public ushort ExtentType;

    /// The map sector at the top left of the view (set when the map is drawn).
    public int SectorX;
    public int SectorY;

    /// How many frames the hero has stood on a bed.
    public short BedWait;

    /// Set when the hero walks into a locked door.
    public sbyte BumpedLockedDoor;
}

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
