namespace FaeryTale.Gfx;

/// A rectangle that is darkened: Darkness 0 leaves the picture alone, 255 makes it black.
public sealed record Shade(int X, int Y, int Width, int Height, byte Darkness);
