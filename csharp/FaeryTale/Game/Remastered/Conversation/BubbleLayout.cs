namespace FaeryTale.Game.Remastered;

/// Where the parts of the conversation bubble are: a face over the person's head and a board of
/// three choices to the right of it. The drawing and the click test both ask this one class, so a
/// button is where it is drawn. Sizes are in "units": each is Scale screen pixels.
public readonly record struct BubbleLayout(int Left, int Top, int Scale)
{
    public const int FACE_SIZE = 40;
    public const int BUTTON_WIDTH = 44;
    public const int BUTTON_HEIGHT = 14;
    public const int BUTTON_GAP = 3;
    public const int BUTTON_COUNT = 3;

    /// Space between the face and the buttons, and between the bubble and the person's head.
    private const int SIDE_GAP = 6;
    private const int HEAD_GAP = 8;

    /// The person's head, in world pixels above the actor's position.
    private const int HEAD_HEIGHT = 30;

    /// One unit is a whole number of pixels, one for each 400 pixels of window height (rounded),
    /// so the bubble is about as big on any screen.
    private const double PIXELS_PER_UNIT_HEIGHT = 400.0;

    /// The full width and height of the bubble in units.
    public static int Width => FACE_SIZE + SIDE_GAP + BUTTON_WIDTH;

    /// Where the bubble goes for a person standing at a world position, kept inside the window.
    public static BubbleLayout For(WorldView view, (int X, int Y) person)
    {
        int scale = Math.Max(1, (int)Math.Round((double)view.Height / PIXELS_PER_UNIT_HEIGHT));
        (double headX, double headY) = view.ToScreen(person.X, person.Y - HEAD_HEIGHT);
        int left = (int)headX - FACE_SIZE * scale / 2;
        int top = (int)headY - (FACE_SIZE + HEAD_GAP) * scale;

        return new BubbleLayout(
            Math.Clamp(left, 0, Math.Max(0, view.Width - Width * scale)),
            Math.Clamp(top, 0, Math.Max(0, view.Height - FACE_SIZE * scale)),
            scale);
    }

    /// The top of button number index (0 is the first), in units from the top of the bubble.
    public static int ButtonTop(int index) => index * (BUTTON_HEIGHT + BUTTON_GAP);

    /// The left of the buttons, in units from the left of the bubble.
    public static int ButtonLeft => FACE_SIZE + SIDE_GAP;

    /// Which button a window pixel is on, if any.
    public int? ButtonAt(int pixelX, int pixelY)
    {
        int x = FloorDivide(pixelX - Left, Scale) - ButtonLeft;
        int y = FloorDivide(pixelY - Top, Scale);

        for (int index = 0; index < BUTTON_COUNT; index++)
        {
            if (x >= 0 && x < BUTTON_WIDTH && y >= ButtonTop(index) && y < ButtonTop(index) + BUTTON_HEIGHT)
            {
                return index;
            }
        }

        return null;
    }

    private static int FloorDivide(int value, int by) => (int)Math.Floor((double)value / by);
}
