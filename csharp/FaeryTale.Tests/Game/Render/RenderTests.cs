using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Tests;

public class RenderTests
{
    [Fact]
    public void SideTellsWhichWayTheHeroIs()
    {
        // A line along x; a point below it (positive y) has a positive cross product.
        Assert.Equal(50, WitchBeam.Side((0, 0), (10, 0), 5, 5));
        Assert.Equal(-50, WitchBeam.Side((0, 0), (10, 0), 5, -5));
    }

    [Fact]
    public void OverviewOriginIsClampedToTheMap()
    {
        Assert.Equal(((short)0, (short)0), OverviewMap.OverviewOrigin(0, 0, 0, 0));
        Assert.Equal(((short)39, (short)12), OverviewMap.OverviewOrigin(0x3000, 0x1000, 0, 0));
        Assert.Equal(((short)46, (short)23), OverviewMap.OverviewOrigin(0x5000, 0x2800, 0, 0));

        // Positions that wrap around the 128 sector map.
        Assert.Equal(((short)0, (short)0), OverviewMap.OverviewOrigin(0x7f00, 0x7f00, 0, 0));
    }

    [Fact]
    public void SunrisePaletteKeepsFixedColoursFadingTheGlow()
    {
        ushort[] early = ScreenEffects.SunrisePalette(25);

        Assert.Equal((0, 0xfff, 0xfff, 0), (early[0], early[1], early[28], early[31]));
        Assert.Equal((0x800, 0x400), (early[29], early[30]));

        ushort[] late = ScreenEffects.SunrisePalette(-14);

        Assert.Equal((0x100 * 8, 0x100 * 4), (late[29], late[30]));
        Assert.All(ScreenEffects.SunrisePalette(-30)[2..28], color => Assert.Equal(0, color));
    }

    [Fact]
    public void LiteralRunsAreCopiedAndRepeatRunsExpanded()
    {
        (byte[] line, int position) = IffPicture.UnpackForTest([2, 10, 20, 30], true, 3);

        Assert.Equal(new byte[] { 10, 20, 30 }, line);
        Assert.Equal(4, position);

        (line, position) = IffPicture.UnpackForTest([unchecked((byte)-3), 7], true, 4);

        Assert.Equal(new byte[] { 7, 7, 7, 7 }, line);
        Assert.Equal(2, position);
    }

    [Fact]
    public void AnUncompressedRowIsCopiedAndRowsAreWholeWords()
    {
        (byte[] line, _) = IffPicture.UnpackForTest([5, 6, 7, 8], false, 2);

        Assert.Equal(new byte[] { 5, 6 }, line);
        Assert.Equal(40, IffPicture.RowBytesForTest(320));
    }

    // A tiny 16 x 2 picture with 2 planes, uncompressed.
    private static byte[] TinyPicture()
    {
        byte[] header = new byte[20];

        header[1] = 16;
        header[3] = 2;
        header[8] = 2;

        byte[] pixels = [0xff, 0x00, 0x00, 0xff, 0x0f, 0xf0, 0xf0, 0x0f];
        List<byte> form = [.. "ILBM"u8.ToArray(), .. "BMHD"u8.ToArray(), 0, 0, 0, (byte)header.Length, .. header, .. "BODY"u8.ToArray(), 0, 0, 0, (byte)pixels.Length, .. pixels];
        List<byte> file = [.. "FORM"u8.ToArray(), 0, 0, 0, (byte)form.Count, .. form];

        return [.. file];
    }

    [Fact]
    public void APictureFileIsUnpackedIntoThePlanesAndJunkIsRefused()
    {
        Bitmap bitmap = new(16, 2, 2);

        Assert.True(IffPicture.ReadForm(TinyPicture(), bitmap, 0));
        Assert.Equal(new byte[] { 0xff, 0x00 }, bitmap.Plane(0)[..2].ToArray());
        Assert.Equal(new byte[] { 0x00, 0xff }, bitmap.Plane(1)[..2].ToArray());
        Assert.Equal(new byte[] { 0x0f, 0xf0 }, bitmap.Plane(0)[2..4].ToArray());
        Assert.False(IffPicture.ReadForm("JUNKJUNKJUNK"u8.ToArray(), bitmap, 0));
    }

    [Fact]
    public void ABlankVideoDrawsATileFromTheImageBank()
    {
        Video video = new();
        video.Screen.SetBitmap(BitmapId.Page1, new Bitmap(320, 200, 5));
        video.Graphics.Image = new byte[MapTiles.IMAGE_PLANE_SIZE * 5];
        video.Graphics.Image[MapTiles.TILE_BYTES * 3] = 0xAB;

        MapTiles.DrawTile(video.Graphics, video.Screen, 3, 0);

        Assert.Equal(0xAB, video.Screen.DrawBitmap.Plane(0)[0]);
        Assert.Equal(0, video.Screen.DrawBitmap.Plane(1)[0]);
    }
}
