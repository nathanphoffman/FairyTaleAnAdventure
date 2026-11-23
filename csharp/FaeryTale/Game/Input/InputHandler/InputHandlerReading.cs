using FaeryTale.Game.Output;

namespace FaeryTale.Game.Input;

/// InputHandler: reading keys and the walk direction.
public static partial class InputHandler
{
    /// Pops the oldest key code from the buffer, or 0 if there is none.
    public static int NextKey()
    {
        lock (padlock)
        {
            return keys.Count > 0 ? keys.Dequeue() : 0;
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
}
