using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Render;

/// The overview map of the whole world (the magic map spell): each sector is drawn as 16 x 16
/// pixels, 18 x 9 sectors.
public static class OverviewMap
{
    public const short OVERVIEW_COLUMNS = 18;
    public const short OVERVIEW_ROWS = 9;

    // Sectors across and down the whole map.
    public const short MAP_SECTORS_WIDE = 64;
    public const short MAP_WRAP = 128;
    public const int SECTOR_PIXELS = 16;
    public const int SECTOR_TILE_ROWS = 8;

    // Bytes between the two copies of each overview line (it is drawn twice as tall).
    public const int OVERVIEW_LINE_COPY = 40;
    public const int OVERVIEW_LINE_BYTES = 80;

    /// Bytes to skip at the end of each overview row of sectors.
    public const int OVERVIEW_ROW_SKIP = 40 * 15 + 4;

    // A sector is 256 pixels (a shift of 8). The overview starts 9 sectors to the left and 4 above
    // the hero (half its size), and its start wraps around the 128 sector map when below 32 or above 96.
    private const int SECTOR_SHIFT = 8;
    private const int HALF_OVERVIEW_WIDTH = 9;
    private const int HALF_OVERVIEW_HEIGHT = 4;
    private const short WRAP_LOW = 32;
    private const short WRAP_HIGH = 96;

    /// The region map has 128 sectors in each row (a shift of 7).
    private const int MAP_ROW_SHIFT = 7;

    // Each sector shows 16 x 8 pixels: a pixel row is 16 pixels, a line is two bytes, high byte first.
    private const int BYTE_SHIFT = 8;
    private const int TERRAIN_ENTRY_BYTES = 4;
    private const int TERRAIN_COLOR_BYTE = 3;
    private const int TILE_ROW_PIXELS = 16;

    /// The overview's top left sector for a map position, clamped to the map.
    public static (short X, short Y) OverviewOrigin(int mx, int my, ushort regionX, ushort regionY)
    {
        // Camera pixels to sectors (256 to a sector), then back to the left / top edge of the overview.
        short x = unchecked((short)(((ushort)mx >> SECTOR_SHIFT) - HALF_OVERVIEW_WIDTH - regionX));
        short y = unchecked((short)(((ushort)my >> SECTOR_SHIFT) - HALF_OVERVIEW_HEIGHT - regionY));

        x = Wrap(x);
        y = Wrap(y);

        return ((short)Math.Clamp((int)x, 0, MAP_SECTORS_WIDE - OVERVIEW_COLUMNS), (short)Math.Clamp((int)y, 0, MapLookup.SECTOR_ROWS_IN_REGION - OVERVIEW_ROWS));
    }

    /// Draws the overview map (the magic map spell): each sector is one 16 by 8 block of pixels
    /// drawn twice as tall, 18 x 9 sectors across the screen. Each pixel's colour is the tile's
    /// overview colour (terrain table byte 3). SectorX / SectorY tell the caller which sector is
    /// at the top left.
    public static void DrawOverviewMap(Video video, MapData map, World world, int mx, int my)
    {
        (short x, short y) = OverviewOrigin(mx, my, world.OriginX, world.OriginY);

        world.SectorX = x;
        world.SectorY = y;
        int offset = 0;

        for (int j = 0; j < OVERVIEW_ROWS; j++)
        {
            for (int number = 0; number < OVERVIEW_COLUMNS; number++)
            {
                // Index into the region map: the sector row times 128 plus the column.
                int index = world.OriginX + x + number + ((y + j) << MAP_ROW_SHIFT);
                int sector = map.MapByte((ushort)index);

                DrawOverviewSector(map, video, sector * MapLookup.SECTOR_BYTES, offset);
                offset += 2;
            }

            offset += OVERVIEW_ROW_SKIP;
        }
    }

    // A start that wrapped around the 128 sector map comes back in range.
    private static short Wrap(short value)
    {
        if (value < WRAP_LOW)
        {
            value += MAP_WRAP;
        }

        if (value > WRAP_HIGH)
        {
            value -= MAP_WRAP;
        }

        return value;
    }

    // Draws one sector of the overview, 8 lines of 16 pixels, offset bytes into the screen planes.
    private static void DrawOverviewSector(MapData map, Video video, int tiles, int offset)
    {
        for (int line = 0; line < SECTOR_TILE_ROWS; line++)
        {
            uint[] words = OverviewLineWords(map, tiles, line);

            for (int plane = 0; plane < words.Length; plane++)
            {
                Span<byte> bytes = video.Screen.DrawBitmap.Plane(plane);
                int at = offset + line * OVERVIEW_LINE_BYTES;

                // A 16 pixel row is two bytes, high byte first; the second copy is for the other page.
                bytes[at] = (byte)(words[plane] >> BYTE_SHIFT);
                bytes[at + OVERVIEW_LINE_COPY] = (byte)(words[plane] >> BYTE_SHIFT);
                bytes[at + 1] = (byte)words[plane];
                bytes[at + OVERVIEW_LINE_COPY + 1] = (byte)words[plane];
            }
        }
    }

    // The 16 pixels of one line of a sector, as one word for each bit plane. Each pixel's colour is
    // the overview colour of its tile.
    private static uint[] OverviewLineWords(MapData map, int tiles, int line)
    {
        uint[] words = new uint[MapTiles.PLANE_COUNT];

        for (int pixel = 0; pixel < SECTOR_PIXELS; pixel++)
        {
            int tile = map.SectorByte(tiles + line * TILE_ROW_PIXELS + pixel);
            byte color = map.TerrainByte(tile * TERRAIN_ENTRY_BYTES + TERRAIN_COLOR_BYTE);

            for (int plane = 0; plane < words.Length; plane++)
            {
                // Builds one bit per pixel for this bitplane: shift the row left and add the plane's bit of this colour.
                words[plane] = (words[plane] << 1) | (uint)((color >> plane) & 1);
            }
        }

        return words;
    }
}
