
namespace FaeryTale.Game.Input;

/// Keyboard and mouse input, replacing the original input handler. The platform layer translates
/// window events and calls the methods below.
///
/// How input reaches the game:
/// - Keys go into a buffer. The game's main loop pops them with NextKey. A key code with bit 7 set
///   means the key was released.
/// - The mouse pointer position is kept inside the text panel, where the menus and the compass
///   are. A left click on the menu column is turned into a menu key press ('a' for the first row,
///   'b' for the second column of the first row, and so on).
/// - ReadWalkDirection works out which way the hero walks, from the mouse over the compass or from
///   the direction keys, and redraws the compass.
///
/// Directions are numbered 0-7 clockwise starting at north-west: 0 NW, 1 N, 2 NE, 3 E, 4 SE, 5 S,
/// 6 SW, 7 W. The value 9 means "not walking".
public static partial class InputHandler
{
    /// The most keys the buffer holds (it was a ring of 128 bytes with one slot always empty).
    private const int KEY_BUFFER_CAPACITY = 127;
    private const int KEY_RELEASED = 0x80;

    /// The code of the Escape key. It is not one of the direction codes (20-29), the function key
    /// codes (10-19) or a letter, so it cannot be mistaken for another key (27 would be "west").
    public const int ESCAPE_KEY = 31;

    // Qualifier bits for the mouse buttons.
    private const short LEFT_BUTTON = 0x4000;
    internal const short RIGHT_BUTTON = 0x2000;

    /// The pointer starts off the play field.
    private const short POINTER_START = 320;

    // The pointer lives in the text panel, so it is kept inside these bounds.
    private const int POINTER_MIN_X = 5;
    private const int POINTER_MAX_X = 315;
    private const int POINTER_MIN_Y = 147;
    private const int POINTER_MAX_Y = 195;

    // The menu column: x range, first row's y, row height, and where the right-hand column starts.
    private const int MENU_MIN_X = 215;
    private const int MENU_MAX_X = 265;
    private const int MENU_RIGHT_COLUMN_X = 240;
    private const int MENU_TOP_Y = 144;
    private const int MENU_ROW_HEIGHT = 9;
    private const int MENU_FIRST_KEY = 0x61; // 'a'

    /// The menu has 6 rows of 2 entries; the pointer can only be below them when it is free to roam.
    private const int MENU_ROWS = 6;

    // The on-screen compass.
    private const int COMPASS_MIN_X = 265;
    private const int COMPASS_LEFT_END_X = 292;
    private const int COMPASS_CENTER_END_X = 300;
    private const int COMPASS_LOWER_Y = 166;
    private const int COMPASS_MIDDLE_END_Y = 174;

    /// Direction value meaning "not walking".
    private const short DIR_NONE = 9;

    /// Keyboard direction keys are numbered from this value (keypad digits, offset by 20).
    private const short KEYDIR_BASE = 20;
    private const short DIRECTION_COUNT = 10;

    private static readonly object padlock = new();
    private static readonly Queue<byte> keys = new();

    private static (short X, short Y) pointer = (POINTER_START, POINTER_START);
    private static (short X, short Y) rawPointer = (POINTER_START, POINTER_START);
    private static short qualifier;
    private static sbyte lastMenu;
    private static Click? pendingClick;

    /// A left click on the play field, kept for click to move: lores pixels and screen lines, or
    /// (Native) pixels of the window when the world is drawn at the window's own size.
    public readonly record struct Click(int X, int Y, bool Native);

    /// Where the pointer is in the window, in pixels, while it is over the world drawn at the
    /// window's own size (null otherwise).
    public static (int X, int Y)? NativePointer { get; private set; }

    // While the pointer is over the world, the game's own pointer position is put somewhere on the
    // play field (clear of the panel's menus and compass).
    private const int NATIVE_PLACEHOLDER_X = 160;
    private const int NATIVE_PLACEHOLDER_Y = 60;

    private static (int X, int Y)? gridPointer;

    /// Where the pointer is on the 640 x 400 grid the menus are drawn on. With the old picture that
    /// is the game pointer doubled; with the native size picture it is worked out from the window
    /// pixel, whether the pointer is over the world or the panel.
    public static (int X, int Y) GridPointer
    {
        get
        {
            lock (padlock)
            {
                return gridPointer ?? (rawPointer.X * 2, rawPointer.Y * 2);
            }
        }
    }

    /// Key event from the platform layer. code is the game's key code, bit 7 set = key released.
    public static void KeyEvent(int code)
    {
        lock (padlock)
        {
            PushKey(code);
        }
    }

    /// Forgets everything a finished game left behind: keys, buttons and the pointer.
    public static void Reset()
    {
        lock (padlock)
        {
            keys.Clear();
            pointer = (POINTER_START, POINTER_START);
            rawPointer = (POINTER_START, POINTER_START);
            qualifier = 0;
            lastMenu = 0;
            ShiftHeld = false;
            pendingClick = null;
            NativePointer = null;
            gridPointer = null;
            RemasteredControls = false;
            JoystickFire = false;
        }
    }

    /// Empties the key buffer (the hero is being revived, so old key presses are dropped).
    public static void ResetKeyBuffer()
    {
        lock (padlock)
        {
            keys.Clear();
        }
    }

    /// Forgets the last menu key sent, so its release is not sent (used when a menu is switched).
    public static void ForgetLastMenu()
    {
        lock (padlock)
        {
            lastMenu = 0;
        }
    }

    // Appends a key to the buffer; the key is dropped if the buffer is full.
    private static void PushKey(int code)
    {
        if (keys.Count < KEY_BUFFER_CAPACITY)
        {
            keys.Enqueue((byte)code);
        }
    }
}
