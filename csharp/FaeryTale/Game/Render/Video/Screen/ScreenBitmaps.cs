using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen: the bitmaps and copying between them.
public sealed partial class Screen
{
    public Bitmap GetBitmap(BitmapId identifier) => bitmaps[IndexOf(identifier)];

    public void SetBitmap(BitmapId identifier, Bitmap bitmap) => bitmaps[IndexOf(identifier)] = bitmap;

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
    private (Bitmap Bitmap, int Planes)? TargetOf(PortId identifier) =>
        GetPort(identifier).Target is { } target ? (bitmaps[IndexOf(target)], PlanesOf(target)) : null;

    // The position of an id in the list of bitmaps.
    private static int IndexOf(BitmapId identifier) => identifier switch
    {
        BitmapId.Page1 => 0,
        BitmapId.Page2 => 1,
        BitmapId.Text or BitmapId.TextPlane0 => 2,
        BitmapId.PageA => PAGE_A_INDEX,
        BitmapId.PageB => PAGE_B_INDEX,
        _ => LAST_BITMAP_INDEX,
    };
}
