using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Land;

/// Which part of the world is loaded: working out the region the camera is in and loading its map,
/// tiles and terrain from disk.
///
/// The world is split into 8 outdoor regions (a 2 x 4 grid of 64 x 32 sector blocks) plus two
/// indoor ones (8: buildings, 9: dungeons and caves). World.Region is the region now loaded;
/// World.NewRegion is the one being loaded, or NO_REGION when nothing is waiting to load.
///
/// The original loaded in the background and checked whether the disk was done. Every read is
/// finished when it returns now, so there is nothing to wait for here.
public static class Regions
{
    /// NewRegion value meaning no region is waiting to be loaded.
    public const ushort NO_REGION = 10;

    /// First region number that is indoors.
    public const ushort FIRST_INDOOR_REGION = 8;

    /// A region is 64 sectors wide and 32 high; there are 4 rows of them.
    public const int REGION_WIDTH_SHIFT = 6;
    public const int REGION_HEIGHT_SHIFT = 5;

    // Block on the disk where the terrain tables start.
    private const short TERRA_BLOCK = 149;

    // Blocks for the sector data and the region map.
    private const short SECTOR_BLOCKS = 64;
    private const short REGION_MAP_BLOCKS = 8;

    // Terrain tables are one block (512 bytes) each; the second goes after the first.
    private const int TERRA_TABLE_SIZE = 512;
    private const int IMAGE_BANKS = 4;

    // The desert region and the inventory slot of the golden statue count. Until the hero has five
    // statues, the door into the desert fort is blocked by four map tiles (254) at this spot.
    private const ushort DESERT_REGION = 4;
    private const int MAP_ROW_BYTES = 128;
    private const int STATUE_SLOT = 25;
    private const byte STATUES_NEEDED = 5;
    private const int DESERT_DOOR_TILE = 11 * MAP_ROW_BYTES + 26;
    private const byte BLOCKED_TILE = 254;

    // The screen's middle is this far from its top left corner, in pixels, and a sector is 256
    // pixels (a shift of 8).
    private const uint CENTER_OFFSET_X = 151;
    private const uint CENTER_OFFSET_Y = 64;
    private const int SECTOR_PIXELS_SHIFT = 8;
    private const uint REGION_ROW_MASK = 3;

    // The map position wraps at these sizes; new regions are only loaded between these rows, and
    // not in the strip of x positions that straddles the join.
    private const uint MAP_X_WRAP = 0x3fff;
    private const uint MAP_Y_WRAP = 0x1fff;
    private const uint MAP_Y_MIN_FOR_LOAD = 109;
    private const uint MAP_Y_MAX_FOR_LOAD = 7936;
    private const uint DEAD_STRIP_FIRST_X = 16128;
    private const uint DEAD_STRIP_LAST_X = 16156;

    /// Works out which region the camera is in and starts loading it if it changed; sets the
    /// region origin (where the loaded region starts, in sectors); then fills the minimap.
    ///
    /// Outdoors the region comes from the middle of the screen: bit 6 of the sector column gives
    /// west or east, and two bits of the sector row give which of four rows.
    public static void RefreshMinimap(Data.Game game, IOutput output)
    {
        if (game.World.Region < FIRST_INDOOR_REGION)
        {
            FollowOutdoorRegion(game, output);
        }
        else
        {
            game.World.TargetRegion = game.World.Region;
        }

        UpdateRegionOrigin(game.World);
        MapLookup.FillMinimap(game, game.World.TileX, game.World.TileY);
    }

    /// The sector (column, row) at the middle of the screen: the map position is the screen's top
    /// left corner, so the middle is 151 pixels across and 64 down from it, and a sector is 256
    /// pixels.
    public static (uint Column, uint Row) CenterSector(World world) =>
        ((world.MapX + CENTER_OFFSET_X) >> SECTOR_PIXELS_SHIFT, (world.MapY + CENTER_OFFSET_Y) >> SECTOR_PIXELS_SHIFT);

    /// Loads the pending region completely (LoadRegionPiece does one piece per call).
    public static void LoadRegionNow(Data.Game game, IOutput output)
    {
        while (game.World.NewRegion < NO_REGION)
        {
            LoadRegionPiece(game, output);
        }
    }

    /// Loads the next missing piece of the region in NewRegion: sector data, region map, the two
    /// terrain tables, then one image bank per call. Each piece is only read if the loaded one is
    /// different. When everything is there, NewRegion becomes the current region.
    public static void LoadRegionPiece(Data.Game game, IOutput output)
    {
        if (game.World.NewRegion >= NO_REGION)
        {
            return;
        }

        Need need = Tables.RegionDiskBlocks[game.World.NewRegion];

        LoadChangedMapTables(game.Map, need);
        LoadChangedTerrain(game.Map, need);

        if (LoadNextImageBank(output, game.Map, need))
        {
            return;
        }

        BlockDesertDoorWithoutStatues(game);
        game.World.Region = game.World.NewRegion;
        game.World.NewRegion = NO_REGION;
    }

