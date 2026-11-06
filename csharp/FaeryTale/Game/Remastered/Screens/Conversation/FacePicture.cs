using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// A person's face, cut from the top of their sprite and blown up, in a rounded frame. It is the
/// pointer over a person and the picture in the conversation bubble.
public static class FacePicture
{
    // The head is in the top middle of a person's 16 x 32 sprite.
    private const int CROP_LEFT = 2;
    private const int CROP_TOP = 0;
    private const int CROP_SIZE = 12;

    private const int FRAME = 2;
    private const int CORNER = 6;

    /// Units across and down of a face picture: the head, blown up, inside its frame.
    public const int SIZE = BubbleLayout.FACE_SIZE;
    private const int ZOOM = (SIZE - 2 * FRAME) / CROP_SIZE;

    /// How far the tail of the speech bubble reaches below the face, and how wide it is at the top.
    public const int TAIL_HEIGHT = 6;
    private const int TAIL_WIDTH = 9;

    private const uint BACKGROUND = 0xff3a_4a5a;
    private const uint OUTER_FRAME = 0xff20_1006;
    private const uint INNER_FRAME = 0xffd8_b070;

    /// The face of a sprite on a square of SIZE units, with the corners cut round.
    public static Image Face(uint[] palette, SpriteImage sprite)
    {
        uint[] pixels = new uint[SIZE * SIZE];

        for (int y = 0; y < SIZE; y++)
        {
            for (int x = 0; x < SIZE; x++)
            {
                pixels[y * SIZE + x] = PixelAt(palette, sprite, x, y);
            }
        }

        return new Image(0, 0, SIZE, SIZE, pixels);
    }

    /// The same face with the tail of a speech bubble hanging under it.
    public static Image WithTail(Image face)
    {
        uint[] pixels = new uint[SIZE * (SIZE + TAIL_HEIGHT)];

        Array.Copy(face.Pixels, pixels, face.Pixels.Length);

        for (int row = 0; row < TAIL_HEIGHT; row++)
        {
            int half = (TAIL_WIDTH - row * TAIL_WIDTH / TAIL_HEIGHT) / 2;

            for (int x = SIZE / 2 - half; x <= SIZE / 2 + half; x++)
            {
                bool edge = x == SIZE / 2 - half || x == SIZE / 2 + half || row == TAIL_HEIGHT - 1;

                pixels[(SIZE + row) * SIZE + x] = edge ? OUTER_FRAME : INNER_FRAME;
            }
        }

        return new Image(0, 0, SIZE, SIZE + TAIL_HEIGHT, pixels);
    }

    // One pixel: outside the rounded corners nothing, then the two frame lines, then the head.
    private static uint PixelAt(uint[] palette, SpriteImage sprite, int x, int y)
    {
        int depth = Math.Min(Math.Min(x, SIZE - 1 - x), Math.Min(y, SIZE - 1 - y));

        if (CutOffByCorner(x, y))
        {
            return 0;
        }

        if (depth == 0)
        {
            return OUTER_FRAME;
        }

        if (depth < FRAME)
        {
            return INNER_FRAME;
        }

        int column = CROP_LEFT + (x - FRAME) / ZOOM;
        int row = CROP_TOP + (y - FRAME) / ZOOM;
        bool inside = column < sprite.Width && row < sprite.Height && sprite.Opaque[row * sprite.Width + column];

        return inside ? palette[sprite.Pixels[row * sprite.Width + column]] : BACKGROUND;
    }

    // Whether a pixel lies outside the circle that rounds off one of the four corners.
    private static bool CutOffByCorner(int x, int y)
    {
        int dx = x < CORNER ? CORNER - x : x >= SIZE - CORNER ? x - (SIZE - 1 - CORNER) : 0;
        int dy = y < CORNER ? CORNER - y : y >= SIZE - CORNER ? y - (SIZE - 1 - CORNER) : 0;

        return dx * dx + dy * dy > CORNER * CORNER;
    }
}
