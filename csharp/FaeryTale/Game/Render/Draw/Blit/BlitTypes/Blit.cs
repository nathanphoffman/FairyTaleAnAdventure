using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

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
