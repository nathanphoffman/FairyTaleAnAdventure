namespace FaeryTale.Gfx;

/// Copying and scrolling parts of bitmaps.
public static class BitmapOps
{
    private const int PIXELS_PER_BYTE = 8;

    /// Copies rect of plane from of src to plane to of dst, putting its top left corner at (dstX,
    /// dstY). Parts outside either bitmap are skipped.
    public static void CopyPlaneRect(Bitmap src, int from, Bitmap dst, int to, Rect rect, (int X, int Y) at)
    {
        // Everything is read first so copies that overlap within one bitmap work.
        bool[] bits = ReadRect(src, from, rect);

        WriteRect(dst, to, rect, at, bits);
    }

    /// Copies rect from src to dst plane by plane for the planes mask selects (bit p for plane p)
    /// that both bitmaps have. Returns how many planes were copied.
    public static int CopyRect(Bitmap src, Bitmap dst, Rect rect, (int X, int Y) to, uint mask)
    {
        int copied = 0;

        for (int plane = 0; plane < Math.Min(src.Depth, dst.Depth); plane++)
        {
            if (((mask >> plane) & 1) != 0)
            {
                CopyPlaneRect(src, plane, dst, plane, rect, to);
                copied++;
            }
        }

        return copied;
    }

    /// Copies the window of src whose top left pixel is (x, y) over the whole of dst, plane by
    /// plane. It works on whole bytes with a shift for the odd pixels, so it is quick enough to pan
    /// a picture every frame. Parts of the window outside src are left as they are in dst.
    public static void CopyWindow(Bitmap src, Bitmap dst, (int X, int Y) origin)
    {
        int firstByte = origin.X / PIXELS_PER_BYTE;
        int shift = origin.X % PIXELS_PER_BYTE;

        for (int plane = 0; plane < Math.Min(src.Depth, dst.Depth); plane++)
        {
            int rows = Math.Min(dst.Rows, Math.Max(src.Rows - origin.Y, 0));

            for (int row = 0; row < rows; row++)
            {
                CopyWindowRow(src, dst, plane, origin.Y + row, row, firstByte, shift);
            }
        }
    }

    /// Scrolls the pixels of area (on the first planes planes) by (dx, dy): a pixel moves up and
    /// left by that much. The strip that scrolls in at the edge is filled with colour fill.
    public static void Scroll(Bitmap bitmap, Rect area, (int X, int Y) by, byte fill, int planeCount)
    {
        for (int plane = 0; plane < Math.Min(bitmap.Depth, planeCount); plane++)
        {
            bool[] bits = ReadRect(bitmap, plane, area);

            for (int y = 0; y < area.Height; y++)
            {
                for (int x = 0; x < area.Width; x++)
                {
                    int sourceX = x + by.X;
                    int sourceY = y + by.Y;
                    bool inside = sourceX >= 0 && sourceX < area.Width && sourceY >= 0 && sourceY < area.Height;

                    // Outside the shape, use bit p of the fill colour.
                    bool on = inside ? bits[sourceY * area.Width + sourceX] : ((fill >> plane) & 1) != 0;

                    if (bitmap.Contains(area.X + x, area.Y + y))
                    {
                        bitmap.SetBit(plane, area.X + x, area.Y + y, on);
                    }
                }
            }
        }
    }

    // One row of CopyWindow: the pixels left of the window's edge fall off the top, the next byte fills in.
    private static void CopyWindowRow(Bitmap src, Bitmap dst, int plane, int sourceRow, int destRow, int firstByte, int shift)
    {
        ReadOnlySpan<byte> from = src.Plane(plane).Slice(sourceRow * src.BytesPerRow, src.BytesPerRow);
        Span<byte> to = dst.Plane(plane).Slice(destRow * dst.BytesPerRow, dst.BytesPerRow);

        for (int index = 0; index < to.Length; index++)
        {
            if (firstByte + index >= from.Length)
            {
                break;
            }

            byte high = from[firstByte + index];
            byte low = firstByte + index + 1 < from.Length ? from[firstByte + index + 1] : (byte)0;

            to[index] = shift == 0 ? high : (byte)((high << shift) | (low >> (PIXELS_PER_BYTE - shift)));
        }
    }

    // The bits of rect in plane p of src, row after row (false outside the bitmap).
    private static bool[] ReadRect(Bitmap src, int plane, Rect rect)
    {
        bool[] bits = new bool[Math.Max(rect.Width, 0) * Math.Max(rect.Height, 0)];

        for (int y = 0; y < rect.Height; y++)
        {
            for (int x = 0; x < rect.Width; x++)
            {
                int px = rect.X + x;
                int py = rect.Y + y;

                bits[y * rect.Width + x] = src.Contains(px, py) && src.Bit(plane, px, py);
            }
        }

        return bits;
    }

    // Writes one plane of a rectangle, skipping pixels that fall outside the bitmap.
    private static void WriteRect(Bitmap dst, int plane, Rect rect, (int X, int Y) at, bool[] bits)
    {
        for (int y = 0; y < rect.Height; y++)
        {
            for (int x = 0; x < rect.Width; x++)
            {
                int px = at.X + x;
                int py = at.Y + y;

                if (dst.Contains(px, py))
                {
                    dst.SetBit(plane, px, py, bits[y * rect.Width + x]);
                }
            }
        }
    }
}
