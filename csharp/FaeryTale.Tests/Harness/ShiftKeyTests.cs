using FaeryTale.Game.Input;
using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class ShiftKeyTests
{
    [Fact]
    public void ShiftIsHeldWhileEitherShiftKeyIsDown()
    {
        InputBridge.KeyEvent("Left Shift", true);
        Assert.True(InputHandler.ShiftHeld);

        InputBridge.KeyEvent("Right Shift", true);
        InputBridge.KeyEvent("Left Shift", false);
        Assert.True(InputHandler.ShiftHeld);

        InputBridge.KeyEvent("Right Shift", false);
        Assert.False(InputHandler.ShiftHeld);
    }
}
