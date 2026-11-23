namespace FaeryTale.Gfx;

/// A rectangle of a bitmap shown on screen with its own colours.
public sealed class ViewPort
{
    public required Bitmap Bitmap { get; init; }

    /// The part of the bitmap shown starts here.
    public (int X, int Y) Scroll { get; init; }

    /// Where on screen it goes (in its own pixels) and how big it is.
    public (int X, int Y) Position { get; init; }

    public (int Width, int Height) Size { get; init; }

    public bool Hires { get; init; }

    /// The colour table: 12 bit colours (0x0RGB).
    public required ushort[] Colors { get; init; }

    /// The mouse pointer is drawn with the colours of this view port.
    public bool ShowsPointer { get; init; }
}
