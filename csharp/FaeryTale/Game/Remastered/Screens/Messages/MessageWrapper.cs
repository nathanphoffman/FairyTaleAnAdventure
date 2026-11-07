namespace FaeryTale.Game.Remastered;

/// Breaks a line of text into lines that fit the message window (Remastered and Reimagined), so that
/// whatever is printed wraps by itself and nobody has to count characters. A line breaks at a space; a
/// word too long for a line on its own breaks wherever it has to.
public static class MessageWrapper
{
    private const byte SPACE = (byte)' ';

    /// The lines for a text, none wider than maxWidth pixels. advanceOf gives how wide a character is.
    public static List<byte[]> Wrap(ReadOnlySpan<byte> text, int maxWidth, Func<byte, int> advanceOf)
    {
        List<byte[]> lines = [];
        List<byte> line = [];
        int width = 0;

        foreach (byte[] word in Words(text))
        {
            int wordWidth = WidthOf(word, advanceOf);
            int spaceWidth = line.Count == 0 ? 0 : advanceOf(SPACE);

            if (line.Count > 0 && width + spaceWidth + wordWidth > maxWidth)
            {
                lines.Add([.. line]);
                line.Clear();
                width = 0;
                spaceWidth = 0;
            }

            AddWord(lines, line, word, ref width, spaceWidth, (maxWidth, advanceOf));
        }

        lines.Add([.. line]);

        return lines;
    }

    // The words of a text, split at spaces (an empty text is one empty word, so it stays one empty line).
    private static List<byte[]> Words(ReadOnlySpan<byte> text)
    {
        List<byte[]> words = [];
        int start = 0;

        for (int index = 0; index <= text.Length; index++)
        {
            if (index == text.Length || text[index] == SPACE)
            {
                words.Add(text[start..index].ToArray());
                start = index + 1;
            }
        }

        return words;
    }

    // Puts a word on the line (after a space unless the line is empty). A word wider than a whole line is
    // cut where the line is full.
    private static void AddWord(List<byte[]> lines, List<byte> line, byte[] word, ref int width, int spaceWidth, (int Max, Func<byte, int> Advance) limits)
    {
        if (line.Count > 0)
        {
            line.Add(SPACE);
            width += spaceWidth;
        }

        foreach (byte character in word)
        {
            int advance = limits.Advance(character);

            if (width + advance > limits.Max && line.Count > 0)
            {
                lines.Add([.. line]);
                line.Clear();
                width = 0;
            }

            line.Add(character);
            width += advance;
        }
    }

    private static int WidthOf(byte[] word, Func<byte, int> advanceOf) => word.Sum(character => advanceOf(character));
}
