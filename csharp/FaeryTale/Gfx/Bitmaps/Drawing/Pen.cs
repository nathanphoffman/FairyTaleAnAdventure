namespace FaeryTale.Gfx;

/// The pen: its two colours, its drawing mode and where it is.
public struct Pen
{
    /// Colour of what is drawn.
    public byte Fg;

    /// Colour of the background of text (in JAM2 mode).
    public byte Bg;

    public byte Mode;
    public int X;
    public int Y;

    public Pen()
    {
        Fg = 1;
        Bg = 0;
        Mode = Drawing.JAM2;
    }

    internal readonly bool Complement => (Mode & Drawing.COMPLEMENT) != 0;
}
