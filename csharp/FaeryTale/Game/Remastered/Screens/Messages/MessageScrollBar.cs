using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The scroll bar on the right edge of the message window (the beige scroll), which says there are
/// older lines to scroll back to and how far back the window is. Drawn on the 640 x 400 overlay grid.
public static class MessageScrollBar
{
    // The track: a thin bar beside the four lines of messages (the lines start at row 288, the message box being raised 4 lines).
    private const int LEFT = 406;
    private const int TOP = 288;
    private const int WIDTH = 7;
    private const int HEIGHT = 78;

    // The thumb is never smaller than this, whatever the history length.
    private const int MIN_THUMB_HEIGHT = 10;

    // The bar can be grabbed a little to either side of it (it is thin).
    private const int GRAB_MARGIN = 4;

    private static bool buttonWasDown;
    private static bool dragging;

    private const uint OUTLINE = 0xff5a_3a1a;
    private const uint TRACK = 0xffc8_9a60;
    private const uint THUMB = 0xff8a_4a20;
    private const uint THUMB_EDGE = 0xffe8_c080;

    /// The bar for a history, or nothing when every line fits in the window.
    public static Overlay Of(MessageHistory history)
    {
        Overlay overlay = new();

        if (history.MaxScroll > 0)
        {
            overlay.Images.Add(Bar(history));
        }

        return overlay;
    }

    /// How far back the window should be scrolled while the player drags the bar (null when not
    /// dragging). A drag starts when the button goes down on the bar, and goes on, wherever the
    /// pointer moves, until the button is let go. pointer is a place on the 640 x 400 grid.
    public static int? DragTarget(MessageHistory history, (int X, int Y) pointer, bool buttonDown)
    {
        bool pressed = buttonDown && !buttonWasDown;

        buttonWasDown = buttonDown;

        if (pressed)
        {
            dragging = history.MaxScroll > 0 && OnBar(pointer);
        }

        if (!buttonDown)
        {
            dragging = false;
        }

        return dragging ? ScrollAt(history, pointer.Y) : null;
    }

    // Whether a place on the grid is on the bar (or just beside it).
    private static bool OnBar((int X, int Y) place) =>
        place.X >= LEFT - GRAB_MARGIN && place.X < LEFT + WIDTH + GRAB_MARGIN && place.Y >= TOP && place.Y < TOP + HEIGHT;

    // The scroll that puts the middle of the thumb at a row of the grid: the top of the track is the
    // oldest line and the bottom the newest.
    private static int ScrollAt(MessageHistory history, int row)
    {
        int thumbHeight = ThumbHeight(history);
        double fromTop = Math.Clamp((row - TOP - thumbHeight / 2.0) / (HEIGHT - thumbHeight), 0, 1);

        return (int)Math.Round((1 - fromTop) * history.MaxScroll);
    }

    private static int ThumbHeight(MessageHistory history) => Math.Max(MIN_THUMB_HEIGHT, HEIGHT * MessageHistory.VISIBLE_LINES / history.LineCount);

    // The track with the thumb on it: the thumb is at the bottom when the newest lines are shown.
    private static Image Bar(MessageHistory history)
    {
        int thumbHeight = ThumbHeight(history);
        int thumbTop = (HEIGHT - thumbHeight) * (history.MaxScroll - history.Scrolled) / history.MaxScroll;
        uint[] pixels = new uint[WIDTH * HEIGHT];

        for (int y = 0; y < HEIGHT; y++)
        {
            for (int x = 0; x < WIDTH; x++)
            {
                pixels[y * WIDTH + x] = PixelAt(x, y, (thumbTop, thumbHeight));
            }
        }

        return new Image(LEFT, TOP, WIDTH, HEIGHT, pixels);
    }

    // One pixel: the outline round the track, the thumb (with a light edge), or the bare track.
    private static uint PixelAt(int x, int y, (int Top, int Height) thumb)
    {
        if (x == 0 || y == 0 || x == WIDTH - 1 || y == HEIGHT - 1)
        {
            return OUTLINE;
        }

        if (y < thumb.Top || y >= thumb.Top + thumb.Height)
        {
            return TRACK;
        }

        bool edge = y == thumb.Top || y == thumb.Top + thumb.Height - 1 || x == 1 || x == WIDTH - 2;

        return edge ? THUMB_EDGE : THUMB;
    }
}
