using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen: ports, fonts and pens.
public sealed partial class Screen
{
    public Port GetPort(PortId identifier) => ports[(int)identifier];

    public void SetFontKind(PortId identifier, FontKind kind) => GetPort(identifier).Font = kind;

    public void SetFont(FontKind kind, Font font) => fonts[(int)kind] = font;

    public void SetAPen(PortId identifier, byte penColor) => GetPort(identifier).Pen.Fg = penColor;

    public void SetBPen(PortId identifier, byte penColor) => GetPort(identifier).Pen.Bg = penColor;

    public void SetDrawMode(PortId identifier, byte mode) => GetPort(identifier).Pen.Mode = mode;
}
