using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class ClickToMoveTests
{
    private static Func<int, int, int> Open => (_, _) => 0;

    [Fact]
    public void ARouteOnOpenGroundGoesStraightThere()
    {
        List<(int X, int Y)> route = PathFinder.Find((5, 0), Open);

        Assert.Equal(5, route.Count);
        Assert.Equal((5, 0), route[^1]);
        Assert.All(route, cell => Assert.Equal(0, cell.Y));
    }

    [Fact]
    public void ARouteGoesAroundAWall()
    {
        // A wall of blocked cells across x = 3 from y = -4 to y = 4.
        Func<int, int, int> wall = (x, y) => x == 3 && Math.Abs(y) <= 4 ? -1 : 0;
        List<(int X, int Y)> route = PathFinder.Find((6, 0), wall);

        Assert.Equal((6, 0), route[^1]);
        Assert.DoesNotContain(route, cell => wall(cell.X, cell.Y) < 0);
        Assert.Contains(route, cell => Math.Abs(cell.Y) >= 5);
    }

    [Fact]
    public void ABlockedGoalEndsAsNearAsPossible()
    {
        Func<int, int, int> block = (x, y) => x >= 5 ? -1 : 0;
        List<(int X, int Y)> route = PathFinder.Find((8, 0), block);

        Assert.Equal((4, 0), route[^1]);
    }

    [Fact]
    public void ADiagonalDoesNotSqueezeBetweenTwoBlockedCorners()
    {
        // (1, 0) and (0, 1) are blocked, so (1, 1) cannot be reached by cutting the corner.
        Func<int, int, int> corner = (x, y) => (x, y) is (1, 0) or (0, 1) ? -1 : 0;
        List<(int X, int Y)> route = PathFinder.Find((1, 1), corner);

        Assert.True(route.Count > 1 || route[^1] != (1, 1));
    }

    [Fact]
    public void ACostlyCellIsAvoidedWhenThereIsADryWay()
    {
        Func<int, int, int> puddle = (x, y) => x == 2 && y == 0 ? 500 : 0;
        List<(int X, int Y)> route = PathFinder.Find((4, 0), puddle);

        Assert.DoesNotContain((2, 0), route);
    }

    [Theory]
    [InlineData(-9, -9, 0)]
    [InlineData(0, -9, 1)]
    [InlineData(9, -9, 2)]
    [InlineData(9, 0, 3)]
    [InlineData(9, 9, 4)]
    [InlineData(0, 9, 5)]
    [InlineData(-9, 9, 6)]
    [InlineData(-9, 0, 7)]
    [InlineData(1, 1, 9)]
    public void TheWalkDirectionPointsAtTheTarget(int dx, int dy, int expected) =>
        Assert.Equal(expected, ClickToMove.DirectionTo(dx, dy));

    [Fact]
    public void AClickAndTheMarkerUseTheSameGroundSpot()
    {
        World world = new() { MapX = 18892, MapY = 15675 };
        (int X, int Y) ground = ScreenMap.ToWorld(world, (250, 90));

        Assert.Equal((18892 + 250 - 16, 15675 + 90), ground);
        Assert.Equal((250, 90), ScreenMap.ToScreen(world, ground));
    }

    [Fact]
    public void TheMarkerMovesBackAsTheMapScrollsSoItStaysOnTheGround()
    {
        World world = new() { MapX = 18892, MapY = 15675 };
        (int X, int Y) ground = ScreenMap.ToWorld(world, (250, 90));

        world.MapX += 7;
        world.MapY -= 3;

        Assert.Equal((243, 93), ScreenMap.ToScreen(world, ground));
    }
}
