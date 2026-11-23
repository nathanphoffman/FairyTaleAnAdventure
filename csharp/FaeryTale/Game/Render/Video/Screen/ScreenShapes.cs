using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen: polygons, clearing, scrolling and copying.
public sealed partial class Screen
{
    /// The bitmap of display page n (0 or 1) as it is wired up now.
    public BitmapId PageBitmap(int n) => Ras[Pages[n].Ras].Bitmap;

    /// The page being drawn on.
    public FPage DrawingPage => Pages[Drawing];

    /// The bitmap of the page being drawn on / being shown.
    public BitmapId DrawingBitmapId => PageBitmap(Drawing);

    public BitmapId ViewingBitmapId => PageBitmap(Viewing);

    public void FillPolygon(PortId identifier, IReadOnlyList<(int X, int Y)> points, Rect clip)
    {
        if (TargetOf(identifier) is { } target)
        {
            Gfx.Drawing.FillPolygon(target.Bitmap, GetPort(identifier).Pen, points, clip, target.Planes);
        }
    }

    /// Fills the whole bitmap of the port with colour pen.
    public void Clear(PortId identifier, byte penColor)
    {
        if (TargetOf(identifier) is { } target)
        {
            target.Bitmap.Clear(penColor, target.Planes);
        }
    }

    /// Scrolls an area of the port's bitmap; what scrolls in gets the port's background colour.
    public void Scroll(PortId identifier, Rect area, (int X, int Y) by)
    {
        if (TargetOf(identifier) is { } target)
        {
            BitmapOps.Scroll(target.Bitmap, area, by, GetPort(identifier).Pen.Bg, target.Planes);
        }
    }

    /// Copies a rectangle of one bitmap to another (not the same one), for the planes in mask.
    public void Copy(BitmapId from, Rect rect, BitmapId to, (int X, int Y) offset, uint mask)
    {
        (Bitmap source, Bitmap destination) = TwoBitmaps(from, to);

        BitmapOps.CopyRect(source, destination, rect, offset, mask);
    }

    /// The compass picture which (0 normal, 1 lit) copied into plane plane of the text panel.
    public void CopyCompass(int which, Rect rect, (int X, int Y) offset, int plane) =>
        BitmapOps.CopyPlaneRect(Compass[which], 0, GetBitmap(BitmapId.Text), plane, rect, offset);
}
