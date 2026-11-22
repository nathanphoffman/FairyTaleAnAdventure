using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen: moving, text, lines and rectangles.
public sealed partial class Screen
{
    public void MoveTo(PortId id, int x, int y)
    {
        Port port = GetPort(id);

        port.Pen.X = x;
        port.Pen.Y = y;
    }

    /// Writes the bytes at the pen of the port.
    public void Text(PortId id, ReadOnlySpan<byte> bytes)
    {
        if (TargetOf(id) is { } target)
        {
            Port port = GetPort(id);

            WriteText(target.Bitmap, port, bytes, target.Planes);
        }
    }

    /// How far the pen of a port moves across after a character, in the port's font.
    public int AdvanceOf(PortId id, byte character) => fonts[(int)GetPort(id).Font].AdvanceOf(character);

    /// Draws a line from the port's pen to (x, y).
    public void LineTo(PortId id, int x, int y)
    {
        if (TargetOf(id) is { } target)
        {
            Gfx.Drawing.LineTo(target.Bitmap, ref GetPort(id).Pen, (x, y), target.Planes);
        }
    }

    public void RectFill(PortId id, (int X, int Y) from, (int X, int Y) to)
    {
        if (TargetOf(id) is { } target)
        {
            Gfx.Drawing.RectFill(target.Bitmap, GetPort(id).Pen, from, to, target.Planes);
        }
    }

    // Writes text with the port's pen and font on the bitmap.
    private void WriteText(Bitmap bitmap, Port port, ReadOnlySpan<byte> bytes, int planes) =>
        Gfx.Drawing.Text(bitmap, ref port.Pen, fonts[(int)port.Font], bytes, planes);
}
