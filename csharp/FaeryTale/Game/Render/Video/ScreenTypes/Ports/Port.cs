using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// A pen, the bitmap it draws on and the font it writes in.
public sealed class Port
{
    public Pen Pen = new();
    public BitmapId? Target;
    public FontKind Font = FontKind.Panel;
}
