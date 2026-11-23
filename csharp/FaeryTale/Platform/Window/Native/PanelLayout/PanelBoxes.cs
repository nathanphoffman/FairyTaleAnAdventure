
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

        for (int x = 0; x < width; x++)
        {
            Seed(frame, frameWidth, firstLine, x, 0);
            Seed(frame, frameWidth, firstLine, x, lines - 1);
        }

        for (int y = 0; y < lines; y++)
        {
            Seed(frame, frameWidth, firstLine, 0, y);
            Seed(frame, frameWidth, firstLine, width - 1, y);
        }

        Spread(frame, frameWidth, firstLine);
    }

    // Marks the menu boxes that have something in them (not all black) as not to be touched.
    private void KeepFilledBoxes(uint[] frame, int frameWidth, int firstLine)
    {
        for (int column = 0; column < BOX_COLUMNS; column++)
        {
            for (int row = 0; row < BOX_ROWS; row++)
            {
                (int left, int top) = (BOX_LEFT + column * BOX_COLUMN_SPACING, FIRST_BOX_LINE + row * BOX_ROW_SPACING);

                if (HasColor(frame, frameWidth, firstLine, left, top))
                {
                    MarkKept(left, top);
                    lastBoxRow = Math.Max(lastBoxRow, row);
                }
            }
        }
    }

    // Whether a menu box has any colour in it.
    private static bool HasColor(uint[] frame, int frameWidth, int firstLine, int left, int top)
    {
        for (int y = top; y < top + BOX_HEIGHT; y++)
        {
            for (int x = left; x < left + BOX_WIDTH; x++)
            {
                if ((frame[(firstLine + y) * frameWidth + x] & COLOR_MASK) != 0)
                {
                    return true;
                }
            }
        }

        return false;
    }

    private void MarkKept(int left, int top)
    {
        for (int y = top; y < top + BOX_HEIGHT; y++)
        {
            Array.Fill(kept, true, y * width + left, BOX_WIDTH);
        }
    }

    // Starts the spread at a pixel when it is black background not yet found and not in a kept box.
    private void Seed(uint[] frame, int frameWidth, int firstLine, int x, int y)
    {
        int index = y * width + x;

        if (!background[index] && !kept[index] && (frame[(firstLine + y) * frameWidth + x] & COLOR_MASK) == 0)
        {
            background[index] = true;
            waiting.Enqueue(index);
        }
    }

    // Seeds at a pixel unless it is outside the panel.
    private void SeedIfInside(uint[] frame, int frameWidth, int firstLine, int x, int y)
    {
        if (x >= 0 && x < width && y >= 0 && y < lines)
        {
            Seed(frame, frameWidth, firstLine, x, y);
        }
    }
}
