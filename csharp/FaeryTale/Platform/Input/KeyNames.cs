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
        int dx = (right ? 1 : 0) - (left ? 1 : 0);
        int dy = (down ? 1 : 0) - (up ? 1 : 0);

        return (dx, dy) switch
        {
            (-1, -1) => DIR_BASE,
            (0, -1) => DIR_BASE + 1,
            (1, -1) => DIR_BASE + 2,
            (1, 0) => DIR_BASE + 3,
            (1, 1) => DIR_BASE + 4,
            (0, 1) => DIR_BASE + 5,
            (-1, 1) => DIR_BASE + 6,
            (-1, 0) => DIR_BASE + 7,
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
            ["Right"] = new(KeyKind.Arrow, 3),
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
            ["Space"] = Code(0x20),
            ["Backspace"] = Code(8),
            ["Tab"] = Code(9),
            ["Return"] = Code(13),
            ["Keypad Enter"] = Code(13),
            ["Delete"] = Code(0x7f),
            ["Escape"] = Code(CODE_ESCAPE),
            ["Keypad 0"] = Code('0'),
        };

        for (char digit = '0'; digit <= '9'; digit++)
        {
            names[digit.ToString()] = Code(digit);
        }

        for (int function = 1; function <= 10; function++)
        {
            names[$"F{function}"] = Code(FUNCTION_KEY_BASE + function - 1);
        }

        // Keypad keys give the direction code of the key's position.
        (int Digit, int Offset)[] keypad = [(7, 0), (8, 1), (9, 2), (6, 3), (3, 4), (2, 5), (1, 6), (4, 7), (5, 9)];

        foreach ((int digit, int offset) in keypad)
        {
            names[$"Keypad {digit}"] = Code(DIR_BASE + offset);
        }

        return names;
    }

    private static Key Code(int code) => new(KeyKind.Code, code);
}
