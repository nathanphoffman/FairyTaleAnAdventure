using FaeryTale.Game.Land.TravelDoors;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class DoorTargetsTests
{

    private static FaeryTale.Game.Data.Game GameOutdoors()
    {
        FaeryTale.Game.Data.Game game = new();

        game.World.Region = 3;

        return game;
    }

    [Fact]
    public void ADoorIsFoundOnBothOfItsTiles()
    {
        FaeryTale.Game.Data.Game game = GameOutdoors();

        Assert.NotNull(DoorTargets.DoorAt(game, (18992 + 3, 15808 + 5)));
        Assert.NotNull(DoorTargets.DoorAt(game, (18992 + 20, 15808 + 5)));
        Assert.Null(DoorTargets.DoorAt(game, (18992 - 40, 15808 + 5)));
    }

    [Fact]
    public void TheEntryPointIsAPlaceTheGameLetsTheHeroGoThrough()
    {
        FaeryTale.Game.Data.Game game = GameOutdoors();
        int index = DoorTargets.DoorAt(game, (18992 + 3, 15808 + 5))!.Value;
        (int x, int y) = DoorTargets.EntryPoint(game, index);

        Assert.Equal(index, DoorTargets.DoorAt(game, (x, y)));
        Assert.True(EnteringDoors.MayEnter(EnteringDoors.DoorAt(index), (ushort)x, (ushort)y, 0));
    }

    [Fact]
    public void TheApproachIsInFrontOfAClosedHorizontalDoor()
    {
        FaeryTale.Game.Data.Game game = GameOutdoors();
        int index = DoorTargets.DoorAt(game, (18992 + 3, 15808 + 5))!.Value;
        (int x, int y) = DoorTargets.ApproachPoint(game, index);

        Assert.True(y > 15808 + 32);
        Assert.InRange(x, 18992, 18992 + 32);
    }
}
