using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class SwingArcTests
{
    // Directions as the game numbers them, clockwise from north west: 1 is north, 3 east, 5 south, 7 west.
    private const int NORTH = 1;
    private const int SOUTH = 5;
    private const int RANGE = 20;

    private static Shape At(int x, int y) => new() { AbsX = (ushort)x, AbsY = (ushort)y };

    [Fact]
    public void AnEnemyBesideTheHeroIsInTheSwing()
    {
        Assert.True(SwingArc.Covers(At(1000, 1000), At(1015, 1000), SOUTH, RANGE));
        Assert.True(SwingArc.Covers(At(1000, 1000), At(985, 1000), SOUTH, RANGE));
    }

    [Fact]
    public void AnEnemyStraightAheadIsInTheSwing()
    {
        Assert.True(SwingArc.Covers(At(1000, 1000), At(1000, 1012), SOUTH, RANGE));
    }

    [Fact]
    public void AnEnemyBehindTheHeroIsNot()
    {
        Assert.False(SwingArc.Covers(At(1000, 1000), At(1000, 990), SOUTH, RANGE));
    }

    [Fact]
    public void AnEnemyOutOfReachIsNot()
    {
        Assert.False(SwingArc.Covers(At(1000, 1000), At(1000, 1030), SOUTH, RANGE));
        Assert.True(SwingArc.Covers(At(1000, 1000), At(1000, 970), NORTH, 40));
    }
}
