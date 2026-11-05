namespace FaeryTale.Game.Render;

/// Where a sprite sequence (all the frames of one kind of character) is in memory (struct
/// seq_info), and where its transparency mask is.
public struct SeqInfo
{
    public short Width;
    public short Height;
    public short Count;

    /// Where the frames start in the sprite memory, and where the masks start.
    public int Location;
    public int MaskLoc;
    public short Bytes;
}

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
}

/// The sprite blitter's work values for the sprite being drawn.
public sealed class BlitterRegisters
{
    public SShape SShape;
    public short ShadowMaskOffset;
    public short SpriteMaskOffset;
    public short SpriteMaskModulo;
    public short SpriteXShift;
    public short LastWordMask;
}

/// Everything about how the game is drawn: the screen, the picture data, the sprite blitter's
/// work values. The rules never read it, they only ask for things to be drawn.
public sealed partial class Video
{
    public Screen Screen = new();
    public Graphics Graphics = new();
    public BlitterRegisters Blitter = new();

    /// Where finished frames go. The platform phase supplies the real window; until then frames
    /// are only counted.
    public IDisplayDevice Device = new NullDisplayDevice();

    /// What the new world drawer of Remastered and Reimagined is told about each frame.
    public Remastered.WorldCapture Capture = new();
}
