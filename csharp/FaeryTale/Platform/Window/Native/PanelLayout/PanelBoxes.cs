
namespace FaeryTale.Platform;

/// PanelLayout: finding the filled boxes of the panel picture.
internal sealed partial class PanelLayout
{
    // Finds the black that is joined to the edge of the panel: that is the background. Menu boxes that
    // have something in them are not touched (their black labels must stay).
    private void FindBackground(uint[] frame, int frameWidth, int firstLine)
    {
        KeepFilledBoxes(frame, frameWidth, firstLine);
        waiting.Clear();

        for (int positionX = 0; positionX < width; positionX++)
        {
            Seed(frame, frameWidth, firstLine, positionX, 0);
            Seed(frame, frameWidth, firstLine, positionX, lines - 1);
        }

        for (int positionY = 0; positionY < lines; positionY++)
        {
            Seed(frame, frameWidth, firstLine, 0, positionY);
            Seed(frame, frameWidth, firstLine, width - 1, positionY);
        }

        Spread(frame, frameWidth, firstLine);
    }

    // Marks the menu boxes that have something in them (not all black) as not to be touched.
    private void KeepFilledBoxes(uint[] frame, int frameWidth, int firstLine)
    {
        for (int column = 0; column < BOX_COLUMNS; column++)
        {
            for (int rowIndex = 0; rowIndex < BOX_ROWS; rowIndex++)
            {
                (int left, int top) = (BOX_LEFT + column * BOX_COLUMN_SPACING, FIRST_BOX_LINE + rowIndex * BOX_ROW_SPACING);

                if (HasColor(frame, frameWidth, firstLine, left, top))
                {
                    MarkKept(left, top);
                    lastBoxRow = Math.Max(lastBoxRow, rowIndex);
                }
            }
        }
    }

    // Whether a menu box has any colour in it.
    private static bool HasColor(uint[] frame, int frameWidth, int firstLine, int left, int topEdge)
    {
        for (int positionY = topEdge; positionY < topEdge + BOX_HEIGHT; positionY++)
        {
            for (int positionX = left; positionX < left + BOX_WIDTH; positionX++)
            {
                if ((frame[(firstLine + positionY) * frameWidth + positionX] & COLOR_MASK) != 0)
                {
                    return true;
                }
            }
        }

        return false;
    }

    private void MarkKept(int left, int topEdge)
    {
        for (int positionY = topEdge; positionY < topEdge + BOX_HEIGHT; positionY++)
        {
            Array.Fill(kept, true, positionY * width + left, BOX_WIDTH);
        }
    }

    // Starts the spread at a pixel when it is black background not yet found and not in a kept box.
    private void Seed(uint[] frame, int frameWidth, int firstLine, int positionX, int positionY)
    {
        int index = positionY * width + positionX;

        if (!background[index] && !kept[index] && (frame[(firstLine + positionY) * frameWidth + positionX] & COLOR_MASK) == 0)
        {
            background[index] = true;
            waiting.Enqueue(index);
        }
    }

    // Seeds at a pixel unless it is outside the panel.
    private void SeedIfInside(uint[] frame, int frameWidth, int firstLine, int positionX, int positionY)
    {
        if (positionX >= 0 && positionX < width && positionY >= 0 && positionY < lines)
        {
            Seed(frame, frameWidth, firstLine, positionX, positionY);
        }
    }
}
