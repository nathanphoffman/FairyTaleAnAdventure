using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class RemasteredKeysTests
{
    [Theory]
    [InlineData("W", "Up")]
    [InlineData("a", "Left")]
    [InlineData("S", "Down")]
    [InlineData("D", "Right")]
    public void WasdWalksLikeTheArrows(string pressed, string arrow) =>
        Assert.Equal(arrow, RemasteredKeys.Translate(pressed));

    [Fact]
    public void TheMenuShortcutsThatLostTheirLettersGetNewOnes()
    {
        Assert.Equal("S", RemasteredKeys.Translate("H")); // Say
        Assert.Equal("A", RemasteredKeys.Translate("N")); // Ask
        Assert.Equal("W", RemasteredKeys.Translate("Z")); // Sword in the Buy menu
    }

    [Fact]
    public void OtherKeysAreLeftAlone() => Assert.Equal("I", RemasteredKeys.Translate("I"));
}
