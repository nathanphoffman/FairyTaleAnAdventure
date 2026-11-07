using System.Buffers.Binary;
using System.Text;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Files;

/// Loads IFF ILBM pictures ("brushes") into a bitmap.
///
/// An IFF file is a FORM containing chunks. Each chunk is a 4-letter id, a big-endian length and
/// the data. This loader understands BMHD (picture size and compression), skips CMAP, GRAB, CAMG
/// and CRNG (colours and display modes, which the game does not use) and unpacks BODY (the
/// pixels). The pixels are stored one bitplane row at a time, which is also how the original
/// screen memory is laid out, so each row is unpacked straight into the matching plane of the
/// destination bitmap.
public static class IffPicture
{
    /// Planes the brush loader fills.
    private const int BRUSH_PLANES = 5;

    private const string FORM = "FORM";
    private const string ILBM = "ILBM";
    private const string BMHD = "BMHD";
    private const string BODY = "BODY";
    private static readonly string[] SKIPPED_CHUNKS = ["CAMG", "CRNG", "CMAP", "GRAB"];

    /// Bytes in a chunk id and in a chunk length.
    private const int CHUNK_FIELD_BYTES = 4;

    // A picture row is a whole number of 16 pixel words, as bytes: round up, divide by 8, make even.
    private const int WORD_ROUNDING = 15;
    private const int PIXELS_PER_BYTE = 8;

    /// Loads the picture name into bitmap at byte column x, row y. Returns false if the file is
    /// missing or not a picture the game can read. Used for the title pages, the story pictures,
    /// the text panel background and the winning picture.
    public static bool LoadPicture(string name, Bitmap bitmap, int x, int y)
    {
        string? path = Paths.Resolve(name, false);

        if (path is null || !File.Exists(path))
        {
            return false;
        }

        return ReadForm(File.ReadAllBytes(path), bitmap, x + bitmap.BytesPerRow * y);
    }

    /// Reads the chunks of an IFF file: the header, then the picture body into bitmap at byte
    /// offset. Returns false if this is not a picture file the game can read.
    internal static bool ReadForm(byte[] file, Bitmap bitmap, int offset)
    {
        Chunks chunks = new(file);

        if (chunks.Id() != FORM || chunks.Length() is not { } formLength)
        {
            return false;
        }

        long left = formLength;
        Header header = new();

        while (left != 0)
        {
            if (chunks.Id() is not { } id)
            {
                return false;
            }

            left -= CHUNK_FIELD_BYTES;

            if (id == ILBM)
            {
                continue; // just the form type, no data
            }

            ChunkResult result = ReadChunk(chunks, id, ref header, ref left, (bitmap, offset));

            if (result == ChunkResult.Failed)
            {
                return false;
            }

            if (result == ChunkResult.Done)
            {
                break;
            }
        }

        return true;
    }

    /// What reading one chunk of a form led to.
    private enum ChunkResult
    {
        Continue,

        /// The body was read: the picture is finished.
        Done,

        Failed,
    }

    // Reads the chunk with this id: the body is unpacked into the bitmap, the header is kept and
    // the chunks the game has no use for are skipped. left is how much of the form remains.
    private static ChunkResult ReadChunk(Chunks chunks, string id, ref Header header, ref long left, (Bitmap Bitmap, int Offset) target)
    {
        if (id == BODY)
        {
            if (chunks.Length() is not { } bodyLength)
            {
                return ChunkResult.Failed;
            }

            UnpackBody(header, new Body(chunks.Take(bodyLength), header.Compressed), target.Bitmap, target.Offset);

            return ChunkResult.Done;
        }

        if ((id != BMHD && !SKIPPED_CHUNKS.Contains(id)) || chunks.Length() is not { } length)
        {
            return ChunkResult.Failed;
        }

        left -= length + CHUNK_FIELD_BYTES;
        byte[] data = chunks.Take(length);

        if (id == BMHD)
        {
            header = Header.Read(data);
        }

        return ChunkResult.Continue;
    }

