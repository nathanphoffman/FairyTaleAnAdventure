using System.Text;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class MessageHistoryTests
{
    private static byte[] Bytes(string text) => Encoding.Latin1.GetBytes(text);

    private static MessageHistory WithLines(int count)
    {
        MessageHistory history = new();

        for (int number = 1; number <= count; number++)
        {
            history.StartLine(Bytes($"line {number}"));
        }

        return history;
    }

    private static string Show(MessageHistory history) =>
        string.Join("|", history.Window().Select(line => Encoding.Latin1.GetString(line)));

    [Fact]
    public void TheWindowShowsTheNewestLinesUntilScrolled()
    {
        MessageHistory history = WithLines(6);

        Assert.Equal("line 3|line 4|line 5|line 6", Show(history));

        history.ScrollBy(2);

        Assert.Equal("line 1|line 2|line 3|line 4", Show(history));
    }

    [Fact]
    public void ScrollingStopsAtTheOldestAndTheNewestLine()
    {
        MessageHistory history = WithLines(6);

        Assert.True(history.ScrollBy(100));
        Assert.Equal(2, history.Scrolled);
        Assert.False(history.ScrollBy(1));
        Assert.True(history.ScrollToNewest());
        Assert.False(history.ScrollBy(-1));
    }

    [Fact]
    public void TextPrintedAfterALineContinuesIt()
    {
        MessageHistory history = new();

        history.StartLine(Bytes("Julian found "));
        history.Append(Bytes("a sword."));

        Assert.Equal("Julian found a sword.", Show(history));
    }

    [Fact]
    public void TextPrintedElsewhereIsNotPartOfTheHistory()
    {
        MessageHistory history = new();

        history.StartLine(Bytes("hello"));
        history.OnMessageLine = false;
        history.Append(Bytes("Brv: 35"));

        Assert.Equal("hello", Show(history));
    }

    [Fact]
    public void AShortHistoryShowsWhatThereIs()
    {
        Assert.Equal("line 1|line 2", Show(WithLines(2)));
    }
}
