using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Tests;

public class StoryTextTests
{
    [Fact]
    public void MessagesAreNulSeparatedAndEndWithAnExtraNul()
    {
        Assert.Equal(new byte[] { 0, 0 }, StoryText.EVENT_MSG[^2..]);
        Assert.StartsWith("% was getting rather hungry.", TextEncoding.FromBytes(StoryText.EVENT_MSG));
    }

    [Fact]
    public void TheStoryHasTwentyPagesAndTheBitmapsAre256Bytes()
    {
        Assert.Equal(20, StoryText.PLACARD_MSGS.Length);
        Assert.Equal(256, StoryText.HINOR.Length);
        Assert.Equal(256, StoryText.HIVAR.Length);
        Assert.Equal(87, StoryText.PLACE_TBL.Length);
        Assert.Equal(111, StoryText.INSIDE_TBL.Length);
    }

    [Fact]
    public void PositionEscapesSurviveAsRawBytes()
    {
        Assert.Equal(0x80, StoryText.PLACARD_MSGS[0][0]);
        Assert.Contains((byte)0xa0, StoryText.TITLE_TEXT);
    }
}
