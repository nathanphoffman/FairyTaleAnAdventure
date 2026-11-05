using FaeryTale.Game.Land.Map;

namespace FaeryTale.Tests;

public class MapTests
{
    [Fact]
    public void SectorColumnsWrapOutsideTheMap()
    {
        Assert.Equal(5, MapLookup.WrapSectorColumn(5));
        Assert.Equal(63, MapLookup.WrapSectorColumn(64));
        Assert.Equal(0, MapLookup.WrapSectorColumn(96));
    }

    [Fact]
    public void SectorRowsAreClamped()
    {
        Assert.Equal(0, MapLookup.ClampSectorRow(-3));
        Assert.Equal(31, MapLookup.ClampSectorRow(40));
        Assert.Equal(7, MapLookup.ClampSectorRow(7));
    }

    [Fact]
    public void PageTurnInsetCoversEveryRange()
    {
        Assert.Equal(9, MapCamera.PageTurnInset(0));
        Assert.Equal(4, MapCamera.PageTurnInset(10));
        Assert.Equal(3, MapCamera.PageTurnInset(11));
        Assert.Equal(4, MapCamera.PageTurnInset(72));
        Assert.Equal(5, MapCamera.PageTurnInset(99));
        Assert.Equal(6, MapCamera.PageTurnInset(124));
        Assert.Equal(7, MapCamera.PageTurnInset(136));
        Assert.Equal(10, MapCamera.PageTurnInset(137));
    }

    [Fact]
    public void CameraStaysWhileTheHeroIsNear()
    {
        Assert.Equal(((ushort)100, (ushort)100), MapCamera.CameraAfter(100, 100, 244 + 10, 170 + 5));
    }

    [Fact]
    public void CameraCreepsBeforeJumping()
    {
        Assert.Equal(((ushort)101, (ushort)100), MapCamera.CameraAfter(100, 100, 144 + 100 + 30, 170));
        Assert.Equal(((ushort)110, (ushort)100), MapCamera.CameraAfter(100, 100, 144 + 100 + 80, 170));
        Assert.Equal(((ushort)100, (ushort)106), MapCamera.CameraAfter(100, 100, 244, 70 + 100 + 50));
    }

    [Fact]
    public void StepsFollowTheDirectionTables()
    {
        Assert.Equal(104, MapSteps.StepX(100, 2, 4)); // east, speed 4: +2 * 4 / 2
        Assert.Equal(96, MapSteps.StepX(100, 6, 4));
        Assert.Equal(100, MapSteps.StepX(100, 9, 4)); // no move
        Assert.Equal(97, MapSteps.StepY(100, 1, 2)); // north: the negative step wraps in 16 bits, then stays 15 bit
    }

    [Fact]
    public void StepYKeepsBit15()
    {
        Assert.Equal(0x8000, MapSteps.StepY(0x8000 | 100, 4, 2) & 0x8000);
    }

    [Fact]
    public void WrapCopiesBit14ToBit15()
    {
        Assert.Equal(0xc000, MapSteps.WrapCoordinate(0x4000));
        Assert.Equal(0x1234, MapSteps.WrapCoordinate(0x1234));
        Assert.Equal(0x1234, MapSteps.WrapCoordinate(0x9234));
    }
}
