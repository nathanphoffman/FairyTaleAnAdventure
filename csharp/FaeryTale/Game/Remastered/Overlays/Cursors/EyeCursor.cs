using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The mouse pointer while shift is held: an eye, to say a click will walk there and look around.
/// It is drawn once from a few rules (an almond shape, a round iris and pupil), like the door icon.
internal static class EyeCursor
{
    private const int WIDTH = 30;
    private const int HEIGHT = 18;

    private const int IRIS_RADIUS = 6;
    private const int PUPIL_RADIUS = 3;
    private const int GLINT_OFFSET = 2;

    private const uint OUTLINE = 0xff20_1006;
    private const uint WHITE = 0xfff4_f0e4;
    private const uint IRIS = 0xff3a_8ac8;
    private const uint PUPIL = 0xff10_1820;
    private const uint GLINT = 0xffff_ffff;

    private static readonly uint[] pixels = Draw();

    /// The icon with its middle at an overlay position.
    public static Image At((int X, int Y) center) => new(center.X - WIDTH / 2, center.Y - HEIGHT / 2, WIDTH, HEIGHT, pixels);

    // Every pixel of the icon.
    private static uint[] Draw()
    {
        uint[] result = new uint[WIDTH * HEIGHT];

        for (int positionY = 0; positionY < HEIGHT; positionY++)
        {
            for (int positionX = 0; positionX < WIDTH; positionX++)
            {
                result[positionY * WIDTH + positionX] = PixelAt(positionX, positionY);
            }
        }

        return result;
    }

    // Whether a pixel is inside the almond: the lids are a curve (a parabola) that meets at the corners.
    private static bool Inside(int positionX, int positionY)
    {
        double across = (positionX - (WIDTH - 1) / 2.0) / (WIDTH / 2.0);
        double up = Math.Abs(positionY - (HEIGHT - 1) / 2.0);
        double lid = HEIGHT / 2.0 * (1 - across * across);

        return positionX >= 0 && positionX < WIDTH && positionY >= 0 && positionY < HEIGHT && up <= lid && Math.Abs(across) < 1;
    }

    private static uint PixelAt(int positionX, int positionY)
    {
        if (!Inside(positionX, positionY))
        {
            return 0;
        }

        if (!Inside(positionX - 1, positionY) || !Inside(positionX + 1, positionY) || !Inside(positionX, positionY - 1) || !Inside(positionX, positionY + 1))
        {
            return OUTLINE;
        }

        return EyeballPixel(positionX - (WIDTH - 1) / 2.0, positionY - (HEIGHT - 1) / 2.0);
    }

    // The inside of the eye at a distance from its middle: pupil, iris or white, with a glint.
    private static uint EyeballPixel(double deltaX, double deltaY)
    {
        double distance = Math.Sqrt(deltaX * deltaX + deltaY * deltaY);

        if (Math.Abs(deltaX + GLINT_OFFSET) < 1 && Math.Abs(deltaY + GLINT_OFFSET) < 1)
        {
            return GLINT;
        }

        return distance <= PUPIL_RADIUS ? PUPIL : distance <= IRIS_RADIUS ? IRIS : WHITE;  // nested ternary: innermost ring wins, else white
    }
}
