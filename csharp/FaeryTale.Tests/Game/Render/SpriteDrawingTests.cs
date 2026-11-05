using FaeryTale.Game.Render;

namespace FaeryTale.Tests;

public class SpriteDrawingTests
{
    private static Sprite SpriteAt(short atype, short inum, short xStart, short yStart, short xStop, short yStop) =>
        new() { Atype = atype, Inum = inum, XStart = xStart, YStart = yStart, XStop = xStop, YStop = yStop, Ground = (short)(yStart + 32) };

    [Fact]
    public void ASpriteInsideTheScreenIsNotCut()
    {
        Sprite sprite = SpriteAt(SpriteImages.PHIL, 5, 100, 50, 131, 81);
        Clip clip = SpriteDrawing.ClipToScreen(sprite);

        Assert.Equal((100, 50, 131, 81), (clip.XStart, clip.YStart, clip.XStop, clip.YStop));
        Assert.Equal((0, 0), (clip.XOff, clip.YOff));
    }

    [Fact]
    public void ASpriteOverTheLeftAndTopEdgesIsCut()
    {
        Clip clip = SpriteDrawing.ClipToScreen(SpriteAt(SpriteImages.PHIL, 5, -5, -10, 26, 21));

        Assert.Equal((0, 0), (clip.XStart, clip.YStart));
        Assert.Equal((5, 10), (clip.XOff, clip.YOff));
    }

    [Fact]
    public void ASpriteOverTheRightAndBottomEdgesStopsAtTheScreen()
    {
        Clip clip = SpriteDrawing.ClipToScreen(SpriteAt(SpriteImages.PHIL, 5, 300, 160, 331, 191));

        Assert.Equal((SpriteDrawing.SCREEN_RIGHT, SpriteDrawing.SCREEN_BOTTOM), (clip.XStop, clip.YStop));
    }

    [Fact]
    public void ImagesWithBitSevenStartEightLinesIntoThePicture()
    {
        Sprite sprite = SpriteAt(SpriteImages.OBJECTS, 0x85, 10, 10, 25, 17);
        Clip clip = SpriteDrawing.ClipToScreen(sprite);

        Assert.Equal(5, sprite.Inum);
        Assert.Equal(8, clip.YOff);
    }

    [Fact]
    public void SmallObjectsAreEightLinesHighAndOtherImagesKeepTheSheetHeight()
    {
        foreach (short small in new short[] { 0x1b, 8, 12, 25, 26, 0x11, 0x17, 0x80 })
        {
            Assert.Equal(8, SpriteImages.ImageHeight(SpriteImages.OBJECTS, small, 32));
        }

        foreach (short big in new short[] { 0, 7, 13, 24, 28, 0x10, 0x18 })
        {
            Assert.Equal(32, SpriteImages.ImageHeight(SpriteImages.OBJECTS, big, 32));
        }

        Assert.Equal(32, SpriteImages.ImageHeight(SpriteImages.ENEMY, 0x1b, 32));
    }

    [Fact]
    public void TableLookupsOutsideTheTableGiveZero()
    {
        Assert.Equal(FaeryTale.Game.Data.ObjectTables.BowX[0], SpriteImages.ByteOf(FaeryTale.Game.Data.ObjectTables.BowX, 0));
        Assert.Equal(0, SpriteImages.ByteOf(FaeryTale.Game.Data.ObjectTables.BowX, 32));
        Assert.Equal(0, SpriteImages.ByteOf(FaeryTale.Game.Data.ObjectTables.BowX, -1));
    }

    [Fact]
    public void SceneryRulesByTileType()
    {
        const SceneryMasks.SceneryRule Skip = SceneryMasks.SceneryRule.Skip;
        const SceneryMasks.SceneryRule Tile = SceneryMasks.SceneryRule.UseTileMask;
        const SceneryMasks.SceneryRule Full = SceneryMasks.SceneryRule.UseFullMask;

        Assert.Equal(Skip, SceneryMasks.RuleFor(0, 1, 0, 0, false));
        Assert.Equal(Skip, SceneryMasks.RuleFor(1, 0, 0, 0, false));
        Assert.Equal(Tile, SceneryMasks.RuleFor(1, 1, 0, 0, false));
        Assert.Equal(Skip, SceneryMasks.RuleFor(2, 0, 0, 36, false));
        Assert.Equal(Tile, SceneryMasks.RuleFor(2, 0, 0, 35, false));
        Assert.Equal(Skip, SceneryMasks.RuleFor(3, 0, 0, 0, true));
        Assert.Equal(Tile, SceneryMasks.RuleFor(3, 0, 0, 0, false));
        Assert.Equal(Skip, SceneryMasks.RuleFor(4, 0, 0, 0, false));
        Assert.Equal(Skip, SceneryMasks.RuleFor(5, 0, 0, 36, false));
        Assert.Equal(Tile, SceneryMasks.RuleFor(5, 0, 0, 10, false));
        Assert.Equal(Tile, SceneryMasks.RuleFor(6, 0, 0, 0, false));
        Assert.Equal(Full, SceneryMasks.RuleFor(6, 0, 1, 0, false));
        Assert.Equal(Skip, SceneryMasks.RuleFor(7, 0, 0, 21, false));
        Assert.Equal(Tile, SceneryMasks.RuleFor(8, 0, 0, 99, false));
    }
}
