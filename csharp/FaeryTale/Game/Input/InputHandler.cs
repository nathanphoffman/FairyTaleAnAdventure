using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

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
public static class InputHandler
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

    /// The pointer moved over the world, to a pixel of the window.
    public static void NativeMouseMoved(int pixelX, int pixelY)
    {
        lock (padlock)
        {
            NativePointer = (pixelX, pixelY);
            rawPointer = (NATIVE_PLACEHOLDER_X, NATIVE_PLACEHOLDER_Y);
            pointer = (NATIVE_PLACEHOLDER_X, NATIVE_PLACEHOLDER_Y);
        }
    }

    /// The pointer is not over the world any more (the next move is a normal one on the panel).
    public static void NativeMouseLeftWorld()
    {
        lock (padlock)
        {
            NativePointer = null;
        }
    }

    /// The play field ends and the text panel starts at this screen line.
    private const int PLAY_FIELD_BOTTOM = 143;

    /// Remastered and Reimagined: the pointer goes over the whole screen, a left click on the play
    /// field is kept for click to move, and the keys are the modern set (WASD walks). Original
    /// keeps the pointer in the text panel and the original keys.
    public static bool RemasteredControls { get; set; }

    /// Whether the joystick fire button is held.
    public static bool JoystickFire { get; set; }

    /// The mouse buttons held (bit 0x4000 left, 0x2000 right).
    public static short MouseButtons
    {
        get
        {
            lock (padlock)
            {
                return qualifier;
            }
        }
    }

    /// Whether the left mouse button is held.
    public static bool LeftButtonDown => (MouseButtons & LEFT_BUTTON) != 0;

    /// Where the pointer is on the whole screen, in lores pixels and screen lines.
    public static (int X, int Y) RawPointer
    {
        get
        {
            lock (padlock)
            {
                return (rawPointer.X, rawPointer.Y);
            }
        }
    }

    /// Where the pointer is, kept inside the text panel.
    public static (int X, int Y) Pointer
    {
        get
        {
            lock (padlock)
            {
                return (pointer.X, pointer.Y);
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
            pendingClick = null;
            NativePointer = null;
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

    /// Pops the oldest key code from the buffer, or 0 if there is none.
    public static int NextKey()
    {
        lock (padlock)
        {
            return keys.Count > 0 ? keys.Dequeue() : 0;
        }
    }

    /// Pointer position in lores pixels (x 0-319) and screen lines (y 0-199). The pointer is kept
    /// inside the text panel (the bottom of the screen), where the menus and the compass are,
    /// because that is the only place the mouse is used.
    public static void MouseMoved(int x, int y)
    {
        lock (padlock)
        {
            rawPointer = ((short)x, (short)y);
            pointer = ((short)Math.Clamp(x, POINTER_MIN_X, POINTER_MAX_X), (short)Math.Clamp(y, POINTER_MIN_Y, POINTER_MAX_Y));
        }
    }

    /// Mouse button change. A left click in the menu column becomes a menu key on press and the
    /// same key with bit 7 set on release, as if it had been typed. The right button only sets a
    /// flag (it makes the hero fight, see ReadWalkDirection).
    public static void MouseButtonsChanged(bool left, bool right)
    {
        short newQualifier = (short)((left ? LEFT_BUTTON : 0) | (right ? RIGHT_BUTTON : 0));

        lock (padlock)
        {
            // Xor with the old state leaves only the bits that changed: the left button was pressed or released.
            if (((newQualifier ^ qualifier) & LEFT_BUTTON) != 0)
            {
                int? key = null;

                if ((newQualifier & LEFT_BUTTON) != 0)
                {
                    (int x, int y) = RemasteredControls ? rawPointer : pointer;

                    key = MenuKeyAt(x, y);
                    RememberPlayFieldClick(x, y);
                }
                else if (lastMenu != 0)
                {
                    key = (byte)lastMenu | KEY_RELEASED;
                    lastMenu = 0;
                }

                if (key is { } pressed)
                {
                    PushKey(pressed);

                    // As in the original: the last key sent is remembered, release included.
                    lastMenu = unchecked((sbyte)pressed);
                }
            }

            qualifier = newQualifier;
        }
    }

    /// Decides the walking direction from the mouse over the compass or from the keyboard
    /// direction keys, and redraws the compass when it changes.
    ///
    /// While a mouse button is down the pointer position decides; otherwise the direction key
    /// (numbered from 20, the keypad digits) does. Anything outside 20-29 means no direction.
    public static void ReadWalkDirection(Data.Ui ui, IOutput output)
    {
        short direction = UserDirection(ui);

        // With no key or compass click, a click to move walk lights the compass instead.
        if (direction == DIR_NONE)
        {
            direction = ui.PathDirection;
        }

        ui.CompassLitDirection = direction;

        if (direction != ui.CompassDrawnDirection)
        {
            ui.CompassDrawnDirection = direction;
            output.DrawCompass(direction);
        }
    }

    /// The direction the player asks for by hand (0-7, 9 = none): the pointer over the compass
    /// while a button is down, else the direction keys.
    public static short UserDirection(Data.Ui ui)
    {
        short buttons;
        (int X, int Y) place;

        lock (padlock)
        {
            buttons = qualifier;
            place = RemasteredControls ? (rawPointer.X, rawPointer.Y) : (pointer.X, pointer.Y);
        }

        if ((buttons & (LEFT_BUTTON | RIGHT_BUTTON)) != 0)
        {
            return CompassDirection(place.X, place.Y);
        }

        short direction = unchecked((short)(ui.HeldDirection - KEYDIR_BASE));

        if (direction < 0 || direction >= DIRECTION_COUNT)
        {
            ui.HeldDirection = 0;
            direction = DIR_NONE;
        }

        return direction;
    }

    /// Walking direction (0-7, 9 = none) for a pointer position over the on-screen compass.
    ///
    /// The compass is split into three columns and three rows; the middle square means no
    /// direction. Left of the compass nothing is selected.
    internal static short CompassDirection(int x, int y)
    {
        // Above the text panel there is no compass (the pointer can only be there when it is free to roam).
        if (x <= COMPASS_MIN_X || y < POINTER_MIN_Y)
        {
            return DIR_NONE;
        }

        (short up, short center, short down) = x < COMPASS_LEFT_END_X ? ((short)0, (short)7, (short)6)
            : x <= COMPASS_CENTER_END_X ? ((short)1, DIR_NONE, (short)5)
            : ((short)2, (short)3, (short)4);

        if (y < COMPASS_LOWER_Y)
        {
            return up;
        }

        return y <= COMPASS_MIDDLE_END_Y ? center : down;
    }

    /// The menu key under the pointer, if any: 'a' plus twice the row number, plus one for the
    /// right-hand column. The menu is two columns of rows 9 lines high, starting at line 144.
    internal static int? MenuKeyAt(int x, int y)
    {
        if (x < MENU_MIN_X || x > MENU_MAX_X || y < MENU_TOP_Y)
        {
            return null;
        }

        int row = (y - MENU_TOP_Y) / MENU_ROW_HEIGHT;
        int right = x >= MENU_RIGHT_COLUMN_X ? 1 : 0;

        if (row >= MENU_ROWS)
        {
            return null;
        }

        return row * 2 + MENU_FIRST_KEY + right;
    }

    /// The play field position (lores pixels) of a left click not yet used by click to move.
    public static Click? TakeClick()
    {
        lock (padlock)
        {
            Click? click = pendingClick;

            pendingClick = null;

            return click;
        }
    }

    // Keeps a left click on the play field for click to move (called with the lock held).
    private static void RememberPlayFieldClick(int x, int y)
    {
        if (!RemasteredControls)
        {
            return;
        }

        if (NativePointer is { } native)
        {
            pendingClick = new Click(native.X, native.Y, true);
        }
        else if (y < PLAY_FIELD_BOTTOM)
        {
            pendingClick = new Click(x, y, false);
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
