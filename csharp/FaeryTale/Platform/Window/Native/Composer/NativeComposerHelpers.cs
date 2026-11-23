using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// NativeComposer: small helpers.
internal sealed partial class NativeComposer
{
    private static uint[] Filled(int count, uint color)
    {
        uint[] pixels = new uint[count];

        Array.Fill(pixels, color);

        return pixels;
    }

    private static int FloorDivide(int value, int by) => (int)Math.Floor((double)value / by);
}
