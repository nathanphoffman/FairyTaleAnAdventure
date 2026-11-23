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

    // Colours are 0xAARRGGBB: the alpha byte sits 24 bits up, red 16, green 8.
    private const int ALPHA_SHIFT = 24;
    private const int RED_SHIFT = 16;
    private const int GREEN_SHIFT = 8;
    private const uint OPAQUE_ALPHA = 0xff00_0000;

    // The largest value of one colour channel, and the mask that picks a channel out.
    private const uint CHANNEL_MAX = 255;
    private const uint CHANNEL_MASK = 0xff;

    // Characters above 127 are not in the built-in font, so glyph lookups keep only 7 bits.
    private const int GLYPH_CODE_MASK = 0x7f;

    public List<Shade> Shades { get; init; } = new();

    public List<Image> Images { get; init; } = new();

    public List<Text> Texts { get; init; } = new();

    /// The game's mouse pointer is drawn over the overlay instead of under it (menus that cover the game).
    public bool PointerOnTop { get; init; }

    /// With the world shown the overlay's grid is centred on the window instead of sitting at the
    /// bottom of it (above the text panel): for menus that cover the whole game.
    public bool CenterOnScreen { get; init; }

    /// The game's mouse pointer is not drawn at all (something else, such as a door icon, takes its place).
    public bool HidePointer { get; init; }

    /// Where the overlay's grid is on a frame: the pixel of the frame its top left corner is at, and
    /// how many frame pixels each grid pixel is (the grid is 640 x 400).
    public readonly record struct Placement(int OriginX, int OriginY, int Scale)
    {
        public static readonly Placement Identity = new(0, 0, 1);
    }

    /// The height of the overlay grid.
    public const int HEIGHT = 400;

    /// Draws the overlay on a frame width pixels wide (and so many rows tall).
    public void Draw(uint[] frame, int width) => DrawAt(frame, width, Placement.Identity);

    /// Draws the overlay with its grid placed and scaled on a frame (a larger frame than the grid:
    /// the part of the frame outside the grid is only touched by a shade that covers the whole grid,
    /// which darkens all of the frame).
    public void DrawAt(uint[] frame, int width, Placement placement)
    {
        if (width == 0)
        {
            return;
        }

        foreach (Shade shade in Shades)
        {
            DrawShade(frame, width, shade, placement);
        }

        foreach (Image image in Images)
        {
            DrawImage(frame, width, image, placement);
        }

        foreach (Text text in Texts)
        {
            DrawText(frame, width, text, placement);
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
    private static void DrawShade(uint[] frame, int width, Shade shade, Placement placement)
    {
        int height = frame.Length / width;
        bool coversGrid = shade.X <= 0 && shade.Y <= 0 && shade.X + shade.Width >= WIDTH && shade.Y + shade.Height >= HEIGHT;
        int left = coversGrid ? 0 : Math.Clamp(placement.OriginX + shade.X * placement.Scale, 0, width);
        int right = coversGrid ? width : Math.Clamp(placement.OriginX + (shade.X + shade.Width) * placement.Scale, 0, width);
        int topEdge = coversGrid ? 0 : Math.Clamp(placement.OriginY + shade.Y * placement.Scale, 0, height);
        int bottom = coversGrid ? height : Math.Clamp(placement.OriginY + (shade.Y + shade.Height) * placement.Scale, 0, height);

        // The rows are independent, so they are darkened on all the cores (the dark overlay of the
        // menus and the map covers the whole window, many pixels every frame).
        Parallel.For(topEdge, Math.Max(bottom, topEdge), row =>
        {
            for (int column = left; column < Math.Max(right, left); column++)
            {
                frame[row * width + column] = Darken(frame[row * width + column], shade.Darkness);
            }
        });
    }

    // Draws a picture: opaque pixels replace what is below, see-through ones are blended with it.
    // Each pixel of the picture is a square of Scale pixels on the frame.
    private static void DrawImage(uint[] frame, int width, Image image, Placement placement)
    {
        int height = frame.Length / width;
        int scale = placement.Scale;

        for (int rowIndex = 0; rowIndex < image.Height; rowIndex++)
        {
            int rowTop = placement.OriginY + (image.Y + rowIndex) * scale;

            for (int column = 0; column < image.Width; column++)
            {
                uint color = image.Pixels[rowIndex * image.Width + column];
                int columnLeft = placement.OriginX + (image.X + column) * scale;

                if (color >> ALPHA_SHIFT == 0)  // alpha is the top byte; 0 means fully transparent
                {
                    continue;
                }

                // The square of frame pixels this picture pixel covers, cut to the frame.
                int firstY = Math.Max(rowTop, 0);
                int lastY = Math.Min(rowTop + scale, height);
                int firstX = Math.Max(columnLeft, 0);
                int lastX = Math.Min(columnLeft + scale, width);

                for (int positionY = firstY; positionY < lastY; positionY++)
                {
                    for (int positionX = firstX; positionX < lastX; positionX++)
                    {
                        frame[positionY * width + positionX] = Blend(frame[positionY * width + positionX], color);
                    }
                }
            }
        }
    }

    // top over below, by the alpha of top.
    internal static uint Blend(uint below, uint topEdge)
    {
        uint alpha = topEdge >> ALPHA_SHIFT;  // alpha lives in the top byte (bits 24-31)

        if (alpha == 0)
        {
            return below;
        }

        if (alpha == CHANNEL_MAX)
        {
            return topEdge;
        }

        uint Mix(int shift)
        {
            uint under = (below >> shift) & CHANNEL_MASK;  // below channel: shift down, mask to 0-255
            uint over = (topEdge >> shift) & CHANNEL_MASK;  // top channel: shift down, mask to 0-255

            return (under * (CHANNEL_MAX - alpha) + over * alpha) / CHANNEL_MAX << shift;  // weighted average of the two channels, back in place
        }

        return OPAQUE_ALPHA | Mix(RED_SHIFT) | Mix(GREEN_SHIFT) | Mix(0);
    }

    // A colour with each of its red, green and blue parts scaled down by darkness out of 255.
    internal static uint Darken(uint color, byte darkness)
    {
        uint keep = CHANNEL_MAX - darkness;

        uint Part(int shift) => ((color >> shift) & CHANNEL_MASK) * keep / CHANNEL_MAX << shift;  // scale one channel toward black, put it back in place

        return OPAQUE_ALPHA | Part(RED_SHIFT) | Part(GREEN_SHIFT) | Part(0);
    }

    // Draws a line of text with the built-in font.
    private static void DrawText(uint[] frame, int width, Text text, Placement placement)
    {
        int height = frame.Length / width;
        int block = text.Scale * placement.Scale;
        int topEdge = placement.OriginY + (text.Y - GLYPH_BASELINE * text.Scale) * placement.Scale;

        for (int index = 0; index < text.Content.Length; index++)
        {
            byte[] glyph = Font8x8.BasicLegacy[text.Content[index] & GLYPH_CODE_MASK];
            int left = placement.OriginX + (text.X + index * GLYPH_SIZE * text.Scale) * placement.Scale;

            for (int rowIndex = 0; rowIndex < glyph.Length; rowIndex++)
            {
                for (int column = 0; column < GLYPH_SIZE; column++)
                {
                    // Bit 0 of each row is the leftmost pixel.
                    if (((glyph[rowIndex] >> column) & 1) != 0)
                    {
                        FillBlock(frame, (width, height), (left + column * block, topEdge + rowIndex * block), (block, text.Color));
                    }
                }
            }
        }
    }

    // Fills a square of the given size at at with a colour, where it is on the frame.
    private static void FillBlock(uint[] frame, (int Width, int Height) frameSize, (int X, int Y) offset, (int Size, uint Color) block)
    {
        for (int positionY = Math.Max(offset.Y, 0); positionY < Math.Min(offset.Y + block.Size, frameSize.Height); positionY++)
        {
            int start = Math.Min(Math.Max(offset.X, 0), frameSize.Width);
            int end = Math.Clamp(offset.X + block.Size, start, frameSize.Width);

            Array.Fill(frame, block.Color, positionY * frameSize.Width + start, end - start);
        }
    }
}
