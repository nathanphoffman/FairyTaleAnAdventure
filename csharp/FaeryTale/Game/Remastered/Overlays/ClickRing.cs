using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The circle that shows where the player clicked: green when the hero can stand there, red when
/// not. Drawn as a picture that is made once.
internal static class ClickRing
{
    /// The play field ends at this screen line; the ring is not drawn over the text panel.
    private const int PLAY_FIELD_BOTTOM = 143;

    /// The overlay grid is twice as wide and twice as tall as the game's screen.
    private const int OVERLAY_SCALE = 2;

    private const int OUTER_RADIUS = 14;
    private const int RING_WIDTH = 4;
    private const float EDGE_SOFTNESS = 1.2f;
    private const uint GREEN = 0x3cdc50;
    private const uint RED = 0xe83c3c;
    private const uint OUTLINE = 0x101010;

    /// Alpha (0-255) of the ring, the see-through middle, and the dark line round the ring.
    private const float RING_ALPHA = 235f;
    private const float FILL_ALPHA = 55f;
    private const float OUTLINE_ALPHA = 150f;

    /// The ring's size in overlay pixels is for the old picture; a bigger or smaller one is drawn by
    /// giving how many screen pixels one overlay pixel is.
    private const float OLD_PICTURE_UNIT = 1f;

    private static readonly uint[] greenPixels = Draw(GREEN, OLD_PICTURE_UNIT);
    private static readonly uint[] redPixels = Draw(RED, OLD_PICTURE_UNIT);

    // Rings already drawn for the zoom steps, so one is made once per size.
    private static readonly Dictionary<(float Unit, bool Walkable), Image> sized = new();

    /// The ring for the zoomed world, centred on a screen pixel: it grows and shrinks with the
    /// world (an overlay pixel is half a game pixel, which is half of scale screen pixels).
    public static Image PictureAtPixel((int X, int Y) center, bool walkable, int scale)
    {
        float unit = scale / (float)OVERLAY_SCALE;

        if (!sized.TryGetValue((unit, walkable), out Image? ring))
        {
            int radius = (int)MathF.Ceiling(OUTER_RADIUS * unit);

            ring = new Image(0, 0, radius * 2 + 1, radius * 2 + 1, Draw(walkable ? GREEN : RED, unit));
            sized[(unit, walkable)] = ring;
        }

        return ring with { X = center.X - ring.Width / 2, Y = center.Y - ring.Height / 2 };
    }

    /// The ring centred on an overlay position, with the rows that would fall on the text panel cut off.
    public static Image PictureAt((int X, int Y) center, bool walkable)
    {
        int size = OUTER_RADIUS * 2 + 1;
        int visibleRows = Math.Clamp(PLAY_FIELD_BOTTOM * OVERLAY_SCALE - (center.Y - OUTER_RADIUS), 0, size);

        return new Image(center.X - OUTER_RADIUS, center.Y - OUTER_RADIUS, size, visibleRows, walkable ? greenPixels : redPixels);
    }

    // The ring as pixels: a coloured band with a dark line outside it and a faint fill inside,
    // with soft edges. Each pixel's strength comes from its distance to the centre.
    private static uint[] Draw(uint color, float unit)
    {
        int radius = (int)MathF.Ceiling(OUTER_RADIUS * unit);
        int size = radius * 2 + 1;
        uint[] pixels = new uint[size * size];

        for (int row = 0; row < size; row++)
        {
            for (int column = 0; column < size; column++)
            {
                float distance = MathF.Sqrt((column - radius) * (column - radius) + (row - radius) * (row - radius));

                pixels[row * size + column] = PixelAt(distance, color, unit);
            }
        }

        return pixels;
    }

    // One pixel of the ring, given its distance from the centre.
    private static uint PixelAt(float distance, uint color, float unit)
    {
        float outer = OUTER_RADIUS * unit;
        float inner = (OUTER_RADIUS - RING_WIDTH) * unit;
        float ring = Coverage(outer - unit - distance) * Coverage(distance - inner);
        float line = Coverage(outer - distance) * Coverage(distance - inner + unit) * (1 - ring);
        float fill = Coverage(inner - distance);

        float alpha = Math.Max(Math.Max(ring * RING_ALPHA, line * OUTLINE_ALPHA), fill * FILL_ALPHA);

        if (alpha <= 0f)
        {
            return 0;
        }

        uint rgb = ring >= line ? color : OUTLINE;

        return (uint)alpha << 24 | rgb;
    }

    // 0 to 1: how much of a pixel is inside an edge, given its signed distance (soft edge).
    private static float Coverage(float signedDistance) => Math.Clamp(0.5f + signedDistance / EDGE_SOFTNESS, 0f, 1f);
}
