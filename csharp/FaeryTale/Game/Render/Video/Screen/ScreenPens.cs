using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen: ports, fonts and pens.
public sealed partial class Screen
{
    public Port GetPort(PortId id) => ports[(int)id];

    public void SetFontKind(PortId id, FontKind kind) => GetPort(id).Font = kind;

    public void SetFont(FontKind kind, Font font) => fonts[(int)kind] = font;

    public void SetAPen(PortId id, byte pen) => GetPort(id).Pen.Fg = pen;

    public void SetBPen(PortId id, byte pen) => GetPort(id).Pen.Bg = pen;

    public void SetDrawMode(PortId id, byte mode) => GetPort(id).Pen.Mode = mode;
}
