
namespace FaeryTale.Game.Startup;

/// PageTurn: averaging and scaling pixels.
internal sealed partial class PageTurn
{
    // The mean colour of a row of a page from one column to another.
    private static uint Average(Flap flap, int rowIndex)
    {
        uint red = 0;
        uint green = 0;
        uint blue = 0;

        for (int column = flap.FirstColumn; column <= flap.LastColumn; column++)
        {
            uint color = flap.Source[rowIndex * PAGE_WIDTH + column];

            red += (color >> RED_SHIFT) & CHANNEL_MASK; // unpack the red byte from 0xAARRGGBB
            green += (color >> GREEN_SHIFT) & CHANNEL_MASK; // unpack the green byte
            blue += color & CHANNEL_MASK;
        }

        uint count = (uint)(flap.LastColumn - flap.FirstColumn + 1);

        return OPAQUE | (red / count << RED_SHIFT) | (green / count << GREEN_SHIFT) | (blue / count); // average each channel, then shift it back into its byte and OR together
    }

    // A colour with each channel multiplied by a factor from 0 to 1.
    private static uint Scale(uint color, double factor)
    {
        uint Part(int shift) => (uint)(((color >> shift) & CHANNEL_MASK) * factor) << shift; // pull one channel out, scale it, and shift it back into place

        return OPAQUE | Part(RED_SHIFT) | Part(GREEN_SHIFT) | Part(0);
    }
}
