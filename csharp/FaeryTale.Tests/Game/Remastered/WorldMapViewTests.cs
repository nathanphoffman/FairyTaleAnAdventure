using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class WorldMapViewTests
{
    private static uint[] BlankWorld() => new uint[WorldOverview.COLUMNS * WorldOverview.ROWS];

    [Fact]
    public void AWorldPositionIsOnTheTileOfThePictureItIsOver()
    {
        // Tiles are 16 pixels wide and 32 high.
        Assert.Equal((2, 3), WorldOverview.TileOf(2 * 16 + 5, 3 * 32 + 9));
        Assert.Equal((WorldOverview.COLUMNS - 1, WorldOverview.ROWS - 1), WorldOverview.TileOf(99999, 99999));
    }

    [Fact]
    public void ZoomingStopsAtTheLimits()
    {
        MapView view = new();

        view.ZoomBySteps(100);
        Assert.Equal(MapView.MAX_SCALE, view.Scale);

        view.ZoomBySteps(-100);
        Assert.Equal(MapView.MIN_SCALE, view.Scale);
    }

    [Fact]
    public void ScrollingStaysInsideTheWorld()
    {
        MapView view = new();

        view.CenterOn((10, 10));
        view.PanBy(-100000, -100000);

        Assert.Equal(0, view.CenterColumn);
        Assert.Equal(0, view.CenterRow);
    }

    [Fact]
    public void ADraggedViewMovesTheLandWithThePointer()
    {
        MapView view = new();

        view.CenterOn((500, 500));
        view.ZoomBySteps(0);
        view.PanBy(20, 0);

        // Scale 1: 20 units across are 20 tile columns.
        Assert.Equal(520, view.CenterColumn);
        Assert.Equal(500, view.CenterRow);
    }

    [Fact]
    public void TheHeroMarkIsKeptInsideTheViewWhenTheHeroIsFarOff()
    {
        MapView view = new();

        view.CenterOn((100, 100));

        FaeryTale.Gfx.Image picture = view.Picture(BlankWorld(), (2000, 1000), true);

        Assert.Equal(MapView.WIDTH * MapView.HEIGHT, picture.Pixels.Length);
        Assert.Contains(picture.Pixels, pixel => pixel == 0xffe0_1010);
    }

    [Fact]
    public void TheMinimapFitsAtTheEdgesOfTheWorld()
    {
        foreach ((int column, int row) in new[] { (0, 0), (WorldOverview.COLUMNS - 1, WorldOverview.ROWS - 1) })
        {
            FaeryTale.Gfx.Image picture = MinimapPicture.Of(BlankWorld(), (column, row));

            Assert.Equal(MinimapPicture.WIDTH * MinimapPicture.HEIGHT, picture.Pixels.Length);
        }
    }
}
