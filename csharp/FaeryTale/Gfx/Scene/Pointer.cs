namespace FaeryTale.Gfx;

/// The mouse pointer: a 16 pixel wide sprite. Each line is two 16 bit planes packed in one word;
/// the first word is a control word.
public sealed record Pointer(uint[] Image, (int X, int Y) Position);
