using FaeryTale.Gfx;

namespace FaeryTale.Tests;

public class GfxTests
{
    private static Pen PenOf(byte fg) => new() { Fg = fg };

    [Fact]
    public void RowsAreWholeWords()
    {
        Assert.Equal(40, Bitmap.BytesPerRowFor(320));
        Assert.Equal(4, Bitmap.BytesPerRowFor(17));
        Assert.Equal(2, Bitmap.BytesPerRowFor(1));
    }

    [Fact]
    public void APixelIsOneBitFromEachPlane()
    {
        Bitmap bitmap = new(32, 4, 5);

        bitmap.Plot(9, 2, 0b10101, false, 5);

        Assert.Equal(0b10101, bitmap.Pixel(9, 2));
        Assert.Equal(0, bitmap.Pixel(8, 2));
        Assert.Equal(0x40, bitmap.Plane(0)[2 * 4 + 1]);
    }

    [Fact]
    public void PlottingOutsideIsIgnoredAndComplementFlips()
    {
        Bitmap bitmap = new(16, 2, 2);

        bitmap.Plot(-1, 0, 3, false, 2);
        bitmap.Plot(16, 0, 3, false, 2);
        bitmap.Plot(0, 2, 3, false, 2);
        Assert.Equal(0, bitmap.Pixel(0, 0));

        bitmap.Plot(0, 0, 1, false, 2);
        bitmap.Plot(0, 0, 0, true, 2);
        Assert.Equal(0b10, bitmap.Pixel(0, 0));
    }

    [Fact]
    public void ClearingFillsEachPlaneByItsPenBit()
    {
        Bitmap bitmap = new(16, 2, 3);

        bitmap.Clear(0b101, 3);
        Assert.Equal(0b101, bitmap.Pixel(5, 1));

        bitmap.Clear(0, 1);
        Assert.Equal(0b100, bitmap.Pixel(5, 1));
    }

    [Fact]
    public void CopyingARectangleBetweenBitmaps()
    {
        Bitmap source = new(16, 4, 2);
        source.Plot(1, 1, 3, false, 2);
        Bitmap destination = new(16, 4, 2);

        Assert.Equal(2, BitmapOps.CopyRect(source, destination, new Rect(0, 0, 4, 4), (8, 0), 0b11));
        Assert.Equal(3, destination.Pixel(9, 1));
        Assert.Equal(0, destination.Pixel(1, 1));
    }

    [Fact]
    public void ScrollingMovesPixelsUpFillingTheBottom()
    {
        Bitmap bitmap = new(16, 4, 1);
        bitmap.Plot(2, 3, 1, false, 1);
        Rect area = new(0, 0, 16, 4);

        BitmapOps.Scroll(bitmap, area, (0, 1), 1, 1);
        Assert.Equal(1, bitmap.Pixel(2, 2));
        Assert.Equal(1, bitmap.Pixel(2, 3));

        BitmapOps.Scroll(bitmap, area, (0, 1), 0, 1);
        Assert.Equal(1, bitmap.Pixel(2, 1));
        Assert.Equal(0, bitmap.Pixel(2, 3));
    }

    [Fact]
    public void AWindowCopyMovesPixelsByAnyAmount()
    {
        Bitmap source = new(64, 4, 2);
        source.Plot(21, 2, 3, false, 2);
        Bitmap destination = new(32, 4, 2);

        BitmapOps.CopyWindow(source, destination, (13, 1));

        // The pixel was at x 21, y 2; the window starts at x 13, y 1.
        Assert.Equal(3, destination.Pixel(8, 1));
        Assert.Equal(0, destination.Pixel(7, 1));
        Assert.Equal(0, destination.Pixel(9, 1));
    }

    [Fact]
    public void AWindowPastTheEdgeIsHarmless()
    {
        Bitmap source = new(32, 4, 1);
        Bitmap destination = new(32, 4, 1);

        BitmapOps.CopyWindow(source, destination, (1000, 1000));
        BitmapOps.CopyWindow(source, destination, (20, 3));
    }

