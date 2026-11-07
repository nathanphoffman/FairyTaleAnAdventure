using System.Text;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class MessageWrapperTests
{
    // Every character is 8 pixels wide, so 40 pixels is five characters.
    private static int Eight(byte character) => 8;

    private static string[] Wrapped(string text, int width) =>
        MessageWrapper.Wrap(Encoding.Latin1.GetBytes(text), width, Eight).Select(line => Encoding.Latin1.GetString(line)).ToArray();

    [Fact]
    public void ATextThatFitsStaysOneLine()
    {
        Assert.Equal(["Hello"], Wrapped("Hello", 40));
    }

    [Fact]
    public void ALongTextBreaksAtSpaces()
    {
        Assert.Equal(["one", "two", "three"], Wrapped("one two three", 40));
        Assert.Equal(["ab cd", "ef gh"], Wrapped("ab cd ef gh", 40));
    }

    [Fact]
    public void AWordTooLongForALineIsCutWhereTheLineIsFull()
    {
        Assert.Equal(["abcde", "fghij", "k"], Wrapped("abcdefghijk", 40));
    }

    [Fact]
    public void AnEmptyTextIsOneEmptyLine()
    {
        Assert.Equal([""], Wrapped("", 40));
    }

    [Fact]
    public void NoLineIsWiderThanTheWindow()
    {
        string[] lines = Wrapped("You're like a kid in an Amiga store in 1988 and more words follow here", 80);

        Assert.All(lines, line => Assert.True(line.Length * 8 <= 80));
        Assert.Equal("You're like a kid in an Amiga store in 1988 and more words follow here", string.Join(" ", lines));
    }
}
