using FaeryTale.Game.Input;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Platform;

/// Turns keys and the mouse (from the window or from a test script) into what the game's input
/// handler takes: key codes, the keypad direction keys made from the arrows, the fire button, and
/// the mouse position and buttons.
public static class InputBridge
{
    /// Added to a key code to say the key was released.
    private const int RELEASED = 0x80;

    /// Lores pixels are doubled horizontally in the 640 wide frame.
    private const int FRAME_TO_LORES = 2;

    // Where the pointer can go, in lores pixels / screen lines (the game's play field and panel).
    private const int LORES_WIDTH = 320;
    private const int SCREEN_LINES = 200;
    private const int POINTER_MIN_X = 5;
    private const int POINTER_MAX_X = 315;
    private const int POINTER_MIN_Y = 147;
    private const int POINTER_MAX_Y = 195;

    private static readonly bool[] arrows = new bool[4];
    private static int heldDirection;
    private const string MAP_KEY = "M";
    private const string INVENTORY_KEY = "I";
    private const string PAGE_UP = "PageUp";
    private const string PAGE_DOWN = "PageDown";

    /// A page is one line less than the window shows, so the eye can follow.
    private const int PAGE_LINES = MessageHistory.VISIBLE_LINES - 1;

    private const string LEFT_SHIFT = "Left Shift";
    private const string RIGHT_SHIFT = "Right Shift";

    private static bool leftShiftDown;
    private static bool rightShiftDown;
    private static bool leftDown;
    private static bool rightDown;

    /// Pointer position for drawing the cursor, in lores pixels and screen lines.
    public static (int X, int Y) Pointer { get; private set; } = (160, 170);

    /// A key was pressed or released: passes it on to the game as an arrow, the fire button or a
    /// key code. A name SDL does not know does nothing.
    public static void KeyEvent(string name, bool down)
    {
        TrackShift(name, down);
        TypeCheatLetter(name, down);
        ScrollMessagesWithKey(name, down);

        if (InputHandler.RemasteredControls && name == MAP_KEY)
        {
            AskForMap(down);

            return;
        }

        if (InputHandler.RemasteredControls && name == INVENTORY_KEY)
        {
            OpenOrCloseInventory(down);

            return;
        }

        KeyNames.Key key = KeyNames.Lookup(InputHandler.RemasteredControls ? RemasteredKeys.Translate(name) : name);

        switch (key.Kind)
        {
            case KeyNames.KeyKind.Arrow:
                SetArrow(key.Value, down);
                break;

            case KeyNames.KeyKind.Fire:
                InputHandler.JoystickFire = down;
                break;

            case KeyNames.KeyKind.Code:
                InputHandler.KeyEvent(key.Value | (down ? 0 : RELEASED));
                break;
        }
    }

    // I opens the inventory and closes it again; the game does not see the key.
    private static void OpenOrCloseInventory(bool down)
    {
        if (down)
        {
            Inventory.Toggle();
        }
    }

    // M opens the big map (and closes it again); the game does not see the key.
    private static void AskForMap(bool down)
    {
        if (down)
        {
            MapScreen.Request();
        }
    }

    // The letters typed are watched for the cheat word (Remastered and Reimagined).
    private static void TypeCheatLetter(string name, bool down)
    {
        if (down && InputHandler.RemasteredControls && name.Length == 1 && char.IsAsciiLetter(name[0]))
        {
            CheatCode.Type(name[0]);
        }
    }

    // Page Up and Page Down scroll the message window back and forward (Remastered and Reimagined).
    private static void ScrollMessagesWithKey(string name, bool down)
    {
        if (!down || !InputHandler.RemasteredControls)
        {
            return;
        }

        if (name == PAGE_UP)
        {
            MessageScrollRequests.Add(PAGE_LINES);
        }
        else if (name == PAGE_DOWN)
        {
            MessageScrollRequests.Add(-PAGE_LINES);
        }
    }

    // Remembers whether either shift key is down (SDL's names for them).
    private static void TrackShift(string name, bool down)
    {
        if (name == LEFT_SHIFT)
        {
            leftShiftDown = down;
        }
        else if (name == RIGHT_SHIFT)
        {
            rightShiftDown = down;
        }

        InputHandler.ShiftHeld = leftShiftDown || rightShiftDown;
    }

    /// The mouse moved to a position in the 640x200 frame.
    public static void MouseMove(int frameX, int frameY)
    {
        int x = frameX / FRAME_TO_LORES;

        InputHandler.MouseMoved(x, frameY);
        Pointer = InputHandler.RemasteredControls
            ? (Math.Clamp(x, 0, LORES_WIDTH - 1), Math.Clamp(frameY, 0, SCREEN_LINES - 1))
            : (Math.Clamp(x, POINTER_MIN_X, POINTER_MAX_X), Math.Clamp(frameY, POINTER_MIN_Y, POINTER_MAX_Y));
    }

    /// A mouse button was pressed or released.
    public static void MouseButton(bool left, bool down)
    {
        if (left)
        {
            leftDown = down;
        }
        else
        {
            rightDown = down;
        }

        InputHandler.MouseButtonsChanged(leftDown, rightDown);
    }

    // Sets one direction input (0 up, 1 down, 2 left, 3 right) and sends the matching keypad
    // direction key to the game: arrows are turned into the keypad direction keys, so two arrows
    // give a diagonal.
    private static void SetArrow(int index, bool down)
    {
        arrows[index] = down;

        int next = KeyNames.DirectionCode(arrows[0], arrows[1], arrows[2], arrows[3]);
        int old = heldDirection;

        heldDirection = next;

        if (next == old)
        {
            return;
        }

        if (old != 0)
        {
            InputHandler.KeyEvent(old | RELEASED);
        }

        if (next != 0)
        {
            InputHandler.KeyEvent(next);
        }
    }
}
