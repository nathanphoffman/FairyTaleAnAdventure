
namespace FaeryTale.Platform;

/// PanelLayout: spreading, copying and shading the pieces.
internal sealed partial class PanelLayout
{
    // Spreads from the found pixels to black pixels above, below and beside them.
    private void Spread(uint[] frame, int frameWidth, int firstLine)
    {
        while (waiting.TryDequeue(out int index))
        {
            int positionX = index % width;
            int positionY = index / width;

            SeedIfInside(frame, frameWidth, firstLine, positionX - 1, positionY);
            SeedIfInside(frame, frameWidth, firstLine, positionX + 1, positionY);
            SeedIfInside(frame, frameWidth, firstLine, positionX, positionY - 1);
            SeedIfInside(frame, frameWidth, firstLine, positionX, positionY + 1);
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
        int topEdge = FIRST_BOX_LINE - FRAME_LINES + TOP_PAD_LINES;
        int bottom = FIRST_BOX_LINE + lastBoxRow * BOX_ROW_SPACING + BOX_HEIGHT + FRAME_LINES + TOP_PAD_LINES;

        for (int positionY = topEdge; positionY < Math.Min(bottom, ArrangedLines); positionY++)
        {
            Array.Fill(Arranged, SOLID, positionY * ArrangedWidth + left, right - left);
        }
    }

    // Puts each piece of the classic picture where it goes in the new one. Background pixels are left
    // as they are (0, the wood shows); the others are copied, solid.
    private void CopyPieces(uint[] frame, int frameWidth, int firstLine)
    {
        for (int positionY = 0; positionY < lines; positionY++)
        {
            for (int positionX = 0; positionX < width; positionX++)
            {
                if (!background[positionY * width + positionX])
                {
                    (int newX, int newY) = Moved(positionX, positionY);

                    if (newY >= 0 && newY < ArrangedLines)
                    {
                        Arranged[newY * ArrangedWidth + newX] = SOLID | frame[(firstLine + positionY) * frameWidth + positionX];
                    }
                }
            }
        }
    }

    // A colour with red, green and blue cut down to the panel's wood brightness.
    private static uint Darken(uint color)
    {
        uint Part(int shift) => ((color >> shift) & CHANNEL_MASK) * WOOD_BRIGHTNESS_PERCENT / PERCENT_SCALE << shift;  // pull one 8-bit colour channel out, scale it by the percent, move it back

        return (color & SOLID) | Part(RED_SHIFT) | Part(GREEN_SHIFT) | Part(0);
    }
}
