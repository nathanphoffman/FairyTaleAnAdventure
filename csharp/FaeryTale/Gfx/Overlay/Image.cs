namespace FaeryTale.Gfx;

/// A ready-made picture of 0xAARRGGBB pixels with its top left corner at (X, Y).
public sealed record Image(int X, int Y, int Width, int Height, uint[] Pixels);