    // Outdoors: loads a pending region when the camera is in the loadable band, then works out the
    // region at the middle of the screen and starts loading it if it is not the current one.
    private static void FollowOutdoorRegion(Data.Game game, IOutput output)
    {
        if (game.World.NewRegion < NO_REGION)
        {
            uint xs = game.World.MapX & MAP_X_WRAP;
            uint ys = game.World.MapY & MAP_Y_WRAP;
            bool inDeadStrip = xs >= DEAD_STRIP_FIRST_X && xs <= DEAD_STRIP_LAST_X;

            if (ys > MAP_Y_MIN_FOR_LOAD && ys < MAP_Y_MAX_FOR_LOAD && !inDeadStrip)
            {
                LoadRegionNow(game, output);
            }
        }

        (uint column, uint row) = CenterSector(game.World);

        // The map is two regions wide (64 sectors each) and four high (32 sectors each).
        uint regionColumn = (column >> REGION_WIDTH_SHIFT) & 1;
        uint regionRow = (row >> REGION_HEIGHT_SHIFT) & REGION_ROW_MASK;

        game.World.TargetRegion = (ushort)(regionColumn + regionRow + regionRow);

        if (game.World.TargetRegion != game.World.Region)
        {
            game.World.NewRegion = game.World.TargetRegion;
            LoadRegionNow(game, output);
        }
    }

    // Where the target region starts, in sectors: 64 across, 32 down.
    private static void UpdateRegionOrigin(World world)
    {
        // Regions are numbered along the row: two to a row, so the row is the number halved.
        int regionRow = world.TargetRegion >> 1;
        int regionColumn = world.TargetRegion >= FIRST_INDOOR_REGION ? 0 : world.TargetRegion & 1;

        world.OriginX = (ushort)(regionColumn << REGION_WIDTH_SHIFT);
        world.OriginY = (ushort)(regionRow << REGION_HEIGHT_SHIFT);
    }

    /// Loads the sector table and the region map of a region into a map of its own (the new world
    /// drawer shows regions next to the loaded one this way).
    public static void LoadMapTables(MapData map, Need need)
    {
        Disk.LoadTrackRange((short)need.Sector, SECTOR_BLOCKS, map.Sector.AsSpan(0, MapData.MAP_OFFSET));
        Disk.LoadTrackRange((short)need.Region, REGION_MAP_BLOCKS, map.Sector.AsSpan(MapData.MAP_OFFSET));
    }

    // Loads the sector table and the region map if they changed.
    private static void LoadChangedMapTables(MapData map, Need need)
    {
        if (need.Sector != map.LoadedBlocks.Sector || need.Region != map.LoadedBlocks.Region)
        {
            LoadMapTables(map, need);
            map.LoadedBlocks.Sector = need.Sector;
            map.LoadedBlocks.Region = need.Region;
        }
    }

    // Loads the two terrain tables if they changed.
    private static void LoadChangedTerrain(MapData map, Need need)
    {
        if (need.Terra1 != map.LoadedBlocks.Terra1)
        {
            Disk.LoadTrackRange((short)(TERRA_BLOCK + need.Terra1), 1, map.Terrain.AsSpan(0, TERRA_TABLE_SIZE));
            map.LoadedBlocks.Terra1 = need.Terra1;
        }

        if (need.Terra2 != map.LoadedBlocks.Terra2)
        {
            Disk.LoadTrackRange((short)(TERRA_BLOCK + need.Terra2), 1, map.Terrain.AsSpan(TERRA_TABLE_SIZE));
            map.LoadedBlocks.Terra2 = need.Terra2;
        }
    }

    // Loads the first of the four image banks that differs from the loaded one. Returns true if
    // it loaded one (only one bank per call, so the game keeps running), false if all are up to date.
    private static bool LoadNextImageBank(IOutput output, MapData map, Need need)
    {
        for (int bank = 0; bank < IMAGE_BANKS; bank++)
        {
            if (need.Image[bank] != map.LoadedBlocks.Image[bank])
            {
                output.LoadImageBank(bank, (short)need.Image[bank]);
                map.LoadedBlocks.Image[bank] = need.Image[bank];

                return true;
            }
        }

        return false;
    }

    // Until the hero has enough statues, the desert fort door is covered with blocked tiles.
    private static void BlockDesertDoorWithoutStatues(Data.Game game)
    {
        if (game.World.NewRegion != DESERT_REGION || game.Hero.Stuff[STATUE_SLOT] >= STATUES_NEEDED)
        {
            return;
        }

        foreach (int offset in new[] { 0, 1, MAP_ROW_BYTES, MAP_ROW_BYTES + 1 })
        {
            game.Map.SetMapByte(DESERT_DOOR_TILE + offset, BLOCKED_TILE);
        }
    }
}
