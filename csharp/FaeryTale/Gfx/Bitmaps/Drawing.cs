namespace FaeryTale.Gfx;

/// The pen: its two colours, its drawing mode and where it is.
public struct Pen
{
    /// Colour of what is drawn.
    public byte Fg;

    /// Colour of the background of text (in JAM2 mode).
    public byte Bg;

    public byte Mode;
    public int X;
    public int Y;

    public Pen()
    {
        Fg = 1;
        Bg = 0;
        Mode = Drawing.JAM2;
    }

    internal readonly bool Complement => (Mode & Drawing.COMPLEMENT) != 0;
}

/// Drawing into a bitmap with a pen: filled rectangles, lines, filled polygons and text.
public static class Drawing
{
    // How drawing combines with what is there.
    public const byte JAM1 = 0;
    public const byte JAM2 = 1;
    public const byte COMPLEMENT = 2;

    /// Fills the rectangle (corners inclusive) with the pen colour, or flips it in COMPLEMENT
    /// mode. It is cut to the bitmap.
    public static void RectFill(Bitmap bitmap, Pen pen, (int X, int Y) from, (int X, int Y) to, int planes)
    {
        for (int y = Math.Max(from.Y, 0); y <= Math.Min(to.Y, bitmap.Rows - 1); y++)
        {
            for (int x = Math.Max(from.X, 0); x <= to.X; x++)
            {
                bitmap.Plot(x, y, pen.Fg, pen.Complement, planes);
            }
        }
    }

    /// Draws a line from the pen to (x, y) and moves the pen there.
    public static void LineTo(Bitmap bitmap, ref Pen pen, (int X, int Y) to, int planes)
    {
        int cx = pen.X;
        int cy = pen.Y;
        int dx = Math.Abs(to.X - cx);
        int dy = -Math.Abs(to.Y - cy);
        int sx = cx < to.X ? 1 : -1;
        int sy = cy < to.Y ? 1 : -1;
        int error = dx + dy;

        while (true)
        {
            bitmap.Plot(cx, cy, pen.Fg, pen.Complement, planes);

            if (cx == to.X && cy == to.Y)
            {
                break;
            }

            int doubled = 2 * error;

            if (doubled >= dy)
            {
                error += dy;
                cx += sx;
            }

            if (doubled <= dx)
            {
                error += dx;
                cy += sy;
            }
        }

        pen.X = to.X;
        pen.Y = to.Y;
    }

    /// Fills a polygon (even-odd rule) with the pen colour, or flips it in COMPLEMENT mode. Only
    /// pixels inside clip (corners inclusive) are touched.
    public static void FillPolygon(Bitmap bitmap, Pen pen, IReadOnlyList<(int X, int Y)> points, Rect clip, int planes)
    {
        if (points.Count < 3)
        {
            return;
        }

        int clipRight = clip.X + clip.Width - 1;
        int clipBottom = clip.Y + clip.Height - 1;
        int top = Math.Max(points.Min(point => point.Y), clip.Y);
        int bottom = Math.Min(points.Max(point => point.Y), clipBottom);

        for (int y = top; y <= bottom; y++)
        {
            List<int> xs = Crossings(points, y);

            for (int pair = 0; pair + 1 < xs.Count; pair += 2)
            {
                for (int x = Math.Max(xs[pair], clip.X); x <= Math.Min(xs[pair + 1], clipRight); x++)
                {
                    bitmap.Plot(x, y, pen.Fg, pen.Complement, planes);
                }
            }
        }
    }

    /// Draws the bytes of text at the pen (which sits on the baseline) and moves the pen along.
    public static void Text(Bitmap bitmap, ref Pen pen, Font font, ReadOnlySpan<byte> text, int planes)
    {
        foreach (byte character in text)
        {
            pen.X += font.DrawGlyph(bitmap, pen, character, planes);
        }
    }

    // The x positions, sorted, where the edges of the polygon cross the pixel row y.
    private static List<int> Crossings(IReadOnlyList<(int X, int Y)> points, int y)
    {
        List<int> xs = [];

        for (int index = 0; index < points.Count; index++)
        {
            (int X, int Y) a = points[index];
            (int X, int Y) b = points[(index + 1) % points.Count];

            if ((a.Y <= y && b.Y > y) || (b.Y <= y && a.Y > y))
            {
                xs.Add(a.X + (y - a.Y) * (b.X - a.X) / (b.Y - a.Y));
            }
        }

        xs.Sort();

        return xs;
    }
}
