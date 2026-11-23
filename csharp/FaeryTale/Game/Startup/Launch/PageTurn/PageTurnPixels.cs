
namespace FaeryTale.Game.Startup;

/// PageTurn: averaging and scaling pixels.
internal sealed partial class PageTurn
{
    // The mean colour of a row of a page from one column to another.
    private static uint Average(Flap flap, int row)
    {
        uint red = 0;
        uint green = 0;
        uint blue = 0;

        for (int column = flap.FirstColumn; column <= flap.LastColumn; column++)
        {
            uint color = flap.Source[row * PAGE_WIDTH + column];

            red += (color >> 16) & CHANNEL_MASK;
            green += (color >> 8) & CHANNEL_MASK;
            blue += color & CHANNEL_MASK;
        }

        uint count = (uint)(flap.LastColumn - flap.FirstColumn + 1);

        return OPAQUE | (red / count << 16) | (green / count << 8) | (blue / count);
    }

    // A colour with each channel multiplied by a factor from 0 to 1.
    private static uint Scale(uint color, double factor)
    {
        uint Part(int shift) => (uint)(((color >> shift) & CHANNEL_MASK) * factor) << shift;

        return OPAQUE | Part(16) | Part(8) | Part(0);
    }
}
