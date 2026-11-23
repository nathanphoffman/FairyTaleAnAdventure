using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// A rectangle of the screen that shows a bitmap with its own colours.
public sealed class ViewPortState
{
    /// The index into Screen.Ras of what it shows, once it shows anything.
    public int? Ras;
    public short Width;
    public short Height;
    public short X;
    public short Y;
    public ushort Modes;
    public ushort[] Colors = [];
}
