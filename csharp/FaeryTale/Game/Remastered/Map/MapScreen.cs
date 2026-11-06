using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The big map (Remastered and Reimagined), opened with M over the game, which waits meanwhile. It
/// shows the whole outdoor world with a mark where the hero is. Drag it with the mouse or use the
/// arrow keys to scroll, the wheel to zoom, C to come back to the hero, M or Esc to close.
public static class MapScreen
{
    private const int FRAME_LEFT = (Overlay.WIDTH - MapView.WIDTH) / 2;
    private const int FRAME_TOP = 28;
    private const int TEXT_SCALE = 1;

    // The wooden board the map lies on, and how far it reaches round the map.
    private const int BORDER = 6;
    private const uint BOARD_SEED = 0x3a9d;
    private const int TITLE_BASELINE = 20;
    private const int HINT_BASELINE = 392;
    private const int BACKGROUND_DARKNESS = 170;
    private const string TITLE = "The Land";
    private const string HINT = "Drag or arrow keys scroll    Wheel zooms    C finds you    M or Esc closes";

    // Keys come with bit 7 set when released; the direction keys are the codes from 20 (up to 27).
    private const int KEY_RELEASED = 0x80;
    private const int KEY_CODE_MASK = 0x7f;
    private const int DIRECTION_KEY_BASE = 20;
    private const int DIRECTION_COUNT = 8;
    private const int CENTER_KEY = 'C';

    /// Units the view moves each frame an arrow key is held.
    private const int KEY_PAN_UNITS = 6;

    // The mark blinks: shown for this many frames, then hidden for as many.
    private const int BLINK_FRAMES = 25;

    // The eight directions of the keys, clockwise from up-left (as the compass numbers them).
    private static readonly (int X, int Y)[] DIRECTIONS = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];

    private static Image? board;
    private static readonly object padlock = new();
    private static bool requested;
    private static int zoomSteps;

    /// Whether the big map is open (the wheel then zooms it).
    public static bool IsOpen { get; private set; }

    /// Asks for the map to open (or, when it is open, to close). Called when M is pressed.
    public static void Request()
    {
        lock (padlock)
        {
            requested = true;
        }
    }

    /// Asks to zoom the open map in (positive) or out (negative) by wheel steps.
    public static void ZoomBy(int steps)
    {
        lock (padlock)
        {
            zoomSteps += steps;
        }
    }

    /// Whether M was pressed since the last call.
    public static bool TakeRequest()
    {
        lock (padlock)
        {
            bool asked = requested;

            requested = false;

            return asked;
        }
    }

    /// Shows the map until the player closes it. Opens only where there is an outdoor world to show.
    public static void Show(Data.Game game, Video video)
    {
        MapView view = new();
        (int X, int Y)? dragFrom = null;
        int held = -1;
        int frame = 0;
        (double, double, double, bool) drawn = default;

        view.CenterOn(HeroTile(game));
        ClickToMove.CancelAll();
        IsOpen = true;

        while (true)
        {
            int key = InputHandler.NextKey();

            if (key == InputHandler.ESCAPE_KEY || TakeRequest())
            {
                break;
            }

            held = HeldDirection(key, held);

            if (key == CENTER_KEY)
            {
                view.CenterOn(HeroTile(game));
            }

            dragFrom = Pan(view, held, dragFrom);
            view.ZoomBySteps(TakeZoom());
            drawn = Redraw(video, view, HeroTile(game), frame++ / BLINK_FRAMES % 2 == 0, drawn);
            Display.Delay(game, video, 1);
        }

        Leave(game, video);
    }

    // The tile the hero is on.
    private static (int Column, int Row) HeroTile(Data.Game game) =>
        WorldOverview.TileOf(game.Actors.List[0].AbsX, game.Actors.List[0].AbsY);

    // The direction key held after a key (0-7), or -1 for none. The key codes are the game's.
    private static int HeldDirection(int key, int held)
    {
        int direction = (key & KEY_CODE_MASK) - DIRECTION_KEY_BASE;

        if (key == 0 || direction < 0 || direction >= DIRECTION_COUNT)
        {
            return held;
        }

        return (key & KEY_RELEASED) == 0 ? direction : held == direction ? -1 : held;
    }

    // Moves the view with a held arrow key or a dragging mouse. Returns where the mouse was.
    private static (int X, int Y)? Pan(MapView view, int held, (int X, int Y)? dragFrom)
    {
        if (held >= 0)
        {
            view.PanBy(DIRECTIONS[held].X * KEY_PAN_UNITS, DIRECTIONS[held].Y * KEY_PAN_UNITS);
        }

        if (!InputHandler.LeftButtonDown)
        {
            return null;
        }

        (int x, int y) = InputHandler.GridPointer;

        if (dragFrom is { } from)
        {
            view.PanBy(from.X - x, from.Y - y);
        }

        return (x, y);
    }

    private static int TakeZoom()
    {
        lock (padlock)
        {
            int steps = zoomSteps;

            zoomSteps = 0;

            return steps;
        }
    }

    // Puts the map on the screen again, but only when the view or the blinking mark changed (making
    // the picture takes a while, and the screen keeps the last one). Returns what was drawn.
    private static (double, double, double, bool) Redraw(Video video, MapView view, (int Column, int Row) hero, bool markShown, (double, double, double, bool) drawn)
    {
        (double, double, double, bool) now = (view.CenterColumn, view.CenterRow, view.Scale, markShown);

        if (now != drawn)
        {
            video.SetOverlay(Draw(view, hero, markShown));
        }

        return now;
    }

    // The darkened game with the map in a frame, a title and a line of help.
    private static Overlay Draw(MapView view, (int Column, int Row) hero, bool markShown)
    {
        Overlay overlay = new() { Shades = { new Shade(0, 0, Overlay.WIDTH, Overlay.HEIGHT, BACKGROUND_DARKNESS) }, PointerOnTop = true, CenterOnScreen = true };
        Image land = view.Picture(WorldOverview.Pixels, hero, markShown);

        overlay.Images.Add(Board());
        overlay.Images.Add(land with { X = FRAME_LEFT, Y = FRAME_TOP });
        MenuStyle.AddCentered(overlay, TITLE, TITLE_BASELINE, MenuStyle.HEADING_COLOR, TEXT_SCALE * 2);
        MenuStyle.AddCentered(overlay, HINT, HINT_BASELINE, MenuStyle.DESCRIPTION_COLOR, TEXT_SCALE);

        return overlay;
    }

    // The wooden board the map lies on, made once (it takes a moment).
    private static Image Board() =>
        board ??= MenuStyle.Board((FRAME_LEFT - BORDER, FRAME_TOP - BORDER, MapView.WIDTH + 2 * BORDER, MapView.HEIGHT + 2 * BORDER), BOARD_SEED)[0];

    // Takes the map off the screen, and keeps the click or key that closed it out of the game.
    private static void Leave(Data.Game game, Video video)
    {
        IsOpen = false;

        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }

        video.SetOverlay(new Overlay());
        InputHandler.TakeClick();
        InputHandler.ResetKeyBuffer();
        InputHandler.ForgetLastMenu();
    }
}
