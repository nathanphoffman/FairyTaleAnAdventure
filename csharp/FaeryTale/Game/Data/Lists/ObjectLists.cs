namespace FaeryTale.Game.Data;

/// One of the object lists: the global one or the list of a region.
public readonly record struct ObjectList(int Region)
{
    public static readonly ObjectList Global = new(-1);

    public bool IsGlobal => Region < 0;
}

/// The object lists the game works on: one for each region and the global one. The lists have
/// room to grow: random treasure is added after the objects in use.
public sealed class ObjectLists
{
    public const int REGION_COUNT = 10;

    // How many objects the lists of the regions with more than one start out using.
    private const short REGION_0_USED = 3;
    private const short REGION_2_USED = 5;
    private const short REGION_3_USED = 12;
    private const short REGION_4_USED = 3;
    private const short REGION_5_USED = 5;
    private const short REGION_8_PLACED_USED = 61;
    private const short REGION_8_EXTRA_USED = 16;
    private const short REGION_9_USED = 9;

    public WorldObject[][] Regions =
    [
        Copy(ObjectTables.InitialObList0),
        Copy(ObjectTables.InitialObList1),
        Copy(ObjectTables.InitialObList2),
        Copy(ObjectTables.InitialObList3),
        Copy(ObjectTables.InitialObList4),
        Copy(ObjectTables.InitialObList5),
        Copy(ObjectTables.InitialObList6),
        Copy(ObjectTables.InitialObList7),
        Copy(ObjectTables.InitialObList8),
        Copy(ObjectTables.InitialObList9),
    ];

    public WorldObject[] Global = Copy(ObjectTables.InitialObListg);

    /// How many objects each region's list uses, and whether random treasure has been scattered there.
    public short[] Used = [REGION_0_USED, 1, REGION_2_USED, REGION_3_USED, REGION_4_USED, REGION_5_USED, 1, 1, REGION_8_PLACED_USED + REGION_8_EXTRA_USED, REGION_9_USED];
    public short[] Distributed = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1];

    /// The objects of which.
    public WorldObject[] List(ObjectList which) => which.IsGlobal ? Global : Region(which.Region);

    /// The list of region region (empty for a region without one).
    public WorldObject[] Region(int region) =>
        region >= 0 && region < Regions.Length ? Regions[region] : Array.Empty<WorldObject>();

    private static WorldObject[] Copy(WorldObject[] list) => (WorldObject[])list.Clone();
}
