using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The kinds of memory a blit reads and writes.
public enum AreaKind
{
    /// A bit plane of the page being drawn.
    Plane,

    /// The blitter's queue region (saved backgrounds).
    Queue,

    /// The blitter's mask region (the scenery mask).
    Mask,

    /// The sprite sheets.
    Shapes,
}

/// An area of memory a blit reads and writes: a bit plane of the page being drawn (number
/// PlaneNumber), the work regions of the blitter, or the sprite sheets.
public readonly record struct Area(AreaKind Kind, byte PlaneNumber)
{
    public static readonly Area Queue = new(AreaKind.Queue, 0);
    public static readonly Area Mask = new(AreaKind.Mask, 0);
    public static readonly Area Shapes = new(AreaKind.Shapes, 0);

    public static Area Plane(int number) => new(AreaKind.Plane, (byte)number);
}

/// A place in an area: Pos bytes from its start (for a plane, from the first visible byte, so it
/// can reach a little before it).
public readonly record struct Loc(Area Area, long Pos)
{
    /// A place that is not used.
    public static readonly Loc NONE = new(Area.Queue, 0);

    /// The place bytes further on.
    public Loc Add(long bytes) => new(Area, Pos + bytes);
}

/// How one sprite is drawn this frame and where its background is saved (struct sshape).
/// CoffOffset is the byte offset of the sprite in a bitplane and Cmod the bytes to skip to the next
/// line; BlitSize is the blitter size word (height * 64 + width in words).
public struct SShape
{
    public Loc BackSave;
    public short SaveSize;
    public short BlitSize;
    public short Coff;
    public short Cmod;
}

/// A blitter job: places and strides for the channels, as the blitter registers held them.
internal sealed class Blit
{
    // Which channels a blit uses.
    internal const byte USE_A = 1;
    internal const byte USE_B = 2;
    internal const byte USE_C = 4;
    internal const byte USE_D = 8;

    // The minterms the game uses (result for each combination of the A, B and C bits).
    internal const uint D_EQUALS_B = 0xcc;
    internal const uint D_EQUALS_A_AND_NOT_C = 0x50;
    internal const uint D_EQUALS_A_THEN_B_ELSE_C = 0xca;

    internal const uint ALL_BITS = 0xffff;

    // BLTSIZE: the width in words is the low 6 bits, the height the 10 bits above them. A width of
    // 0 in the size word means 64 words.
    private const int WIDTH_BITS = 6;
    private const int WIDTH_MASK = 0x3f;
    private const int HEIGHT_MASK = 0x3ff;
    private const int MAX_WIDTH = 64;

    public Loc A = Loc.NONE;
    public Loc B = Loc.NONE;
    public Loc C = Loc.NONE;
    public Loc D = Loc.NONE;

    /// Bytes to skip at the end of each line.
    public int AMod;
    public int BMod;
    public int CMod;
    public int DMod;

    /// Masks for the first and last word of a line of A.
    public uint FirstWordMask = ALL_BITS;
    public uint LastWordMask = ALL_BITS;
    public int AShift;
    public int BShift;
    public byte Uses;
    public uint Minterm;

    /// In words and lines.
    public int Width;
    public int Height;

    public Blit(byte uses, uint minterm, short blitSize)
    {
        Uses = uses;
        Minterm = minterm;
        Width = WidthOf(blitSize);
        Height = HeightOf(blitSize);
    }

    /// BLTSIZE: height in the top 10 bits. The size word is height * 64 + width in words, so the
    /// height is everything above the low 6 bits.
    internal static int HeightOf(short size) => (size >> WIDTH_BITS) & HEIGHT_MASK;

    /// ... and the width in words in the low 6 bits (0 means 64).
    internal static int WidthOf(short size) => (size & WIDTH_MASK) == 0 ? MAX_WIDTH : size & WIDTH_MASK;
}

/// The memory a blit can reach: the bit planes being drawn on, the two work regions and the sprite
/// sheets. A read outside an area gives zeros, and a write outside it is dropped.
internal sealed class BlitMemory(Bitmap bitmap, byte[] queue, byte[] mask, byte[] shapes)
{
    private const int BYTE_BITS = 8;

    public byte Byte(Loc at)
    {
        (byte[] bytes, long baseOffset) = AreaBytes(at.Area);
        long index = baseOffset + at.Pos;

        return index >= 0 && index < bytes.Length ? bytes[index] : (byte)0;
    }

    public void SetByte(Loc at, byte value)
    {
        (byte[] bytes, long baseOffset) = AreaBytes(at.Area);
        long index = baseOffset + at.Pos;

        if (index >= 0 && index < bytes.Length)
        {
            bytes[index] = value;
        }
    }

    /// Reads a big-endian 16-bit word (the first byte is the high half).
    public uint ReadWord(Loc at) => ((uint)Byte(at) << BYTE_BITS) | Byte(at.Add(1));

    /// Writes a big-endian 16-bit word (the high byte is stored first).
    public void WriteWord(Loc at, uint value)
    {
        SetByte(at, (byte)(value >> BYTE_BITS));
        SetByte(at.Add(1), (byte)value);
    }

    // The bytes of an area, and how far in the position 0 of the area is.
    private (byte[] Bytes, long Base) AreaBytes(Area area) => area.Kind switch
    {
        AreaKind.Plane when area.PlaneNumber < bitmap.Depth => (bitmap.PaddedPlane(area.PlaneNumber), Bitmap.PAD),
        AreaKind.Plane => ([], 0),
        AreaKind.Queue => (queue, 0),
        AreaKind.Mask => (mask, 0),
        _ => (shapes, 0),
    };
}
