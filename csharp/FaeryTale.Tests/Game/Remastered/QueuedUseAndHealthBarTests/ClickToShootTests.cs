using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class ClickToShootTests
{
    [Theory]
    [InlineData(50, 0, 0, 3)]
    [InlineData(-50, 2, 2, 7)]
    [InlineData(0, 40, 0, 5)]
    [InlineData(30, 30, 0, 4)]
    [InlineData(-30, 30, 0, 6)]
    public void NearestLineFindsTheDirectionAndGap(int deltaX, int deltaY, int offset, int direction)
    {
        (double gap, short facing) = ClickToShoot.NearestLine(deltaX, deltaY);

        Assert.Equal(offset, gap, 3);
        Assert.Equal(direction, facing);
    }
}
