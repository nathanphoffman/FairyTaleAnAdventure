
namespace FaeryTale.Platform;

/// PanelLayout: spreading, copying and shading the pieces.
internal sealed partial class PanelLayout
{
    // Spreads from the found pixels to black pixels above, below and beside them.
    private void Spread(uint[] frame, int frameWidth, int firstLine)
    {
        while (waiting.TryDequeue(out int index))
        {
            int x = index % width;
            int y = index / width;

            SeedIfInside(frame, frameWidth, firstLine, x - 1, y);
            SeedIfInside(frame, frameWidth, firstLine, x + 1, y);
            SeedIfInside(frame, frameWidth, firstLine, x, y - 1);
            SeedIfInside(frame, frameWidth, firstLine, x, y + 1);
        }
    }

    // A solid black rectangle behind the menu boxes, a little bigger than they are, so that every box
    // has a thick black frame (and the gaps between them are black too).
    private void DrawMenuFrame()
    {
        if (lastBoxRow < 0)
        {
            return;
        }

        int left = BOX_LEFT - FRAME_PIXELS + MENU_SHIFT;
        int right = BOX_LEFT + BOX_COLUMN_SPACING + BOX_WIDTH + FRAME_PIXELS + MENU_SHIFT;
        int top = FIRST_BOX_LINE - FRAME_LINES + TOP_PAD_LINES;
        int bottom = FIRST_BOX_LINE + lastBoxRow * BOX_ROW_SPACING + BOX_HEIGHT + FRAME_LINES + TOP_PAD_LINES;

        for (int y = top; y < Math.Min(bottom, ArrangedLines); y++)
        {
            Array.Fill(Arranged, SOLID, y * ArrangedWidth + left, right - left);
        }
    }

    // Puts each piece of the classic picture where it goes in the new one. Background pixels are left
    // as they are (0, the wood shows); the others are copied, solid.
    private void CopyPieces(uint[] frame, int frameWidth, int firstLine)
    {
        for (int y = 0; y < lines; y++)
        {
            for (int x = 0; x < width; x++)
            {
                if (!background[y * width + x])
                {
                    (int newX, int newY) = Moved(x, y);

                    if (newY >= 0 && newY < ArrangedLines)
                    {
                        Arranged[newY * ArrangedWidth + newX] = SOLID | frame[(firstLine + y) * frameWidth + x];
                    }
                }
            }
        }
    }

    // A colour with red, green and blue cut down to the panel's wood brightness.
    private static uint Darken(uint color)
    {
        uint Part(int shift) => ((color >> shift) & 0xff) * WOOD_BRIGHTNESS_PERCENT / 100 << shift;

        return (color & 0xff000000) | Part(16) | Part(8) | Part(0);
    }
}
