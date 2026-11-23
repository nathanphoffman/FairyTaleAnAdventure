namespace FaeryTale.Game.Render;

/// The picture data the drawing code reads: the tile pictures, the shadow masks and the sprite sheets.
public sealed class Graphics
{
    /// Sprite sheets loaded at once (hero, objects, ...).
    public const int SPRITE_SHEETS = 7;

    /// The tile pictures: five planes of 256 tiles for each of the four image banks.
    public byte[] Image = [];

    /// The masks that hide sprites behind scenery tiles.
    public byte[] Shadow = [];

    /// The sprite sheets: each sheet is its frames (five planes each) followed by their masks.
    public byte[] Shape = [];

    /// Where the image and the mask of the sprite being drawn start in Shape.
    public int ShapeData;
    public int MaskBuffer;

    /// Where the next sprite sheet loads in Shape.
    public int NextShape;

    /// Counted up whenever the tile pictures or the sprite sheets are loaded, so drawers that keep
    /// decoded copies of them know when to throw them away.
    public int TileVersion;
    public int SpriteVersion;

    public SeqInfo[] SeqList = new SeqInfo[SPRITE_SHEETS];

    /// The size of the tile pictures: five planes, each holding the 64 tiles of all four banks.
    public const int IMAGE_BYTES = PLANES * IMAGE_PLANE_SIZE;

    private const int PLANES = 5;
    private const int IMAGE_PLANE_SIZE = 16384;
    private const int BANK_PLANE_SIZE = 4096;
    private const short BANK_PLANE_BLOCKS = 8;

    /// Loads one of the four image banks (the tiles) from the disk, starting at a block.
    public void LoadImageBank(int bank, short firstBlock)
    {
        TileVersion++;

        for (int plane = 0; plane < PLANES; plane++)
        {
            int from = bank * BANK_PLANE_SIZE + plane * IMAGE_PLANE_SIZE;

            Files.Disk.LoadTrackRange((short)(firstBlock + plane * BANK_PLANE_BLOCKS), BANK_PLANE_BLOCKS, Image.AsSpan(from));
        }
    }
}
