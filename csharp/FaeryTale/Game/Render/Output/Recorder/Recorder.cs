
namespace FaeryTale.Game.Output;

/// An output that draws nothing and writes down what the rules asked for, so the rules can run
/// (and be tested) without a screen. It counts a tick for each WaitFrame.
public sealed partial class Recorder : IOutput
{
    private readonly List<Call> calls = new();
    private readonly List<string> printedLines = new();

    public IReadOnlyList<Call> Calls => calls;

    public uint Ticks { get; private set; }

    /// The lines started with PrintBytes, each with what PrintContBytes added.
    public IReadOnlyList<string> PrintedLines => printedLines;

    public void PrintBytes(ReadOnlySpan<byte> text)
    {
        string line = TextEncoding.FromBytes(text);

        printedLines.Add(line);
        Record("PrintBytes", Quote(line));
    }

    public void PrintContBytes(ReadOnlySpan<byte> text)
    {
        string added = TextEncoding.FromBytes(text);

        if (printedLines.Count > 0)
        {
            printedLines[^1] += added;
        }

        Record("PrintContBytes", Quote(added));
    }

    public void PrintNumber(int value, int length) => Record("PrintNumber", value, length);

    private void Record(string name, params object[] arguments) =>
        calls.Add(new Call(name, string.Join(", ", arguments)));

    private static string Quote(string text) => $"\"{text}\"";

    private static string Hex(ReadOnlySpan<ushort> colors)
    {
        string[] words = new string[colors.Length];

        for (int index = 0; index < colors.Length; index++)
        {
            words[index] = $"0x{colors[index]:X4}";
        }

        return $"[{string.Join(", ", words)}]";
    }
}
