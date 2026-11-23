using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen: moving, text, lines and rectangles.
public sealed partial class Screen
{
    public void MoveTo(PortId identifier, int positionX, int positionY)
    {
        Port port = GetPort(identifier);

        port.Pen.X = positionX;
        port.Pen.Y = positionY;
    }

    /// Writes the bytes at the pen of the port.
    public void Text(PortId identifier, ReadOnlySpan<byte> bytes)
    {
        if (TargetOf(identifier) is { } target)
        {
            Port port = GetPort(identifier);

            WriteText(target.Bitmap, port, bytes, target.Planes);
        }
    }

    /// How far the pen of a port moves across after a character, in the port's font.
    public int AdvanceOf(PortId identifier, byte character) => fonts[(int)GetPort(identifier).Font].AdvanceOf(character);

    /// Draws a line from the port's pen to (x, y).
    public void LineTo(PortId identifier, int positionX, int positionY)
    {
        if (TargetOf(identifier) is { } target)
        {
            Gfx.Drawing.LineTo(target.Bitmap, ref GetPort(identifier).Pen, (positionX, positionY), target.Planes);
        }
    }

    public void RectFill(PortId identifier, (int X, int Y) from, (int X, int Y) to)
    {
        if (TargetOf(identifier) is { } target)
        {
            Gfx.Drawing.RectFill(target.Bitmap, GetPort(identifier).Pen, from, to, target.Planes);
        }
    }

    // Writes text with the port's pen and font on the bitmap.
    private void WriteText(Bitmap bitmap, Port port, ReadOnlySpan<byte> bytes, int planes) =>
        Gfx.Drawing.Text(bitmap, ref port.Pen, fonts[(int)port.Font], bytes, planes);
}
