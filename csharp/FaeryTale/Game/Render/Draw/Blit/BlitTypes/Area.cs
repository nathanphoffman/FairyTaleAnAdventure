using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// An area of memory a blit reads and writes: a bit plane of the page being drawn (number
/// PlaneNumber), the work regions of the blitter, or the sprite sheets.
public readonly record struct Area(AreaKind Kind, byte PlaneNumber)
{
    public static readonly Area Queue = new(AreaKind.Queue, 0);
    public static readonly Area Mask = new(AreaKind.Mask, 0);
    public static readonly Area Shapes = new(AreaKind.Shapes, 0);

    public static Area Plane(int number) => new(AreaKind.Plane, (byte)number);
}
