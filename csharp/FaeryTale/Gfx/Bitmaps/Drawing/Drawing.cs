namespace FaeryTale.Gfx;

/// Drawing into a bitmap with a pen: filled rectangles, lines, filled polygons and text.
public static class Drawing
{
    // How drawing combines with what is there.
    public const byte JAM1 = 0;
    public const byte JAM2 = 1;
    public const byte COMPLEMENT = 2;

    // A polygon needs at least three corners to enclose anything.
    private const int MIN_POLYGON_POINTS = 3;

    /// Fills the rectangle (corners inclusive) with the pen colour, or flips it in COMPLEMENT
    /// mode. It is cut to the bitmap.
    public static void RectFill(Bitmap bitmap, Pen drawingPen, (int X, int Y) from, (int X, int Y) to, int planes)
    {
        for (int positionY = Math.Max(from.Y, 0); positionY <= Math.Min(to.Y, bitmap.Rows - 1); positionY++)
        {
            for (int positionX = Math.Max(from.X, 0); positionX <= to.X; positionX++)
            {
                bitmap.Plot(positionX, positionY, drawingPen.Fg, drawingPen.Complement, planes);
            }
        }
    }

    /// Draws a line from the pen to (x, y) and moves the pen there.
    public static void LineTo(Bitmap bitmap, ref Pen drawingPen, (int X, int Y) to, int planes)
    {
        int cx = drawingPen.X;
        int cy = drawingPen.Y;
        int deltaX = Math.Abs(to.X - cx);
        int deltaY = -Math.Abs(to.Y - cy);
        int sx = cx < to.X ? 1 : -1;
        int sy = cy < to.Y ? 1 : -1;
        int error = deltaX + deltaY;

        while (true)
        {
            bitmap.Plot(cx, cy, drawingPen.Fg, drawingPen.Complement, planes);

            if (cx == to.X && cy == to.Y)
            {
                break;
            }

            int doubled = 2 * error;

            if (doubled >= deltaY)
            {
                error += deltaY;
                cx += sx;
            }

            if (doubled <= deltaX)
            {
                error += deltaX;
                cy += sy;
            }
        }

        drawingPen.X = to.X;
        drawingPen.Y = to.Y;
    }

    /// Fills a polygon (even-odd rule) with the pen colour, or flips it in COMPLEMENT mode. Only
    /// pixels inside clip (corners inclusive) are touched.
    public static void FillPolygon(Bitmap bitmap, Pen drawingPen, IReadOnlyList<(int X, int Y)> points, Rect clip, int planes)
    {
        if (points.Count < MIN_POLYGON_POINTS)
        {
            return;
        }

        int clipRight = clip.X + clip.Width - 1;
        int clipBottom = clip.Y + clip.Height - 1;
        int topEdge = Math.Max(points.Min(point => point.Y), clip.Y);
        int bottom = Math.Min(points.Max(point => point.Y), clipBottom);

        for (int positionY = topEdge; positionY <= bottom; positionY++)
        {
            List<int> xs = Crossings(points, positionY);

            for (int pair = 0; pair + 1 < xs.Count; pair += 2)
            {
                for (int positionX = Math.Max(xs[pair], clip.X); positionX <= Math.Min(xs[pair + 1], clipRight); positionX++)
                {
                    bitmap.Plot(positionX, positionY, drawingPen.Fg, drawingPen.Complement, planes);
                }
            }
        }
    }

    /// Draws the bytes of text at the pen (which sits on the baseline) and moves the pen along.
    public static void Text(Bitmap bitmap, ref Pen drawingPen, Font font, ReadOnlySpan<byte> text, int planes)
    {
        foreach (byte character in text)
        {
            drawingPen.X += font.DrawGlyph(bitmap, drawingPen, character, planes);
        }
    }

    // The x positions, sorted, where the edges of the polygon cross the pixel row y.
    private static List<int> Crossings(IReadOnlyList<(int X, int Y)> points, int positionY)
    {
        List<int> xs = [];

        for (int index = 0; index < points.Count; index++)
        {
            (int X, int Y) a = points[index];
            (int X, int Y) b = points[(index + 1) % points.Count];

            if ((a.Y <= positionY && b.Y > positionY) || (b.Y <= positionY && a.Y > positionY))
            {
                xs.Add(a.X + (positionY - a.Y) * (b.X - a.X) / (b.Y - a.Y));
            }
        }

        xs.Sort();

        return xs;
    }
}
