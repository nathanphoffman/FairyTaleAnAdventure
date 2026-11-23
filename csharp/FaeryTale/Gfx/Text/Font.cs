using System.Buffers.Binary;

namespace FaeryTale.Gfx;

/// Fonts: the built-in 8x8 font and bitmap fonts read from an Amiga font file.
public abstract class Font
{
    /// The built-in font: a fixed 8x8 stand-in for the original's built-in font (Topaz 8).
    public static Font Builtin { get; } = new BuiltinFont();

    /// Reads an Amiga disk font (fonts/Amber/9): an executable file whose only hunk holds a
    /// DiskFontHeader. Offsets inside it are relative to the start of that hunk's data. Returns
    /// null if the file is missing or not a font.
    public static Font? Load(string path)
    {
        try
        {
            return FromHunkFile(File.ReadAllBytes(path));
        }
        catch (IOException problem)
        {
            Console.Error.WriteLine($"Could not read the font file {path}: {problem.Message}");

            return null;
        }
    }

    /// Reads a font from the bytes of a font file, or null if they are not a font.
    internal static Font? FromHunkFile(byte[] file)
    {
        if (Be32(file, 0) != HUNK_HEADER)
        {
            return null;
        }

        // HUNK_HEADER: 0, table size, first, last, one size per hunk, then HUNK_CODE, size, data.
        if (Be32(file, HUNK_TABLE_SIZE_OFFSET) is not { } table)
        {
            return null;
        }

        int hunk = HUNK_HEADER_FIXED + LONG_BYTES * (int)table;

        if (Be32(file, hunk) != HUNK_CODE || hunk + HUNK_CODE_PREFIX > file.Length)
        {
            return null;
        }

        return ReadFontTables(file.AsSpan(hunk + HUNK_CODE_PREFIX).ToArray());
    }

    /// Draws character character with its baseline at the pen and returns how far the pen moves.
    public abstract int DrawGlyph(Bitmap bitmap, Pen drawingPen, byte character, int planes);

    /// How far the pen moves across after a character (its width with the space after it).
    public abstract int AdvanceOf(byte character);

    // AmigaDOS hunk file markers.
    private const uint HUNK_HEADER = 0x3f3;
    private const uint HUNK_CODE = 0x3e9;

    // Bytes before the per-hunk size table in a HUNK_HEADER, and bytes of HUNK_CODE tag and size.
    private const int HUNK_HEADER_FIXED = 20;
    private const int HUNK_CODE_PREFIX = 8;
    private const int LONG_BYTES = 4;

    // Where the table size sits in the hunk header, and how big one entry of the glyph location table is
    // (a 16 bit bit-offset then a 16 bit width).
    private const int HUNK_TABLE_SIZE_OFFSET = 8;
    private const int CHAR_LOC_ENTRY_BYTES = 4;

    // Where the TextFont sits inside the DiskFontHeader, and where its fields sit inside that.
    private const int TEXTFONT_OFFSET = 58;
    private const int TF_YSIZE = 20;
    private const int TF_BASELINE = 26;
    private const int TF_LOCHAR = 32;
    private const int TF_HICHAR = 33;
    private const int TF_CHARDATA = 34;
    private const int TF_MODULO = 38;
    private const int TF_CHARLOC = 40;
    private const int TF_CHARSPACE = 44;
    private const int TF_CHARKERN = 48;

    internal static ushort? Be16(byte[] data, int offset) =>
        offset >= 0 && offset + 2 <= data.Length ? BinaryPrimitives.ReadUInt16BigEndian(data.AsSpan(offset)) : null;

    internal static uint? Be32(byte[] data, int offset) =>
        offset >= 0 && offset + LONG_BYTES <= data.Length ? BinaryPrimitives.ReadUInt32BigEndian(data.AsSpan(offset)) : null;

