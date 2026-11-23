using System.Text;

namespace FaeryTale.Testing;

/// Saves a frame the way FTA_SHOT expects: a binary PPM image. Regression hashes are taken of
/// these bytes, so the format must match the Rust version exactly.
public static class PpmWriter
{
    // One pixel is a red, a green and a blue byte; red sits 16 bits up in 0xRRGGBB, green 8.
    private const int BYTES_PER_PIXEL = 3;
    private const int RED_SHIFT = 16;
    private const int GREEN_SHIFT = 8;

    /// The 0x00RRGGBB pixels as a binary PPM image of the given width.
    public static byte[] FrameAsPpm(ReadOnlySpan<uint> frame, int width, int height)
    {
        byte[] header = Encoding.ASCII.GetBytes($"P6\n{width} {height}\n255\n");
        byte[] image = new byte[header.Length + frame.Length * BYTES_PER_PIXEL];

        header.CopyTo(image, 0);

        for (int index = 0; index < frame.Length; index++)
        {
            int offset = header.Length + index * BYTES_PER_PIXEL;

            image[offset] = (byte)(frame[index] >> RED_SHIFT); // red is the top byte of 0xRRGGBB; shift right 16 moves it down
            image[offset + 1] = (byte)(frame[index] >> GREEN_SHIFT); // green is the middle byte; shift right 8
            image[offset + 2] = (byte)frame[index];
        }

        return image;
    }
}
