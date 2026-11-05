namespace FaeryTale.Game.Data;

/// The map data the rules read: the sectors and the region map, the terrain table, and the
/// overview map.
public sealed class MapData
{
    /// Where the region map starts in Sector.
    public const int MAP_OFFSET = 128 * 256;

    private const int MINIMAP_SIZE = 120;

    // The memory areas the game loads its map data into (with a little slack, as the original had).
    private const int AREA_SLACK = 128;
    private const int SECTOR_AREA_SIZE = MAP_OFFSET + 4096;
    private const int TERRAIN_AREA_SIZE = 1024;

    /// The 256 sectors of 128 tile numbers each, then (from MAP_OFFSET) the region map.
    public byte[] Sector = Array.Empty<byte>();

    /// The terrain table: four bytes per tile.
    public byte[] Terrain = Array.Empty<byte>();

    /// The overview map of the world sectors.
    public short[] Minimap = new short[MINIMAP_SIZE];

    /// Which disk blocks are loaded now (image banks, terrain, sectors, region, characters).
    public Need LoadedBlocks = new() { Terra1 = 1, Terra2 = 2 };

    /// Makes the zeroed sector and terrain areas the region loader reads into.
    public void Allocate()
    {
        Sector = new byte[SECTOR_AREA_SIZE + AREA_SLACK];
        Terrain = new byte[TERRAIN_AREA_SIZE + AREA_SLACK];
    }

    /// The sector number at byte index of the region map (0 outside it).
    public byte MapByte(int index) => ByteAt(Sector, MAP_OFFSET + index);

    /// Changes the sector number at byte index of the region map.
    public void SetMapByte(int index, byte value) => SetByteAt(Sector, MAP_OFFSET + index, value);

    /// The tile number at byte index of the sector table (0 outside it).
    public byte SectorByte(int index) => ByteAt(Sector, index);

    /// Changes the tile number at byte index of the sector table.
    public void SetSectorByte(int index, byte value) => SetByteAt(Sector, index, value);

    /// Byte index of the terrain table (0 outside it).
    public byte TerrainByte(int index) => ByteAt(Terrain, index);

    private static byte ByteAt(byte[] bytes, int index) => index >= 0 && index < bytes.Length ? bytes[index] : (byte)0;

    private static void SetByteAt(byte[] bytes, int index, byte value)
    {
        if (index >= 0 && index < bytes.Length)
        {
            bytes[index] = value;
        }
    }
}
