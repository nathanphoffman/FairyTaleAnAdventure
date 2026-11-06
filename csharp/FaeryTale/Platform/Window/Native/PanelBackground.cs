using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// A wooden board in place of the black behind the text panel (Remastered and Reimagined): behind the
/// beige message box, the menu buttons and the compass. Only black that is joined to the edge of the
/// panel is replaced, and the menu boxes are left as they are (their black labels must stay); a box
/// with nothing in it (an unused menu slot, drawn all black) is background like the rest.
internal sealed class PanelBackground
{
    // The panel's picture has no alpha, so black is whatever has no red, green or blue in it.
    private const uint COLOR_MASK = 0x00ff_ffff;
    private const uint BOARD_SEED = 0x51d3;

    // The menu boxes: two columns of six, 48 pixels wide and 8 lines tall, the first 2 lines down.
    private const int BOX_LEFT = 430;
    private const int BOX_COLUMN_SPACING = 52;
    private const int BOX_WIDTH = 48;
    private const int BOX_HEIGHT = 8;
    private const int FIRST_BOX_LINE = 2;
    private const int BOX_ROW_SPACING = 9;
    private const int BOX_COLUMNS = 2;
    private const int BOX_ROWS = 6;

    private readonly int width;
    private readonly int lines;
    private readonly uint[] wood;
    private readonly bool[] replaced;
    private readonly bool[] kept;
    private readonly Queue<int> waiting = new();

    /// A background for a panel width x lines pixels (game pixels, before they are scaled up).
    public PanelBackground(int width, int lines)
    {
        this.width = width;
        this.lines = lines;
        wood = BoardPixels(width, lines);
        replaced = new bool[width * lines];
        kept = new bool[width * lines];
    }

    /// Works out which pixels of the panel are black background (from the panel's picture, one line
    /// after another starting at first), by spreading in from the edge through black pixels.
    public void Prepare(uint[] frame, int frameWidth, int firstLine)
    {
        Array.Clear(replaced);
        Array.Clear(kept);
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

    /// The colour of a pixel of the panel: the wood where there was black background, else what the
    /// panel's picture has.
    public uint Pixel(uint[] frame, int frameWidth, int firstLine, int x, int y) =>
        replaced[y * width + x] ? wood[y * width + x] : frame[(firstLine + y) * frameWidth + x];

    // The board's pixels: a plank the size of the panel, without the soft shadow that Wood adds
    // beside and below it.
    private static uint[] BoardPixels(int width, int lines)
    {
        uint[] plank = Wood.Plank(width, lines, BOARD_SEED, false);
        uint[] board = new uint[width * lines];

        for (int y = 0; y < lines; y++)
        {
            Array.Copy(plank, y * (width + Wood.SHADOW), board, y * width, width);
        }

        return board;
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

        if (!replaced[index] && !kept[index] && (frame[(firstLine + y) * frameWidth + x] & COLOR_MASK) == 0)
        {
            replaced[index] = true;
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
}