    // Cuts the glyph strip and the three per-glyph tables out of a font segment.
    private static BitmapFont? ReadFontTables(byte[] segment)
    {
        int tf = TEXTFONT_OFFSET;

        if (Be16(segment, tf + TF_YSIZE) is not { } height
            || Be16(segment, tf + TF_BASELINE) is not { } baseline
            || tf + TF_HICHAR >= segment.Length
            || Be32(segment, tf + TF_CHARDATA) is not { } charData
            || Be16(segment, tf + TF_MODULO) is not { } modulo
            || Be32(segment, tf + TF_CHARLOC) is not { } charLoc
            || Be32(segment, tf + TF_CHARSPACE) is not { } charSpace
            || Be32(segment, tf + TF_CHARKERN) is not { } charKern)
        {
            return null;
        }

        byte low = segment[tf + TF_LOCHAR];
        byte high = segment[tf + TF_HICHAR];
        int count = high - low + 2; // plus the "default" glyph at the end
        int dataLength = modulo * height;

        if ((long)charData + dataLength > segment.Length)
        {
            return null;
        }

        BitmapFont font = new(height, baseline, low, high, modulo, segment.AsSpan((int)charData, dataLength).ToArray(), count);

        for (int index = 0; index < count; index++)
        {
            if (Be16(segment, (int)charLoc + index * CHAR_LOC_ENTRY_BYTES) is not { } offset
                || Be16(segment, (int)charLoc + index * CHAR_LOC_ENTRY_BYTES + 2) is not { } width
                || Be16(segment, (int)charSpace + index * 2) is not { } space
                || Be16(segment, (int)charKern + index * 2) is not { } kern)
            {
                return null;
            }

            font.SetGlyph(index, offset, width, space, (short)kern);
        }

        return font;
    }

    // The built-in 8x8 font.
    private sealed class BuiltinFont : Font
    {
        // Size of the cells, and how far the baseline sits below the top.
        private const int SIZE = 8;
        private const int BASELINE = 6;
        private const byte ASCII_MASK = 0x7f;

        public override int AdvanceOf(byte character) => SIZE;

        public override int DrawGlyph(Bitmap bitmap, Pen drawingPen, byte character, int planes)
        {
            int topEdge = drawingPen.Y - BASELINE;
            byte[] glyph = Font8x8.BasicLegacy[character & ASCII_MASK];
            bool jam2 = (drawingPen.Mode & Drawing.JAM2) != 0;

            for (int rowIndex = 0; rowIndex < SIZE; rowIndex++)
            {
                for (int column = 0; column < SIZE; column++)
                {
                    bool on = ((glyph[rowIndex] >> column) & 1) != 0; // bit 0 is leftmost

                    if (on)
                    {
                        bitmap.Plot(drawingPen.X + column, topEdge + rowIndex, drawingPen.Fg, false, planes);
                    }
                    else if (jam2)
                    {
                        bitmap.Plot(drawingPen.X + column, topEdge + rowIndex, drawingPen.Bg, false, planes);
                    }
                }
            }

            return SIZE;
        }
    }

    // A bitmap font in the Amiga's layout: one wide strip of glyph bitmaps (modulo bytes per row)
    // with a table giving each glyph's bit offset and width, its advance and its kerning.
    private sealed class BitmapFont(int height, int baseline, byte low, byte high, int modulo, byte[] data, int count) : Font
    {
        private const byte LEFTMOST_BIT_FIRST = 0x80;
    private const int BITS_PER_BYTE = 8;

        private readonly (ushort Offset, ushort Width)[] locations = new (ushort, ushort)[count];
        private readonly ushort[] spaces = new ushort[count];
        private readonly short[] kerns = new short[count];

        public void SetGlyph(int index, ushort offset, ushort width, ushort space, short kern)
        {
            locations[index] = (offset, width);
            spaces[index] = space;
            kerns[index] = kern;
        }

        public override int AdvanceOf(byte character) =>
            spaces[character >= low && character <= high ? character - low : high - low + 1];

        public override int DrawGlyph(Bitmap bitmap, Pen drawingPen, byte character, int planes)
        {
            // The "default" glyph sits after the last real one.
            int index = character >= low && character <= high ? character - low : high - low + 1;
            (ushort bitOffset, ushort bitWidth) = locations[index];
            int advance = spaces[index];
            int kern = kerns[index];
            int topEdge = drawingPen.Y - baseline;
            bool jam2 = (drawingPen.Mode & Drawing.JAM2) != 0;

            for (int rowIndex = 0; rowIndex < height; rowIndex++)
            {
                if (jam2)
                {
                    for (int positionX = 0; positionX < advance; positionX++)
                    {
                        bitmap.Plot(drawingPen.X + positionX, topEdge + rowIndex, drawingPen.Bg, false, planes);
                    }
                }

                for (int column = 0; column < bitWidth; column++)
                {
                    int bit = bitOffset + column;

                    // The leftmost pixel of a byte is its top bit.
                    if ((data[rowIndex * modulo + bit / BITS_PER_BYTE] & (LEFTMOST_BIT_FIRST >> (bit % BITS_PER_BYTE))) != 0)
                    {
                        bitmap.Plot(drawingPen.X + kern + column, topEdge + rowIndex, drawingPen.Fg, false, planes);
                    }
                }
            }

            return advance;
        }
    }
}