    // Unpacks the BODY into the bitmap, starting offset bytes into each plane: each picture row is
    // unpacked once per bitplane (up to 5, fewer if the bitmap is shallower).
    private static void UnpackBody(Header header, Body body, Bitmap bitmap, int offset)
    {
        int rowBytes = header.RowBytes;
        int bytesPerRow = bitmap.BytesPerRow;
        byte[] line = new byte[rowBytes];

        for (int row = 0; row < header.Height; row++)
        {
            int start = offset + row * bytesPerRow;

            for (int plane = 0; plane < BRUSH_PLANES; plane++)
            {
                if (bitmap.Depth > plane)
                {
                    body.UnpackLine(line);
                    Span<byte> destination = bitmap.Plane(plane);

                    if (start >= 0 && start + rowBytes <= destination.Length)
                    {
                        line.CopyTo(destination[start..]);
                    }
                }
            }
        }
    }

    // What the loader needs from the ILBM bitmap header chunk (the file is big-endian).
    private sealed class Header
    {
        public int Width;
        public int Height;
        public bool Compressed;

        /// Bytes in one unpacked scan line.
        public int RowBytes => (Width + WORD_ROUNDING) / PIXELS_PER_BYTE & ~1;

        // Reads the sizes at the start of the chunk's data: the width and height, then (after the
        // position, plane count and masking fields) the compression type.
        public static Header Read(byte[] data)
        {
            int Be16(int at) => at + 2 <= data.Length ? BinaryPrimitives.ReadUInt16BigEndian(data.AsSpan(at)) : 0;

            return new Header { Width = Be16(0), Height = Be16(2), Compressed = data.Length > 10 && data[10] != 0 };
        }
    }

    // The compressed or plain pixel data of a BODY chunk, read row by row.
    private sealed class Body(byte[] data, bool compressed)
    {
        private int at;

        public int Position => at;

        // Unpacks the next scan line (destination.Length bytes).
        public void UnpackLine(byte[] destination)
        {
            if (compressed)
            {
                UnpackByteRun(destination);

                return;
            }

            int end = Math.Min(at + destination.Length, data.Length);
            int start = Math.Min(at, end);

            data.AsSpan(start, end - start).CopyTo(destination);
            at = end;
        }

        // Unpacks one ByteRun1 scan line, stopping after destination.Length bytes.
        //
        // ByteRun1: a control byte n from 0 to 127 means the next n+1 bytes are copied as they are;
        // from -127 to -1 means the next byte is repeated 1-n times.
        private void UnpackByteRun(byte[] destination)
        {
            int written = 0;

            while (written < destination.Length)
            {
                if (at >= data.Length)
                {
                    return;
                }

                sbyte control = unchecked((sbyte)data[at++]);

                if (control >= 0)
                {
                    for (int count = 0; count <= control; count++)
                    {
                        Put(destination, written++, at < data.Length ? data[at] : (byte)0);
                        at++;
                    }
                }
                else
                {
                    byte value = at < data.Length ? data[at] : (byte)0;

                    at++;

                    for (int count = 0; count <= -(int)control; count++)
                    {
                        Put(destination, written++, value);
                    }
                }
            }
        }

        // Stores a byte unless the run has gone past the end of the line.
        private static void Put(byte[] destination, int index, byte value)
        {
            if (index < destination.Length)
            {
                destination[index] = value;
            }
        }
    }

    // Reads chunk ids and lengths from the file.
    private sealed class Chunks(byte[] data)
    {
        private int at;

        public string? Id()
        {
            if (at + CHUNK_FIELD_BYTES > data.Length)
            {
                return null;
            }

            string id = Encoding.ASCII.GetString(data, at, CHUNK_FIELD_BYTES);

            at += CHUNK_FIELD_BYTES;

            return id;
        }

        public long? Length()
        {
            if (at + CHUNK_FIELD_BYTES > data.Length)
            {
                return null;
            }

            uint length = BinaryPrimitives.ReadUInt32BigEndian(data.AsSpan(at));

            at += CHUNK_FIELD_BYTES;

            return length;
        }

        // The next length bytes, which are skipped over.
        public byte[] Take(long length)
        {
            int end = (int)Math.Min(at + length, data.Length);
            byte[] bytes = data.AsSpan(Math.Min(at, end), end - Math.Min(at, end)).ToArray();

            at = end;

            return bytes;
        }
    }

    /// Test access to the byte-run unpacker.
    internal static (byte[] Line, int Position) UnpackForTest(byte[] data, bool compressed, int lineLength)
    {
        Body body = new(data, compressed);
        byte[] line = new byte[lineLength];

        body.UnpackLine(line);

        return (line, body.Position);
    }

    /// Test access to the header row size.
    internal static int RowBytesForTest(int width) => new Header { Width = width }.RowBytes;
}
