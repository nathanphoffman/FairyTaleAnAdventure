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

        for (int positionY = 0; positionY < SIZE; positionY++)
        {
            for (int positionX = 0; positionX < SIZE; positionX++)
            {
                pixels[positionY * SIZE + positionX] = PixelAt(palette, sprite, positionX, positionY);
            }
        }

        return new Image(0, 0, SIZE, SIZE, pixels);
    }

    /// The same face with the tail of a speech bubble hanging under it.
    public static Image WithTail(Image face)
    {
        uint[] pixels = new uint[SIZE * (SIZE + TAIL_HEIGHT)];

        Array.Copy(face.Pixels, pixels, face.Pixels.Length);

        for (int rowIndex = 0; rowIndex < TAIL_HEIGHT; rowIndex++)
        {
            int half = (TAIL_WIDTH - rowIndex * TAIL_WIDTH / TAIL_HEIGHT) / 2;

            for (int positionX = SIZE / 2 - half; positionX <= SIZE / 2 + half; positionX++)
            {
                bool edge = positionX == SIZE / 2 - half || positionX == SIZE / 2 + half || rowIndex == TAIL_HEIGHT - 1;

                pixels[(SIZE + rowIndex) * SIZE + positionX] = edge ? OUTER_FRAME : INNER_FRAME;
            }
        }

        return new Image(0, 0, SIZE, SIZE + TAIL_HEIGHT, pixels);
    }

    // One pixel: outside the rounded corners nothing, then the two frame lines, then the head.
    private static uint PixelAt(uint[] palette, SpriteImage sprite, int positionX, int positionY)
    {
        int depth = Math.Min(Math.Min(positionX, SIZE - 1 - positionX), Math.Min(positionY, SIZE - 1 - positionY));

        if (CutOffByCorner(positionX, positionY))
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

        int column = CROP_LEFT + (positionX - FRAME) / ZOOM;
        int rowIndex = CROP_TOP + (positionY - FRAME) / ZOOM;
        bool inside = column < sprite.Width && rowIndex < sprite.Height && sprite.Opaque[rowIndex * sprite.Width + column];

        return inside ? palette[sprite.Pixels[rowIndex * sprite.Width + column]] : BACKGROUND;
    }

    // Whether a pixel lies outside the circle that rounds off one of the four corners.
    private static bool CutOffByCorner(int positionX, int positionY)
    {
        int deltaX = positionX < CORNER ? CORNER - positionX : positionX >= SIZE - CORNER ? positionX - (SIZE - 1 - CORNER) : 0;  // distance into a corner zone: left edge, right edge, else 0
        int deltaY = positionY < CORNER ? CORNER - positionY : positionY >= SIZE - CORNER ? positionY - (SIZE - 1 - CORNER) : 0;  // distance into a corner zone: top edge, bottom edge, else 0

        return deltaX * deltaX + deltaY * deltaY > CORNER * CORNER;
    }
}