    [Fact]
    public void RectanglesAreCutToTheBitmap()
    {
        Bitmap bitmap = new(16, 4, 1);

        Drawing.RectFill(bitmap, PenOf(1), (-5, -5), (100, 1), 1);

        Assert.Equal(1, bitmap.Pixel(0, 0));
        Assert.Equal(1, bitmap.Pixel(15, 1));
        Assert.Equal(0, bitmap.Pixel(0, 2));
    }

    [Fact]
    public void ALineMovesThePenIncludingBothEnds()
    {
        Bitmap bitmap = new(16, 8, 1);
        Pen pen = PenOf(1);
        pen.X = 1;
        pen.Y = 1;

        Drawing.LineTo(bitmap, ref pen, (5, 3), 1);

        Assert.Equal((5, 3), (pen.X, pen.Y));
        Assert.Equal(1, bitmap.Pixel(1, 1));
        Assert.Equal(1, bitmap.Pixel(5, 3));
    }

    [Fact]
    public void PolygonsFillInsideStayingInTheClip()
    {
        Bitmap bitmap = new(32, 16, 1);
        (int, int)[] square = [(2, 2), (10, 2), (10, 10), (2, 10)];

        Drawing.FillPolygon(bitmap, PenOf(1), square, new Rect(0, 0, 6, 16), 1);

        Assert.Equal(1, bitmap.Pixel(3, 5));
        Assert.Equal(0, bitmap.Pixel(8, 5));
    }

    [Fact]
    public void ComplementModeDrawsTwiceToErase()
    {
        Bitmap bitmap = new(16, 4, 1);
        Pen pen = PenOf(1);
        pen.Mode = Drawing.COMPLEMENT;

        Drawing.RectFill(bitmap, pen, (0, 0), (3, 3), 1);
        Assert.Equal(1, bitmap.Pixel(1, 1));

        Drawing.RectFill(bitmap, pen, (0, 0), (3, 3), 1);
        Assert.Equal(0, bitmap.Pixel(1, 1));
    }

    [Fact]
    public void ReadsBigEndianValuesAndPastTheEndGivesNull()
    {
        byte[] data = [0x12, 0x34, 0x56, 0x78];

        Assert.Equal((ushort?)0x1234, Font.Be16(data, 0));
        Assert.Equal((uint?)0x1234_5678, Font.Be32(data, 0));
        Assert.Null(Font.Be16([1, 2, 3], 2));
        Assert.Null(Font.Be32([1, 2, 3], 0));
    }

    [Fact]
    public void ABuiltinLetterIsDrawnAdvancingEight()
    {
        Bitmap bitmap = new(16, 16, 1);
        Pen pen = new() { X = 0, Y = 8 };

        Assert.Equal(8, Font.Builtin.DrawGlyph(bitmap, pen, (byte)'I', 1));
        Assert.Contains(Enumerable.Range(0, 16), y => Enumerable.Range(0, 8).Any(x => bitmap.Pixel(x, y) == 1));
    }

    [Fact]
    public void ExpandsTwelveBitColourToRgb()
    {
        Assert.Equal(0x000000u, Scene.Rgb4(0x0000));
        Assert.Equal(0xFFFFFFu, Scene.Rgb4(0x0FFF));
        Assert.Equal(0xFF0088u, Scene.Rgb4(0x0F08));
    }

    [Fact]
    public void PutPixelIgnoresPointsOutsideTheFrame()
    {
        uint[] frame = new uint[Scene.FRAME_WIDTH * Scene.FRAME_HEIGHT];

        Scene.PutPixel(frame, -1, 0, 7);
        Scene.PutPixel(frame, Scene.FRAME_WIDTH, 0, 7);
        Scene.PutPixel(frame, 3, 2, 9);

        Assert.Equal(1, frame.Count(pixel => pixel != 0));
        Assert.Equal(9u, frame[2 * Scene.FRAME_WIDTH + 3]);
    }

