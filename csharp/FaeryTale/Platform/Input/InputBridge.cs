using FaeryTale.Game.Input;

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
    private const int POINTER_MIN_X = 5;
    private const int POINTER_MAX_X = 315;
    private const int POINTER_MIN_Y = 147;
    private const int POINTER_MAX_Y = 195;

    private static readonly bool[] arrows = new bool[4];
    private static int heldDirection;
    private static bool leftDown;
    private static bool rightDown;

    /// Pointer position for drawing the cursor, in lores pixels and screen lines.
    public static (int X, int Y) Pointer { get; private set; } = (160, 170);

    /// A key was pressed or released: passes it on to the game as an arrow, the fire button or a
    /// key code. A name SDL does not know does nothing.
    public static void KeyEvent(string name, bool down)
    {
        KeyNames.Key key = KeyNames.Lookup(name);

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

    /// The mouse moved to a position in the 640x200 frame.
    public static void MouseMove(int frameX, int frameY)
    {
        int x = frameX / FRAME_TO_LORES;

        InputHandler.MouseMoved(x, frameY);
        Pointer = (Math.Clamp(x, POINTER_MIN_X, POINTER_MAX_X), Math.Clamp(frameY, POINTER_MIN_Y, POINTER_MAX_Y));
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
