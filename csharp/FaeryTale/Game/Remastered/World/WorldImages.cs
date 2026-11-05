using FaeryTale.Game.Render;

namespace FaeryTale.Game.Remastered;

/// A sprite picture decoded for the new drawer: a colour number (0-31) for every pixel and whether
/// the pixel is part of the figure or see-through.
public sealed record SpriteImage(int Width, int Height, byte[] Pixels, bool[] Opaque);

/// The tile pictures and sprite pictures of the game, taken out of their bitplanes (five layers of
/// bits, as the Amiga stored them) into one colour number per pixel, and kept so each is only
/// decoded once. They are thrown away when the game loads new pictures.
public sealed class WorldImages
{
    public const int TILE_WIDTH = 16;
    public const int TILE_HEIGHT = 32;
    private const int TILE_COUNT = 256;
    private const int PLANES = 5;

    // The tile pictures: one plane holds 256 tiles of 64 bytes (two bytes wide, 32 lines).
    private const int TILE_PLANE_BYTES = 16384;
    private const int TILE_BYTES = 64;
    private const int TILE_ROW_BYTES = 2;

    private const int BITS_PER_BYTE = 8;
    private const int TOP_BIT = 7;

    private readonly byte[]?[] tiles = new byte[TILE_COUNT][];
    private readonly Dictionary<(int Location, int Inum, int Bytes), SpriteImage?> sprites = [];
    private int tileVersion = -1;
    private int spriteVersion = -1;

    /// The colour numbers of a tile, 16 across and 32 down.
    public byte[] Tile(Graphics graphics, int tile)
    {
        if (graphics.TileVersion != tileVersion)
        {
            Array.Clear(tiles);
            tileVersion = graphics.TileVersion;
        }

        return tiles[tile & (TILE_COUNT - 1)] ??= DecodeTile(graphics.Image, tile & (TILE_COUNT - 1));
    }

    /// The picture number inum of a sprite sheet, or null when it is not in memory.
    public SpriteImage? Sprite(Graphics graphics, SeqInfo sheet, int inum)
    {
        if (graphics.SpriteVersion != spriteVersion)
        {
            sprites.Clear();
            spriteVersion = graphics.SpriteVersion;
        }

        (int, int, int) key = (sheet.Location, inum, sheet.Bytes);

        if (!sprites.TryGetValue(key, out SpriteImage? image))
        {
            image = DecodeSprite(graphics.Shape, sheet, inum);
            sprites[key] = image;
        }

        return image;
    }

    // The tile's colour numbers: for each pixel, the bit at its place in each of the five planes
    // (plane 0 is the lowest bit of the colour).
    private static byte[] DecodeTile(byte[] image, int tile)
    {
        byte[] pixels = new byte[TILE_WIDTH * TILE_HEIGHT];

        if (image.Length < PLANES * TILE_PLANE_BYTES)
        {
            return pixels;
        }

        for (int plane = 0; plane < PLANES; plane++)
        {
            int source = plane * TILE_PLANE_BYTES + tile * TILE_BYTES;

            for (int line = 0; line < TILE_HEIGHT; line++)
            {
                for (int column = 0; column < TILE_WIDTH; column++)
                {
                    int bit = (image[source + line * TILE_ROW_BYTES + column / BITS_PER_BYTE] >> (TOP_BIT - column % BITS_PER_BYTE)) & 1;

                    pixels[line * TILE_WIDTH + column] |= (byte)(bit << plane);
                }
            }
        }

        return pixels;
    }

    // A sprite sheet holds, for each picture, five planes of the picture and then, after all the
    // pictures, one plane of mask for each (a set bit is part of the figure).
    private static SpriteImage? DecodeSprite(byte[] shape, SeqInfo sheet, int inum)
    {
        int rowBytes = sheet.Width * TILE_ROW_BYTES;
        int planeBytes = sheet.Bytes;

        if (rowBytes <= 0 || planeBytes <= 0 || inum < 0)
        {
            return null;
        }

        int height = planeBytes / rowBytes;
        int picture = sheet.Location + planeBytes * PLANES * inum;
        int mask = sheet.MaskLoc + planeBytes * inum;

        if (picture + planeBytes * PLANES > shape.Length || mask + planeBytes > shape.Length)
        {
            return null;
        }

        int width = rowBytes * BITS_PER_BYTE;
        byte[] pixels = new byte[width * height];
        bool[] opaque = new bool[width * height];

        for (int line = 0; line < height; line++)
        {
            for (int column = 0; column < width; column++)
            {
                int at = line * rowBytes + column / BITS_PER_BYTE;
                int shift = TOP_BIT - column % BITS_PER_BYTE;

                opaque[line * width + column] = ((shape[mask + at] >> shift) & 1) != 0;

                for (int plane = 0; plane < PLANES; plane++)
                {
                    pixels[line * width + column] |= (byte)(((shape[picture + plane * planeBytes + at] >> shift) & 1) << plane);
                }
            }
        }

        return new SpriteImage(width, height, pixels, opaque);
    }
}
