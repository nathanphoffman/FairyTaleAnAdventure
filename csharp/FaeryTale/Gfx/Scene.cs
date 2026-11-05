namespace FaeryTale.Gfx;

/// A rectangle of a bitmap shown on screen with its own colours.
public sealed class ViewPort
{
    public required Bitmap Bitmap { get; init; }

    /// The part of the bitmap shown starts here.
    public (int X, int Y) Scroll { get; init; }

    /// Where on screen it goes (in its own pixels) and how big it is.
    public (int X, int Y) Position { get; init; }

    public (int Width, int Height) Size { get; init; }

    public bool Hires { get; init; }

    /// The colour table: 12 bit colours (0x0RGB).
    public required ushort[] Colors { get; init; }

    /// The mouse pointer is drawn with the colours of this view port.
    public bool ShowsPointer { get; init; }
}

/// The mouse pointer: a 16 pixel wide sprite. Each line is two 16 bit planes packed in one word;
/// the first word is a control word.
public sealed record Pointer(uint[] Image, (int X, int Y) Position);

/// Everything shown: the view ports from the top one down, and the pointer. Turned into the
/// pixels of a frame by Compose.
public sealed class Scene
{
    /// Size of the frame in pixels. The play field is lores (each of its pixels is shown twice as
    /// wide); the text panel is hires.
    public const int FRAME_WIDTH = 640;
    public const int FRAME_HEIGHT = 200;

    private const int LORES_SCALE = 2;
    private const int BITS_PER_BYTE = 8;
    private const int NIBBLE_MASK = 15;
    private const int NIBBLE_TO_BYTE = 17; // 0xF * 17 = 0xFF
    private const int GREEN_NIBBLE_SHIFT = 4;
    private const int RED_NIBBLE_SHIFT = 8;
    private const int RED_BYTE_SHIFT = 16;
    private const int GREEN_BYTE_SHIFT = 8;

    // The pointer's colours are 16 + the sprite pixel value in the colour table.
    private const int POINTER_FIRST_COLOR = 16;
    private const int POINTER_WIDTH = 16;
    private const int POINTER_MAX_LINES = 32;
    private const int HIGH_WORD_SHIFT = 16;

    public List<ViewPort> ViewPorts { get; init; } = [];

    public Pointer? Pointer { get; init; }

    /// 12 bit colour (4 bits per channel, 0x0RGB) to 0xRRGGBB.
    public static uint Rgb4(ushort color)
    {
        // Pulls out each 4 bit component: shift it down, then mask off the rest.
        uint red = (uint)((color >> RED_NIBBLE_SHIFT) & NIBBLE_MASK);
        uint green = (uint)((color >> GREEN_NIBBLE_SHIFT) & NIBBLE_MASK);
        uint blue = (uint)(color & NIBBLE_MASK);

        // Puts the three components, each stretched from 4 to 8 bits, side by side in one pixel.
        return ((red * NIBBLE_TO_BYTE) << RED_BYTE_SHIFT) | ((green * NIBBLE_TO_BYTE) << GREEN_BYTE_SHIFT) | (blue * NIBBLE_TO_BYTE);
    }

    /// Draws the whole scene into the frame (FRAME_WIDTH x FRAME_HEIGHT pixels).
    public void Compose(uint[] frame)
    {
        Array.Fill(frame, 0u);

        foreach (ViewPort viewPort in ViewPorts)
        {
            DrawViewPort(viewPort, frame);
        }

        ViewPort? pointerColors = ViewPorts.FirstOrDefault(viewPort => viewPort.ShowsPointer);

        if (Pointer is not null && pointerColors is not null)
        {
            DrawPointer(frame, pointerColors.Colors, Pointer);
        }
    }

    // Sets a frame pixel if it lies inside the frame.
    internal static void PutPixel(uint[] frame, int x, int y, uint color)
    {
        if (x >= 0 && x < FRAME_WIDTH && y >= 0 && y < FRAME_HEIGHT)
        {
            frame[y * FRAME_WIDTH + x] = color;
        }
    }

    // Draws one view port's rows into the frame.
    private static void DrawViewPort(ViewPort viewPort, uint[] frame)
    {
        Bitmap bitmap = viewPort.Bitmap;

        for (int row = 0; row < viewPort.Size.Height; row++)
        {
            int sourceY = viewPort.Scroll.Y + row;
            int outY = viewPort.Position.Y + row;

            if (sourceY < 0 || sourceY >= bitmap.Rows || outY < 0 || outY >= FRAME_HEIGHT)
            {
                continue;
            }

            for (int column = 0; column < viewPort.Size.Width; column++)
            {
                int sourceX = viewPort.Scroll.X + column;

                if (sourceX < 0 || sourceX >= bitmap.BytesPerRow * BITS_PER_BYTE)
                {
                    continue;
                }

                int index = bitmap.Pixel(sourceX, sourceY);
                uint color = index < viewPort.Colors.Length ? Rgb4(viewPort.Colors[index]) : 0;
                int screenX = viewPort.Position.X + column;

                if (viewPort.Hires)
                {
                    PutPixel(frame, screenX, outY, color);
                }
                else
                {
                    PutPixel(frame, screenX * LORES_SCALE, outY, color);
                    PutPixel(frame, screenX * LORES_SCALE + 1, outY, color);
                }
            }
        }
    }

    // Draws the mouse pointer: a 16 pixel wide sprite, 2 frame pixels per sprite pixel.
    private static void DrawPointer(uint[] frame, ushort[] colors, Pointer pointer)
    {
        for (int line = 0; line < POINTER_MAX_LINES; line++)
        {
            uint word = 1 + line < pointer.Image.Length ? pointer.Image[1 + line] : 0;

            if (word == 0)
            {
                break;
            }

            DrawPointerLine(frame, colors, word, (pointer.Position.X * LORES_SCALE, pointer.Position.Y + line));
        }
    }

    // Draws one line of the pointer sprite. A 32 bit value holds two 16 pixel rows of a bitplane.
    private static void DrawPointerLine(uint[] frame, ushort[] colors, uint word, (int X, int Y) origin)
    {
        ushort a = (ushort)(word >> HIGH_WORD_SHIFT);
        ushort b = (ushort)word;

        for (int bit = 0; bit < POINTER_WIDTH; bit++)
        {
            int shift = POINTER_WIDTH - 1 - bit;

            // One pixel of the two interleaved planes: plane a is the low bit, plane b the high bit.
            int value = ((a >> shift) & 1) | (((b >> shift) & 1) << 1);

            if (value == 0 || POINTER_FIRST_COLOR + value >= colors.Length)
            {
                continue;
            }

            for (int dx = 0; dx < LORES_SCALE; dx++)
            {
                PutPixel(frame, origin.X + bit * LORES_SCALE + dx, origin.Y, Rgb4(colors[POINTER_FIRST_COLOR + value]));
            }
        }
    }
}
