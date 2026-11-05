namespace FaeryTale.Gfx;

/// A rectangle that is darkened: Darkness 0 leaves the picture alone, 255 makes it black.
public sealed record Shade(int X, int Y, int Width, int Height, byte Darkness);

/// A ready-made picture of 0xAARRGGBB pixels with its top left corner at (X, Y).
public sealed record Image(int X, int Y, int Width, int Height, uint[] Pixels);

/// A line of text with its left end and baseline at (X, Y), an 0xAARRGGBB colour and a scale:
/// each pixel of the 8x8 font is Scale x Scale overlay pixels.
public sealed record Text(string Content, int X, int Y, uint Color, int Scale);

/// Text and dark panels drawn over the finished frame, on a 640 x 400 grid: first the shades,
/// then the pictures, then the text on top.
public sealed class Overlay
{
    public const int WIDTH = 640;
    public const int GLYPH_SIZE = 8;
    public const uint WHITE = 0xffff_ffff;

    private const int HINT_RIGHT_EDGE = 632;
    private const int HINT_BASELINE = 390;
    private const int HINT_PADDING = 6;
    private const int GLYPH_BASELINE = 7;
    private const byte HINT_DARKNESS = 230;

    public List<Shade> Shades { get; init; } = new();

    public List<Image> Images { get; init; } = new();

    public List<Text> Texts { get; init; } = new();

    /// Draws the overlay on a frame width pixels wide (and so many rows tall).
    public void Draw(uint[] frame, int width)
    {
        if (width == 0)
        {
            return;
        }

        foreach (Shade shade in Shades)
        {
            DrawShade(frame, width, shade);
        }

        foreach (Image image in Images)
        {
            DrawImage(frame, width, image);
        }

        foreach (Text text in Texts)
        {
            DrawText(frame, width, text);
        }
    }

    public bool IsEmpty => Shades.Count == 0 && Images.Count == 0 && Texts.Count == 0;

    /// How wide the text is at the given scale.
    public static int TextWidth(string content, int scale) => content.Length * GLYPH_SIZE * scale;

    /// A short message on a dark box in the bottom right corner.
    public static Overlay CornerHint(string content)
    {
        int width = TextWidth(content, 1);
        int left = HINT_RIGHT_EDGE - width;

        Shade box = new(
            left - HINT_PADDING,
            HINT_BASELINE - GLYPH_BASELINE - HINT_PADDING,
            width + HINT_PADDING * 2,
            GLYPH_SIZE + HINT_PADDING * 2,
            HINT_DARKNESS);

        return new Overlay
        {
            Shades = { box },
            Texts = { new Text(content, left, HINT_BASELINE, WHITE, 1) },
        };
    }

    // Darkens the pixels of a shade.
    private static void DrawShade(uint[] frame, int width, Shade shade)
    {
        int height = frame.Length / width;
        int left = Math.Clamp(shade.X, 0, width);
        int right = Math.Clamp(shade.X + shade.Width, 0, width);
        int top = Math.Clamp(shade.Y, 0, height);
        int bottom = Math.Clamp(shade.Y + shade.Height, 0, height);

        for (int row = top; row < bottom; row++)
        {
            for (int column = left; column < Math.Max(right, left); column++)
            {
                frame[row * width + column] = Darken(frame[row * width + column], shade.Darkness);
            }
        }
    }

    // Draws a picture: opaque pixels replace what is below, see-through ones are blended with it.
    private static void DrawImage(uint[] frame, int width, Image image)
    {
        int height = frame.Length / width;

        for (int row = 0; row < image.Height; row++)
        {
            int y = image.Y + row;

            if (y < 0 || y >= height)
            {
                continue;
            }

            for (int column = 0; column < image.Width; column++)
            {
                int x = image.X + column;

                if (x >= 0 && x < width)
                {
                    frame[y * width + x] = Blend(frame[y * width + x], image.Pixels[row * image.Width + column]);
                }
            }
        }
    }

    // top over below, by the alpha of top.
    internal static uint Blend(uint below, uint top)
    {
        uint alpha = top >> 24;

        if (alpha == 0)
        {
            return below;
        }

        if (alpha == 255)
        {
            return top;
        }

        uint Mix(int shift)
        {
            uint under = (below >> shift) & 0xff;
            uint over = (top >> shift) & 0xff;

            return (under * (255 - alpha) + over * alpha) / 255 << shift;
        }

        return 0xff00_0000 | Mix(16) | Mix(8) | Mix(0);
    }

    // A colour with each of its red, green and blue parts scaled down by darkness out of 255.
    internal static uint Darken(uint color, byte darkness)
    {
        uint keep = 255u - darkness;

        uint Part(int shift) => ((color >> shift) & 0xff) * keep / 255 << shift;

        return 0xff00_0000 | Part(16) | Part(8) | Part(0);
    }

    // Draws a line of text with the built-in font.
    private static void DrawText(uint[] frame, int width, Text text)
    {
        int height = frame.Length / width;
        int top = text.Y - GLYPH_BASELINE * text.Scale;

        for (int index = 0; index < text.Content.Length; index++)
        {
            byte[] glyph = Font8x8.BasicLegacy[text.Content[index] & 0x7f];
            int left = text.X + index * GLYPH_SIZE * text.Scale;

            for (int row = 0; row < glyph.Length; row++)
            {
                for (int column = 0; column < GLYPH_SIZE; column++)
                {
                    // Bit 0 of each row is the leftmost pixel.
                    if (((glyph[row] >> column) & 1) != 0)
                    {
                        FillBlock(frame, (width, height), (left + column * text.Scale, top + row * text.Scale), text);
                    }
                }
            }
        }
    }

    // Fills a square of text.Scale pixels at at with the text colour, where it is on the frame.
    private static void FillBlock(uint[] frame, (int Width, int Height) size, (int X, int Y) at, Text text)
    {
        for (int y = Math.Max(at.Y, 0); y < Math.Min(at.Y + text.Scale, size.Height); y++)
        {
            int start = Math.Min(Math.Max(at.X, 0), size.Width);
            int end = Math.Clamp(at.X + text.Scale, start, size.Width);

            Array.Fill(frame, text.Color, y * size.Width + start, end - start);
        }
    }
}
