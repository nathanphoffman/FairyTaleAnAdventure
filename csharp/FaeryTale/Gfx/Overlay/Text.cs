namespace FaeryTale.Gfx;

/// A line of text with its left end and baseline at (X, Y), an 0xAARRGGBB colour and a scale:
/// each pixel of the 8x8 font is Scale x Scale overlay pixels.
public sealed record Text(string Content, int X, int Y, uint Color, int Scale);
