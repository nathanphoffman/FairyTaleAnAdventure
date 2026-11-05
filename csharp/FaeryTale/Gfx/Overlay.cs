namespace FaeryTale.Gfx;

/// A rectangle that is darkened: Darkness 0 leaves the picture alone, 255 makes it black.
public sealed record Shade(int X, int Y, int Width, int Height, byte Darkness);

/// A ready-made picture of 0xAARRGGBB pixels with its top left corner at (X, Y).
public sealed record Image(int X, int Y, int Width, int Height, uint[] Pixels);

/// A line of text with its left end and baseline at (X, Y), an 0xAARRGGBB colour and a scale:
/// each pixel of the 8x8 font is Scale x Scale overlay pixels.
public sealed record Text(string Content, int X, int Y, uint Color, int Scale);

/// Text and dark panels drawn over the finished frame, on a 640 x 400 grid. Drawing it comes with
/// the graphics phase; for now it only holds what the rules asked to show.
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
}
