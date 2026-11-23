
namespace FaeryTale.Game.Input;

/// InputHandler: the mouse and clicks.
public static partial class InputHandler
{
    /// The pointer moved, to a place of the menu grid (native size drawing).
    public static void NativeGridMoved(int gridX, int gridY)
    {
        lock (padlock)
        {
            gridPointer = (gridX, gridY);
        }
    }

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

    /// Whether a shift key is held (Remastered: shift and click to look).
    public static bool ShiftHeld { get; set; }

    /// Pointer position in lores pixels (x 0-319) and screen lines (y 0-199). The pointer is kept
    /// inside the text panel (the bottom of the screen), where the menus and the compass are,
    /// because that is the only place the mouse is used.
    public static void MouseMoved(int x, int y)
    {
        lock (padlock)
        {
            gridPointer = null;
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
}
