using FaeryTale.Game.Input;

namespace FaeryTale.Platform;

/// Translating key names into the game's key codes. The names are the ones SDL gives its keys
/// (the test scripts and the real window use the same ones), so a name SDL does not know, such as
/// "LCtrl" (SDL says "Left Ctrl"), does nothing, exactly as it did in the Rust game.
///
/// Key codes are upper-case letters, digits and symbols as ASCII, F1-F10 = 10-19, keypad digits =
/// direction codes 20-29, bit 7 = released.
public static class KeyNames
{
    /// What a key does in the game.
    public enum KeyKind
    {
        /// The game has no use for the key.
        None,

        /// An arrow key: Value is 0 up, 1 down, 2 left, 3 right.
        Arrow,

        /// A control key: the joystick fire button.
        Fire,

        /// A key with a game key code in Value.
        Code,
    }

    public readonly record struct Key(KeyKind Kind, int Value);

    /// Direction codes are the compass numbers 0-9 offset by this.
    private const int DIR_BASE = 20;

    /// Offsets from DIR_BASE for each compass direction (and the middle of the keypad).
    private const int NORTH_EAST_OFFSET = 2;
    private const int EAST_OFFSET = 3;
    private const int SOUTH_EAST_OFFSET = 4;
    private const int SOUTH_OFFSET = 5;
    private const int SOUTH_WEST_OFFSET = 6;
    private const int WEST_OFFSET = 7;
    private const int CENTER_OFFSET = 9;

    /// The value of the right arrow in a Key of kind Arrow.
    private const int RIGHT_ARROW = 3;

    /// Plain ASCII codes for keys that have no character to write.
    private const int SPACE_CODE = 0x20;
    private const int BACKSPACE_CODE = 8;
    private const int TAB_CODE = 9;
    private const int RETURN_CODE = 13;
    private const int DELETE_CODE = 0x7f;

    /// How many function keys there are (F1 to F10).
    private const int FUNCTION_KEY_COUNT = 10;

    /// F1 is code 10, F2 is 11 ... F10 is 19.
    private const int FUNCTION_KEY_BASE = 10;

    /// Escape: skips the intro, leaves a game, or opens the game menu.
    public const int CODE_ESCAPE = InputHandler.ESCAPE_KEY;

    private static readonly Dictionary<string, Key> NAMES = Build();

    /// What the key with this SDL name does (None if there is no such key or it is unused).
    public static Key Lookup(string name)
    {
        if (NAMES.TryGetValue(name, out Key key))
        {
            return key;
        }

        // A single letter is the same key in either case.
        return name.Length == 1 && char.IsAsciiLetter(name[0]) ? new Key(KeyKind.Code, char.ToUpperInvariant(name[0])) : default;
    }

    /// Direction codes: the game's compass numbering, offset by 20.
    ///   20 21 22      keypad 7 8 9
    ///   27 29 23      keypad 4 5 6
    ///   26 25 24      keypad 1 2 3
    public static int DirectionCode(bool up, bool down, bool left, bool right)
    {
        int deltaX = (right ? 1 : 0) - (left ? 1 : 0);  // true counts as 1, so right minus left gives -1, 0 or 1
        int deltaY = (down ? 1 : 0) - (up ? 1 : 0);  // true counts as 1, so down minus up gives -1, 0 or 1

        return (deltaX, deltaY) switch
        {
            (-1, -1) => DIR_BASE,
            (0, -1) => DIR_BASE + 1,
            (1, -1) => DIR_BASE + NORTH_EAST_OFFSET,
            (1, 0) => DIR_BASE + EAST_OFFSET,
            (1, 1) => DIR_BASE + SOUTH_EAST_OFFSET,
            (0, 1) => DIR_BASE + SOUTH_OFFSET,
            (-1, 1) => DIR_BASE + SOUTH_WEST_OFFSET,
            (-1, 0) => DIR_BASE + WEST_OFFSET,
            _ => 0,
        };
    }

    private static Dictionary<string, Key> Build()
    {
        Dictionary<string, Key> names = new(StringComparer.OrdinalIgnoreCase)
        {
            ["Up"] = new(KeyKind.Arrow, 0),
            ["Down"] = new(KeyKind.Arrow, 1),
            ["Left"] = new(KeyKind.Arrow, 2),
            ["Right"] = new(KeyKind.Arrow, RIGHT_ARROW),
            ["Left Ctrl"] = new(KeyKind.Fire, 0),
            ["Right Ctrl"] = new(KeyKind.Fire, 0),
            ["-"] = Code('-'),
            ["Keypad -"] = Code('-'),
            ["="] = Code('='),
            ["`"] = Code('`'),
            ["\\"] = Code('\\'),
            ["["] = Code('{'),
            ["]"] = Code('}'),
            [";"] = Code(':'),
            ["'"] = Code('?'),
            ["/"] = Code('?'),
            [","] = Code(','),
            ["."] = Code('.'),
            ["Keypad ."] = Code('.'),
            ["Space"] = Code(SPACE_CODE),
            ["Backspace"] = Code(BACKSPACE_CODE),
            ["Tab"] = Code(TAB_CODE),
            ["Return"] = Code(RETURN_CODE),
            ["Keypad Enter"] = Code(RETURN_CODE),
            ["Delete"] = Code(DELETE_CODE),
            ["Escape"] = Code(CODE_ESCAPE),
            ["Keypad 0"] = Code('0'),
        };

        for (char digit = '0'; digit <= '9'; digit++)
        {
            names[digit.ToString()] = Code(digit);
        }

        for (int function = 1; function <= FUNCTION_KEY_COUNT; function++)
        {
            names[$"F{function}"] = Code(FUNCTION_KEY_BASE + function - 1);
        }

        // Keypad keys give the direction code of the key's position.
        (char Digit, int Offset)[] keypad = [('7', 0), ('8', 1), ('9', NORTH_EAST_OFFSET), ('6', EAST_OFFSET), ('3', SOUTH_EAST_OFFSET), ('2', SOUTH_OFFSET), ('1', SOUTH_WEST_OFFSET), ('4', WEST_OFFSET), ('5', CENTER_OFFSET)];

        foreach ((char digit, int offset) in keypad)
        {
            names[$"Keypad {digit}"] = Code(DIR_BASE + offset);
        }

        return names;
    }

    private static Key Code(int code) => new(KeyKind.Code, code);
}
