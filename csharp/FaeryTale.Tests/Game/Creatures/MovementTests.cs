using FaeryTale.Game.Creatures.Movement;

namespace FaeryTale.Tests;

public class MovementTests
{
    // The movement helpers are internal to the game project; this alias keeps the tests short.
    private const sbyte ENVIRON_BACKWARDS = -3;
    private const sbyte ENVIRON_SLOW = -1;
    private const int CARRIER_SLOT = 3;

    [Fact]
    public void TheSwanNeedsTheLassoWhereTheTurtleDoesNot()
    {
        Assert.True(MoveContextProbe.SwanCarries(1, CARRIER_SLOT, true));
        Assert.False(MoveContextProbe.SwanCarries(1, CARRIER_SLOT, false));
        Assert.False(MoveContextProbe.SwanCarries(0, CARRIER_SLOT, true));
        Assert.False(MoveContextProbe.SwanCarries(1, CARRIER_SLOT + 1, true));
        Assert.True(MoveContextProbe.TurtleCarries(1, CARRIER_SLOT));
        Assert.False(MoveContextProbe.TurtleCarries(0, CARRIER_SLOT));
        Assert.False(MoveContextProbe.TurtleCarries(1, 0));
    }

    [Fact]
    public void TheTurtleKeepsToTheWater()
    {
        // Straight on, then right, then two left of that, then one more left.
        Assert.Equal(2, MoveContextProbe.TurtleHeading(2, _ => true));
        Assert.Equal(3, MoveContextProbe.TurtleHeading(2, d => d == 3));
        Assert.Equal(1, MoveContextProbe.TurtleHeading(2, d => d == 1));
        Assert.Equal(0, MoveContextProbe.TurtleHeading(2, _ => false));

        // The compass wraps.
        Assert.Equal(0, MoveContextProbe.TurtleHeading(7, d => d == 0));
        Assert.Equal(7, MoveContextProbe.TurtleHeading(0, d => d == 7));
    }

    [Fact]
    public void BeingStuckShowsImpatience()
    {
        Assert.Null(MoveContextProbe.FrustratedFrame(20, 0));
        Assert.Equal((short?)84, MoveContextProbe.FrustratedFrame(21, 0));
        Assert.Equal((short?)85, MoveContextProbe.FrustratedFrame(21, 2));
        Assert.Equal((short?)40, MoveContextProbe.FrustratedFrame(41, 0));
    }

    [Fact]
    public void WalkingSpeeds()
    {
        Assert.Equal(3, MoveContextProbe.StepLength(true, 0));
        Assert.Equal(-2, MoveContextProbe.StepLength(false, ENVIRON_BACKWARDS));
        Assert.Equal(4, MoveContextProbe.StepLength(false, ENVIRON_SLOW));
        Assert.Equal(1, MoveContextProbe.StepLength(false, 2));
        Assert.Equal(1, MoveContextProbe.StepLength(false, 7));
        Assert.Equal(2, MoveContextProbe.StepLength(false, 0));
        Assert.Equal(2, MoveContextProbe.StepLength(false, 6));
    }

    [Fact]
    public void ActorsClimbOutOfShallowWater()
    {
        Assert.True(MoveContextProbe.ClimbsOutOfWater(3, 0));
        Assert.False(MoveContextProbe.ClimbsOutOfWater(2, 0));
        Assert.True(MoveContextProbe.ClimbsOutOfWater(6, 3));
        Assert.False(MoveContextProbe.ClimbsOutOfWater(5, 3));
        Assert.True(MoveContextProbe.ClimbsOutOfWater(11, 4));
        Assert.False(MoveContextProbe.ClimbsOutOfWater(10, 4));
        Assert.False(MoveContextProbe.ClimbsOutOfWater(20, 5));
    }

    [Fact]
    public void WadingDeeperSinksBeforeDrowning()
    {
        // Deep water (5): wade towards 30.
        Assert.Equal((1, "Nothing"), MoveContextProbe.WadeStep(0, 5, false));
        Assert.Equal((16, "Sinking"), MoveContextProbe.WadeStep(15, 5, false));
        Assert.Equal((30, "Drowned"), MoveContextProbe.WadeStep(29, 5, false));
        Assert.Equal((30, "Nothing"), MoveContextProbe.WadeStep(30, 5, false));

        // Shallow water (4) stops at 10 and comes back up from deeper.
        Assert.Equal((10, "Nothing"), MoveContextProbe.WadeStep(9, 4, false));
        Assert.Equal((10, "Nothing"), MoveContextProbe.WadeStep(10, 4, false));
        Assert.Equal((11, "Nothing"), MoveContextProbe.WadeStep(12, 4, false));

        // The dead are not affected.
        Assert.Equal((16, "Nothing"), MoveContextProbe.WadeStep(15, 5, true));
    }
}
