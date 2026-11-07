using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Game.Ui;

/// Text and story routines that were 68000 assembly in the original. The story strings are in
/// StoryText.
///
/// Message lists are zero-terminated strings stored one after another; picking message n means
/// skipping n zero bytes.
public static class TextRoutines
{
    /// In a string: followed by x/2 and y, moves the text position.
    private const byte XY = 128;

    // A positioning code in a string is three bytes long: the marker, then x / 2 and y.
    private const int XY_BYTES = 3;
    private const int XY_X_SCALE = 2;

    private const uint DECIMAL_BASE = 10;

    /// Digits in the number buffer; one trailing space follows them.
    private const int NUMBER_DIGITS = 10;

    /// Size of the print queue; the indexes wrap with this mask.
    private const short PRINT_QUEUE_MASK = 31;

    // Menu states: an item the player has, and one they lack.
    private const int MENU_STATE_HAVE = 10;
    private const int MENU_STATE_LACKING = 8;

    private const int ERROR_SCREEN_X = 200;
    private const int ERROR_SCREEN_Y = 90;

    /// Error number meaning graphics are not open, so nothing can be printed.
    private const int ERROR_NO_GRAPHICS = 2;

    /// Right-aligned decimal in 10 characters, leading positions blank, and a trailing space
    /// (which the number printing adds after the number).
    public static byte[] FormatNumber(uint value)
    {
        byte[] text = new byte[NUMBER_DIGITS + 1];
        Array.Fill(text, (byte)' ');
        int index = NUMBER_DIGITS;

        while (true)
        {
            index--;
            text[index] = (byte)('0' + value % DECIMAL_BASE);
            value /= DECIMAL_BASE;

            if (value == 0 || index == 0)
            {
                break;
            }
        }

        return text;
    }

    /// Moves the text position (the C name is "move").
    public static void MoveTo(IOutput output, int x, int y) => output.SetTextPosition(x, y);

    /// Prints a string that may hold XY escapes (128, x/2, y) to move the text position. A string
    /// ends at a zero byte (or its end); bytes of 128 and above end each plain run of text.
    /// (The x position is stored halved so it fits in a byte.)
    public static void PrintPositioned(IOutput output, byte[] text)
    {
        int at = 0;

        while (ByteAt(text, at) != 0)
        {
            if (ByteAt(text, at) == XY)
            {
                MoveTo(output, ByteAt(text, at + 1) * XY_X_SCALE, ByteAt(text, at + 2));
                at += XY_BYTES;

                continue;
            }

            int run = 0;

            while (ByteAt(text, at + run) != 0 && ByteAt(text, at + run) < XY)
            {
                run++;
            }

            output.PrintContBytes(text.AsSpan(at, run));
            at += run;

            if (run == 0)
            {
                at++; // stray byte >= 128 that is not XY
            }
        }
    }

    /// Prints story page text number n (the strings use position escapes, see PrintPositioned).
    public static void PlacardText(IOutput output, int n) => PrintPositioned(output, StoryText.PLACARD_MSGS[n]);

    /// The n-th zero-terminated string of a list of messages (the first is number 0). A number
    /// past the end gives the final zero byte, an empty string.
    public static ReadOnlySpan<byte> NthString(byte[] list, int n)
    {
        int at = 0;

        for (int skipped = 0; skipped < n; skipped++)
        {
            int length = list.AsSpan(at).IndexOf((byte)0);

            at = Math.Min(at + Math.Max(length, 0) + 1, list.Length - 1);
        }

        return list.AsSpan(at);
    }

    /// Shows the num-th string of a list of messages, word-wrapped.
    public static void ShowMessage(Data.Game game, IOutput output, byte[] list, int num) =>
        Messages.PrintWrapped(game, output, NthString(list, num));

    /// Shows event message n (things that happen to the hero, such as hunger).
    public static void ShowEvent(Data.Game game, IOutput output, int n) => ShowMessage(game, output, StoryText.EVENT_MSG, n);

    /// Shows speech number n (what people say).
    public static void Speak(Data.Game game, IOutput output, int n)
    {
        if (game.Experience != Experience.Original)
        {
            Journal.Record(game, NthString(StoryText.SPEECHES, n));
        }

        ShowMessage(game, output, StoryText.SPEECHES, n);
    }

    /// Adds a request to the print queue (ignored if the queue is full). The screen code handles
    /// them one per tick, which spreads the redrawing work over several frames.
    public static void QueuePrint(Data.Ui ui, int request)
    {
        short next = (short)((ui.PrintQueue.Write + 1) & PRINT_QUEUE_MASK);

        if (next == ui.PrintQueue.Read)
        {
            return;
        }

        ui.PrintQueue.Items[ui.PrintQueue.Write] = (sbyte)request;
        ui.PrintQueue.Write = next;
    }

    /// Menu state for an item: 10 if the player has item n, otherwise 8.
    public static int ItemMenuState(Hero hero, int n) => hero.Stuff[n] != 0 ? MENU_STATE_HAVE : MENU_STATE_LACKING;

    /// Shows "ERROR n" on the text panel (startup failures). Error 2 (no graphics) cannot be shown.
    public static void ShowError(IOutput output, int n)
    {
        if (n == ERROR_NO_GRAPHICS)
        {
            return;
        }

        MoveTo(output, ERROR_SCREEN_X, ERROR_SCREEN_Y);
        output.PrintContBytes("ERROR "u8);
        output.PrintNumber(n, 2);
    }

    private static byte ByteAt(byte[] bytes, int index) => index < bytes.Length ? bytes[index] : (byte)0;
}
