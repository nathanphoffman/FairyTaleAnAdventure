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

    private static readonly uint[] greenPixels = Draw(GREEN);
    private static readonly uint[] redPixels = Draw(RED);

    /// The ring centred on an overlay position, with the rows that would fall on the text panel cut off.
    public static Image PictureAt((int X, int Y) center, bool walkable)
    {
        int size = OUTER_RADIUS * 2 + 1;
        int visibleRows = Math.Clamp(PLAY_FIELD_BOTTOM * OVERLAY_SCALE - (center.Y - OUTER_RADIUS), 0, size);

        return new Image(center.X - OUTER_RADIUS, center.Y - OUTER_RADIUS, size, visibleRows, walkable ? greenPixels : redPixels);
    }

    // The ring as pixels: a coloured band with a dark line outside it and a faint fill inside,
    // with soft edges. Each pixel's strength comes from its distance to the centre.
    private static uint[] Draw(uint color)
    {
        int size = OUTER_RADIUS * 2 + 1;
        uint[] pixels = new uint[size * size];

        for (int row = 0; row < size; row++)
        {
            for (int column = 0; column < size; column++)
            {
                float distance = MathF.Sqrt((column - OUTER_RADIUS) * (column - OUTER_RADIUS) + (row - OUTER_RADIUS) * (row - OUTER_RADIUS));

                pixels[row * size + column] = PixelAt(distance, color);
            }
        }

        return pixels;
    }

    // One pixel of the ring, given its distance from the centre.
    private static uint PixelAt(float distance, uint color)
    {
        float inner = OUTER_RADIUS - RING_WIDTH;
        float ring = Coverage(OUTER_RADIUS - 1 - distance) * Coverage(distance - inner);
        float line = Coverage(OUTER_RADIUS - distance) * Coverage(distance - inner + 1) * (1 - ring);
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