    [Fact]
    public void ALoresViewPortDoublesItsPixels()
    {
        Bitmap bitmap = new(16, 2, 1);
        bitmap.Plot(0, 0, 1, false, 1);
        Scene scene = new() { ViewPorts = [new ViewPort { Bitmap = bitmap, Position = (1, 0), Size = (16, 2), Colors = [0x000, 0xF00] }] };
        uint[] frame = new uint[Scene.FRAME_WIDTH * Scene.FRAME_HEIGHT];

        scene.Compose(frame);

        Assert.Equal(0xFF0000u, frame[2]);
        Assert.Equal(0xFF0000u, frame[3]);
        Assert.Equal(0u, frame[4]);
    }

    [Fact]
    public void ColourNumbersPastTheTableAreBlack()
    {
        Bitmap bitmap = new(16, 1, 2);
        bitmap.Plot(0, 0, 3, false, 2);
        Scene scene = new() { ViewPorts = [new ViewPort { Bitmap = bitmap, Size = (1, 1), Hires = true, Colors = [0x000, 0xFFF] }] };
        uint[] frame = Enumerable.Repeat(5u, Scene.FRAME_WIDTH * Scene.FRAME_HEIGHT).ToArray();

        scene.Compose(frame);

        Assert.Equal(0u, frame[0]);
    }

    private const int COLUMNS = Overlay.WIDTH;
    private const int ROWS = 400;

    [Fact]
    public void AShadeDarkensOnlyItsOwnArea()
    {
        uint[] frame = Enumerable.Repeat(0xff80_8080u, COLUMNS * ROWS).ToArray();
        Overlay overlay = new() { Shades = { new Shade(10, 10, 10, 10, 255) } };

        overlay.Draw(frame, COLUMNS);

        Assert.Equal(0xff00_0000u, frame[10 * COLUMNS + 10]);
        Assert.Equal(0xff80_8080u, frame[9 * COLUMNS + 10]);
        Assert.Equal(0xff80_8080u, frame[10 * COLUMNS + 9]);
        Assert.Equal(0xff80_8080u, frame[10 * COLUMNS + 20]);
    }

    [Fact]
    public void HalfDarknessHalvesTheColour()
    {
        Assert.Equal(127u, Overlay.Darken(0xffff_ffff, 128) & 0xff);
        Assert.Equal(0xffff_ffffu, Overlay.Darken(0xffff_ffff, 0));
    }

    [Fact]
    public void TextLightsPixelsInItsColourAndDoubleScaleIsFourTimesAsBig()
    {
        int Count(int scale)
        {
            uint[] frame = new uint[COLUMNS * ROWS];

            new Overlay { Texts = { new Text("A", 100, 100, 0xffff_0000, scale) } }.Draw(frame, COLUMNS);

            return frame.Count(pixel => pixel != 0);
        }

        Assert.Equal(Count(1) * 4, Count(2));
    }

    [Fact]
    public void TheCornerHintLandsInTheBottomRight()
    {
        uint[] frame = Enumerable.Repeat(0xff80_8080u, COLUMNS * ROWS).ToArray();

        Overlay.CornerHint("Press ESC to skip").Draw(frame, COLUMNS);

        Assert.All(frame[..(COLUMNS * (ROWS / 2))], pixel => Assert.Equal(0xff80_8080u, pixel));
        Assert.Contains(Overlay.WHITE, frame);
    }

    [Fact]
    public void DrawingOffTheFrameOrOnAnEmptyOneIsHarmless()
    {
        Overlay overlay = Overlay.CornerHint("Press ESC to skip");

        overlay.Draw([], COLUMNS);
        overlay.Draw(new uint[4], 0);
        overlay.Draw(new uint[100], 10);
    }

    [Fact]
    public void AnImageCoversWhereOpaqueAndBlendsWhereNot()
    {
        uint[] frame = Enumerable.Repeat(0xff00_00ffu, COLUMNS * ROWS).ToArray();
        Overlay overlay = new() { Images = { new Image(5, 5, 2, 1, [0xffff_0000u, 0x80ff_0000u]) } };

        overlay.Draw(frame, COLUMNS);

        Assert.Equal(0xffff_0000u, frame[5 * COLUMNS + 5]);
        uint blended = frame[5 * COLUMNS + 6];
        Assert.True(((blended >> 16) & 0xff) > 100 && (blended & 0xff) > 100);
        Assert.Equal(0xff00_00ffu, frame[5 * COLUMNS + 7]);
    }
}
