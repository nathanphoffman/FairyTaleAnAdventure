using FaeryTale.Game.Input;

namespace FaeryTale.Tests;

public class MenuKeyBoundsTests
{
    [Fact]
    public void TheBottomLinesOfTheScreenAreNotAMenuEntry()
    {
        // In Remastered the pointer can reach the last lines of the screen, below the 6 menu rows.
        Assert.Null(InputHandler.MenuKeyAt(230, 199));
        Assert.Null(InputHandler.MenuKeyAt(250, 198));
    }

    [Fact]
    public void TheLastRowOfTheMenuStillWorks() => Assert.Equal('a' + 10, InputHandler.MenuKeyAt(230, 144 + 5 * 9));
}
