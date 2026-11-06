using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The mouse pointer over something that can be picked up or opened: a pointing hand. It is drawn
/// from a small picture of letters (one per pixel), the finger tip is where the pointer is.
internal static class HandCursor
{
    private const int WIDTH = 16;

    // The finger tip, in pixels from the top left of the picture.
    private const int TIP_X = 5;
    private const int TIP_Y = 1;

    private const uint OUTLINE = 0xff20_1006;
    private const uint SKIN = 0xffff_d8a8;
    private const uint SHADE = 0xffd8_a070;

    // '.' is see-through, 'o' the outline, 'h' skin and 's' a darker skin for the thumb and knuckles.
    private static readonly string[] ROWS =
    [
        ".....oo",
        "....ohho",
        "....ohho",
        "....ohho",
        "....ohhooo",
        "....ohhohhooo",
        "....ohhohhohhoo",
        ".oo.ohhhhhhhhhho",
        "ohhoohhhhhhhhhho",
        "ohshohhhhhhhhhho",
        ".ohhhhhhhhhhhhho",
        "..ohhhhhhhhhhhho",
        "..ohhhhhhhhhhho",
        "...ohhhhhhhhhho",
        "...ohhhhhhhhho",
        "....ohhhhhhhho",
        "....ohhhhhhhho",
        ".....oooooooo",
    ];

    private static readonly uint[] pixels = Draw();

    /// The icon placed so that the finger tip is at an overlay position.
    public static Image At((int X, int Y) tip) => new(tip.X - TIP_X, tip.Y - TIP_Y, WIDTH, ROWS.Length, pixels);

    // Every pixel of the icon from its letters (a short row is see-through to the right).
    private static uint[] Draw()
    {
        uint[] result = new uint[WIDTH * ROWS.Length];

        for (int y = 0; y < ROWS.Length; y++)
        {
            for (int x = 0; x < ROWS[y].Length; x++)
            {
                result[y * WIDTH + x] = ROWS[y][x] switch { 'o' => OUTLINE, 'h' => SKIN, 's' => SHADE, _ => 0u };
            }
        }

        return result;
    }
}
