namespace FaeryTale.Gfx;

/// Planar bitmaps: the bit planes of a picture, each pixel's colour number made of one bit from
/// each plane. Plane p holds bit p of every pixel's colour number.
public sealed class Bitmap
{
    /// Extra bytes before and after each plane. The sprite blitter reads and writes a word or two
    /// outside the visible area (as the original's memory allowed), so the planes have room for that.
    public const int PAD = 256;

    private const int PIXELS_PER_BYTE = 8;
    private const byte LEFTMOST_PIXEL = 0x80;
    private const byte ALL_PIXELS_SET = 0xff;

    // Rows are a whole number of 16 pixel words (2 bytes each).
    private const int WORD_PIXELS = 16;
    private const int WORD_BYTES = 2;

    private readonly byte[][] planes;

    private Bitmap(int bytesPerRow, int rows, byte[][] planes)
    {
        BytesPerRow = bytesPerRow;
        Rows = rows;
        this.planes = planes;
    }

    /// A blank bitmap of width by rows pixels with depth planes.
    public Bitmap(int width, int rows, int depth)
    {
        BytesPerRow = BytesPerRowFor(width);
        Rows = rows;
        planes = new byte[depth][];

        for (int plane = 0; plane < depth; plane++)
        {
            planes[plane] = new byte[BytesPerRow * rows + 2 * PAD];
        }
    }

    public int BytesPerRow { get; }

    public int Rows { get; }

    public int Depth => planes.Length;

    /// Width in pixels (a whole number of words).
    public int Width => BytesPerRow * PIXELS_PER_BYTE;

    private int VisibleLength => BytesPerRow * Rows;

    /// No planes at all; the state before a bitmap is set up.
    public static Bitmap Empty() => new(0, 0, []);

    /// Bytes in a row of width pixels.
    public static int BytesPerRowFor(int width) => (width + WORD_PIXELS - 1) / WORD_PIXELS * WORD_BYTES;

    /// A one plane bitmap whose rows are the first bytes of data (the rest of the plane is blank).
    public static Bitmap FromPlane(int width, int rows, ReadOnlySpan<byte> data)
    {
        Bitmap bitmap = new(width, rows, 1);
        Span<byte> plane = bitmap.Plane(0);

        data[..Math.Min(plane.Length, data.Length)].CopyTo(plane);

        return bitmap;
    }

    /// An independent copy.
    public Bitmap Clone() => new(BytesPerRow, Rows, planes.Select(plane => (byte[])plane.Clone()).ToArray());

    /// The visible bytes of plane p, row after row.
    public Span<byte> Plane(int plane) => planes[plane].AsSpan(PAD, VisibleLength);

    /// Plane p with its padding: the visible bytes start PAD bytes in. The sprite blitter may reach
    /// a little outside the visible bytes, into the padding.
    public byte[] PaddedPlane(int plane) => planes[plane];

    /// True when the point is inside the bitmap.
    public bool Contains(int positionX, int positionY) => positionX >= 0 && positionY >= 0 && positionY < Rows && positionX < Width;

    /// Bit p of the pixel at (x, y), which must be inside the bitmap.
    public bool Bit(int plane, int positionX, int positionY)
    {
        (int byteIndex, byte mask) = PixelLocation(positionX, positionY);

        return (Plane(plane)[byteIndex] & mask) != 0;
    }

    public void SetBit(int plane, int positionX, int positionY, bool on)
    {
        (int byteIndex, byte mask) = PixelLocation(positionX, positionY);
        Span<byte> bytes = Plane(plane);

        if (on)
        {
            bytes[byteIndex] |= mask;
        }
        else
        {
            bytes[byteIndex] &= (byte)~mask;
        }
    }

    /// The colour number of the pixel at (x, y), which must be inside the bitmap.
    public int Pixel(int positionX, int positionY)
    {
        (int byteIndex, byte mask) = PixelLocation(positionX, positionY);
        int index = 0;

        for (int plane = 0; plane < Depth; plane++)
        {
            if ((Plane(plane)[byteIndex] & mask) != 0)
            {
                index |= 1 << plane; // each plane with the pixel set adds 2 to the power of its number
            }
        }

        return index;
    }

    /// Draws one pixel in colour pen on the first planes planes, or flips it (complement). Pixels
    /// outside the bitmap are ignored.
    public void Plot(int positionX, int positionY, byte penColor, bool complement, int planeCount)
    {
        if (!Contains(positionX, positionY))
        {
            return;
        }

        for (int plane = 0; plane < Math.Min(Depth, planeCount); plane++)
        {
            // Bit p of the pen number says whether this plane gets the pixel.
            bool on = complement ? !Bit(plane, positionX, positionY) : ((penColor >> plane) & 1) != 0;

            SetBit(plane, positionX, positionY, on);
        }
    }

    /// Fills the first planes planes with colour pen.
    public void Clear(byte penColor, int planeCount)
    {
        for (int plane = 0; plane < Math.Min(Depth, planeCount); plane++)
        {
            // A plane is all ones if the pen has that bit set, and all zeros otherwise.
            Plane(plane).Fill(((penColor >> plane) & 1) != 0 ? ALL_PIXELS_SET : (byte)0);
        }
    }

    // The byte holding the pixel and the pixel's bit inside it: the leftmost pixel is the top bit.
    private (int Byte, byte Mask) PixelLocation(int positionX, int positionY) =>
        (positionY * BytesPerRow + positionX / PIXELS_PER_BYTE, (byte)(LEFTMOST_PIXEL >> (positionX % PIXELS_PER_BYTE)));  // shifting the top bit right x places walks the mask across the byte
}
