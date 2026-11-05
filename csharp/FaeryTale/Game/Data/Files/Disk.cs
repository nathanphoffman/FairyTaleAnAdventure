namespace FaeryTale.Game.Files;

/// Reading the game's data from the disk image. Nearly all the game's graphics, maps and sounds
/// are in one big file, "image", a dump of the original floppy disk (1760 blocks of 512 bytes).
/// The game asks for "N blocks starting at block B". In the original those reads ran in the
/// background; here every read is finished when the call returns.
public static class Disk
{
    /// Error code returned by Open when there is no image.
    public const int ERR_NO_FILE = 30;

    private const string IMAGE_FILE = "image";
    private const int BLOCK_SIZE = 512;

    private static FileStream? image;

    /// Opens the image file. Returns 0, or an error code.
    public static int Open()
    {
        string? path = Paths.Resolve(IMAGE_FILE, false);

        if (path is null || !File.Exists(path))
        {
            return ERR_NO_FILE;
        }

        image = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read);

        return 0;
    }

    /// Closes the image file.
    public static void Close()
    {
        image?.Dispose();
        image = null;
    }

    /// Reads count blocks from block first. A read past the end of the image is padded with zeros.
    public static byte[] ReadBlocks(short first, short count)
    {
        byte[] data = new byte[Math.Max((int)count, 0) * BLOCK_SIZE];

        ReadInto(first, data);

        return data;
    }

    /// Reads count blocks from block first into buffer (as many as fit in it).
    public static void LoadTrackRange(short first, short count, Span<byte> buffer)
    {
        int bytes = Math.Min(Math.Max((int)count, 0) * BLOCK_SIZE, buffer.Length);

        ReadInto(first, buffer[..bytes]);
    }

    // Reads the bytes at block first into buffer, as far as the image goes.
    private static void ReadInto(short first, Span<byte> buffer)
    {
        if (image is null)
        {
            return;
        }

        image.Seek((long)first * BLOCK_SIZE, SeekOrigin.Begin);

        int done = 0;

        while (done < buffer.Length)
        {
            int read = image.Read(buffer[done..]);

            if (read == 0)
            {
                break;
            }

            done += read;
        }
    }
}
