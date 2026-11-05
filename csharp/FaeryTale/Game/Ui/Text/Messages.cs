using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Ui;

/// Printing messages in the text panel: word-wrapping story text and the "found ..." announcements.
///
/// The text panel at the bottom of the screen shows the last few lines of messages. Print scrolls
/// the panel up one line and writes a new line at the bottom; PrintCont adds more text to the line
/// that is already there. PrintWrapped is the general routine: it takes a message string from the
/// story tables, breaks it into lines that fit the panel and prints each one.
public static class Messages
{
    /// Characters on a line before PrintWrapped looks for a place to wrap it.
    private const short WRAP_COLUMN = 37;

    /// Past this count PrintWrapped knows the message is finished.
    private const short END_COLUMN = 38;

    /// Set when the message text ends, to leave the line loop.
    private const short MESSAGE_ENDED = 1000;
    private const byte CARRIAGE_RETURN = 13;

    /// In message text: replaced by the current brother's name.
    private const byte NAME_MARK = (byte)'%';

    /// Characters in the line buffer of PrintWrapped.
    private const int LINE_BUFFER_SIZE = 200;

    /// The current brother's name.
    public static string BrotherName(Data.Game game) => Tables.BrotherNames[game.Hero.Brother - 1];

    /// Prints message text, wrapping it into lines at spaces. A carriage return (13) starts a new
    /// line and a % is replaced by the current brother's name.
    ///
    /// How it works: characters are copied into a buffer, remembering where the last space was.
    /// When a line reaches 37 characters it is cut at that space (the space becomes the end of the
    /// string) and printed, and the leftover characters start the next line. The message ends at a
    /// zero byte (or the end of message).
    public static void PrintWrapped(Data.Game game, IOutput output, ReadOnlySpan<byte> message)
    {
        LineBuffer buffer = new();
        CharSource chars = new(message.ToArray());
        int lineStart = 0;
        short index = 0;

        while (true)
        {
            int? lineBreak = CopyUpToALine(game, chars, buffer, ref index);

            if (lineBreak is { } breakAt)
            {
                // Found a wrap point: end the line and print it.
                buffer.Terminate(breakAt);
                output.PrintBytes(buffer.Line(lineStart));

                if (index > END_COLUMN)
                {
                    break;
                }

                index = (short)(buffer.End - (breakAt + 1));
                lineStart = breakAt + 1;
            }
            else
            {
                // No wrap point, so print the whole thing.
                buffer.Put(0);

                if (index > END_COLUMN)
                {
                    break;
                }

                output.PrintBytes(buffer.Line(lineStart));
                index = 0;
                lineStart = buffer.End;
            }
        }
    }

    /// Prints "[Brother] found [container] containing ", ready for the contents to be added.
    public static void AnnounceContainer(Data.Game game, IOutput output, string container)
    {
        output.Print(BrotherName(game));
        output.PrintCont(" found ");
        output.PrintCont(container);
        output.PrintCont(" containing ");
    }

    /// Prints "[Brother] found [treasure]".
    public static void AnnounceTreasure(Data.Game game, IOutput output, string treasure)
    {
        output.Print(BrotherName(game));
        output.PrintCont(" found ");
        output.PrintCont(treasure);
    }

    /// Adds the current brother's name to the line being printed.
    public static void Name(Data.Game game, IOutput output) => output.PrintCont(BrotherName(game));

    // Copies message characters into the buffer until a line is full or the text ends, counting
    // them in index and returning where the last space (or the end) was, or null if there was none.
    private static int? CopyUpToALine(Data.Game game, CharSource chars, LineBuffer buffer, ref short index)
    {
        int? lineBreak = null;

        while (index < WRAP_COLUMN)
        {
            byte c = chars.Next();

            if (c == (byte)' ')
            {
                lineBreak = buffer.End;
            }

            if (c == 0)
            {
                lineBreak = buffer.End;
                buffer.Terminate(buffer.End);
                index = MESSAGE_ENDED;
            }

            if (c == CARRIAGE_RETURN)
            {
                lineBreak = buffer.End;
                buffer.Terminate(buffer.End);

                break;
            }

            if (c == NAME_MARK)
            {
                index += buffer.PutBrotherName(game);
            }
            else
            {
                buffer.Put(c);
            }

            index++;
        }

        return lineBreak;
    }

    // The message characters, then zeros for ever.
    private sealed class CharSource(byte[] message)
    {
        private int position;

        public byte Next() => position < message.Length ? message[position++] : (byte)0;
    }

    // The line buffer: message characters are collected here, and End is where the next one goes.
    private sealed class LineBuffer
    {
        private readonly byte[] bytes = new byte[LINE_BUFFER_SIZE];

        public int End { get; private set; }

        public void Put(byte c)
        {
            if (End < bytes.Length)
            {
                bytes[End] = c;
            }

            End++;
        }

        // Ends the text at the position at (without moving the end).
        public void Terminate(int at)
        {
            if (at >= 0 && at < bytes.Length)
            {
                bytes[at] = 0;
            }
        }

        // The line that starts at start and runs to the zero byte.
        public ReadOnlySpan<byte> Line(int start)
        {
            if (start >= bytes.Length)
            {
                return ReadOnlySpan<byte>.Empty;
            }

            ReadOnlySpan<byte> rest = bytes.AsSpan(start);
            int length = rest.IndexOf((byte)0);

            return rest[..(length < 0 ? rest.Length : length)];
        }

        // Adds the current brother's name. Returns its length.
        public short PutBrotherName(Data.Game game)
        {
            string name = BrotherName(game);

            foreach (char c in name)
            {
                Put((byte)c);
            }

            return (short)name.Length;
        }
    }
}
