using System.Text;

namespace FaeryTale.Testing;

/// Saves a frame the way FTA_SHOT expects: a binary PPM image. Regression hashes are taken of
/// these bytes, so the format must match the Rust version exactly.
public static class PpmWriter
{
    /// The 0x00RRGGBB pixels as a binary PPM image of the given width.
    public static byte[] FrameAsPpm(ReadOnlySpan<uint> frame, int width, int height)
    {
        byte[] header = Encoding.ASCII.GetBytes($"P6\n{width} {height}\n255\n");
        byte[] image = new byte[header.Length + frame.Length * 3];

        header.CopyTo(image, 0);

        for (int index = 0; index < frame.Length; index++)
        {
            int at = header.Length + index * 3;

            image[at] = (byte)(frame[index] >> 16);
            image[at + 1] = (byte)(frame[index] >> 8);
            image[at + 2] = (byte)frame[index];
        }

        return image;
    }
}
