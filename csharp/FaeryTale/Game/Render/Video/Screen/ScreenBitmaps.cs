using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen: the bitmaps and copying between them.
public sealed partial class Screen
{
    public Bitmap GetBitmap(BitmapId id) => bitmaps[IndexOf(id)];

    public void SetBitmap(BitmapId id, Bitmap bitmap) => bitmaps[IndexOf(id)] = bitmap;

    /// What the sprite blitter works on: the bitmap being drawn on and its two work regions.
    public (Bitmap Bitmap, byte[] Queue, byte[] Mask) BlitterMemory() => (bitmaps[IndexOf(DrawTarget)], QueueRegion, MaskRegion);

    public Bitmap DrawBitmap => GetBitmap(DrawTarget);

    /// Two different bitmaps, the first to read and the second to write.
    public (Bitmap From, Bitmap To) TwoBitmaps(BitmapId from, BitmapId to)
    {
        if (IndexOf(from) == IndexOf(to))
        {
            throw new InvalidOperationException("a copy needs two different bitmaps");
        }

        return (GetBitmap(from), GetBitmap(to));
    }

    // Splits a port from the bitmap it draws on and the planes drawing touches.
    private (Bitmap Bitmap, int Planes)? TargetOf(PortId id) =>
        GetPort(id).Target is { } target ? (bitmaps[IndexOf(target)], PlanesOf(target)) : null;

    // The position of an id in the list of bitmaps.
    private static int IndexOf(BitmapId id) => id switch
    {
        BitmapId.Page1 => 0,
        BitmapId.Page2 => 1,
        BitmapId.Text or BitmapId.TextPlane0 => 2,
        BitmapId.PageA => 3,
        BitmapId.PageB => 4,
        _ => 5,
    };
}
