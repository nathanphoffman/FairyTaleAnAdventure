namespace FaeryTale.Game.Render;

/// The part of a sprite that is on the screen.
internal sealed class Clip
{
    public short XStart;
    public short YStart;
    public short XStop;
    public short YStop;

    /// How many pixels of the left edge, and lines of the top, are cut off.
    public sbyte XOff;
    public sbyte YOff;
}
