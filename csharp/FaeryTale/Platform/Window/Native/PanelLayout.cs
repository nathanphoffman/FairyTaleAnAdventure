using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// How the text panel is laid out in Remastered and Reimagined: the beige message box, the menu
/// buttons and the compass sit on a board of dark wood instead of a black strip. The classic panel
/// picture is cut into those three pieces and put together again with a little room between them, the
/// compass level with the menu, the message box raised so that its middle is level with theirs, and a
/// thick black frame round the menu buttons.
///
/// The picture it makes has 0 where the wood should show. The mouse still works with the
/// classic positions, so ToClassic turns a place of the new layout back into the old one.
internal sealed class PanelLayout
{
    // The panel's picture has no alpha, so black is whatever has no red, green or blue in it.
    private const uint COLOR_MASK = 0x00ff_ffff;
    private const uint SOLID = 0xff00_0000;

    // The three pieces, by where they start across the classic picture: the message box ends where the
    // menu begins, and the menu where the compass begins.
    private const int MENU_START = 426;
    private const int COMPASS_START = 534;

    /// How far each piece is moved to make room between them (pixels across), and how far the compass is
    /// raised so that it is level with the menu (lines up).
    public const int MENU_SHIFT = 4;
    public const int COMPASS_SHIFT = 8;
    public const int COMPASS_RAISE_LINES = 5;

    /// The message box is raised this many lines, so that the menu and the compass are level with its
    /// middle and not with its top. (Moving the menu down instead would push the six rows of the Magic
    /// and Buy menus off the bottom of the window.)
    public const int MESSAGE_RAISE_LINES = 4;

    /// The new layout reaches this many lines above the classic panel (over the terrain): room for the
    /// raised pieces and for the wood above them.
    public const int TOP_PAD_LINES = 8;

    /// The classic panel starts at this line of the 200 line picture.
    public const int PANEL_FIRST_LINE = 143;

    /// The wood reaches this many pixels past the pieces on each side.
    public const int PAD_SIDE = 8;

    // The panel's wood is darker than the menus' boards (out of 100 of their brightness).
    private const int WOOD_BRIGHTNESS_PERCENT = 62;
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

    // The black frame round the menu: this many pixels at the sides and lines at the top and bottom,
    // so that it is as thick on every side.
    private const int FRAME_PIXELS = 4;
    private const int FRAME_LINES = 2;

    private readonly int width;
    private readonly int lines;
    private readonly bool[] background;
    private readonly bool[] kept;
    private readonly Queue<int> waiting = new();
    private readonly uint[] wood;
    private int lastBoxRow;

    /// The new layout's picture, ArrangedWidth across and ArrangedLines down (starting TOP_PAD_LINES above
    /// the classic panel); 0 where the terrain shows.
    public uint[] Arranged { get; }

    /// How wide the new layout is: the classic width and the room the compass was moved over.
    public int ArrangedWidth => width + COMPASS_SHIFT;

    /// How tall the new layout is: the classic panel's lines and the room above it.
    public int ArrangedLines => lines + TOP_PAD_LINES;

    /// A layout for a panel width x lines pixels (game pixels, before they are scaled up).
    public PanelLayout(int width, int lines)
    {
        this.width = width;
        this.lines = lines;
        background = new bool[width * lines];
        kept = new bool[width * lines];
        Arranged = new uint[(width + COMPASS_SHIFT) * (lines + TOP_PAD_LINES)];
        wood = BoardPixels(width + COMPASS_SHIFT + 2 * PAD_SIDE, lines + TOP_PAD_LINES);
    }

    /// Builds the new layout from the classic panel's picture (lines starting at firstLine).
    public void Prepare(uint[] frame, int frameWidth, int firstLine)
    {
        Array.Clear(background);
        Array.Clear(kept);
        Array.Clear(Arranged);
        lastBoxRow = -1;

        FindBackground(frame, frameWidth, firstLine);
        DrawMenuFrame();
        CopyPieces(frame, frameWidth, firstLine);
    }

    /// The wood at a place of the board (x across from its left edge, y down from its top). The board is
    /// ArrangedWidth + 2 * PAD_SIDE wide and ArrangedLines tall.
    public uint BoardPixel(int x, int y) => wood[y * (ArrangedWidth + 2 * PAD_SIDE) + x];

    /// The place of the classic panel that a place of the new layout is (x across in pixels, y down in
    /// rows of the 640 x 400 grid, both counted from the panel's top left), so the mouse finds the
    /// buttons and the compass where they were.
    public static (int X, int Y) ToClassic(int x, int y)
    {
        if (x >= COMPASS_START + COMPASS_SHIFT)
        {
            return (x - COMPASS_SHIFT, y + COMPASS_RAISE_LINES * 2);
        }

        return x >= MENU_START + MENU_SHIFT ? (x - MENU_SHIFT, y) : (x, y + MESSAGE_RAISE_LINES * 2);
    }

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

    // Where a pixel of the classic picture goes: everything the room above lower in the picture, the
    // message box up a little, the menu a little to the right, the compass a little more to the right
    // and up.
    private static (int X, int Y) Moved(int x, int y)
    {
        if (x >= COMPASS_START)
        {
            return (x + COMPASS_SHIFT, y - COMPASS_RAISE_LINES + TOP_PAD_LINES);
        }

        return x >= MENU_START ? (x + MENU_SHIFT, y + TOP_PAD_LINES) : (x, y + TOP_PAD_LINES - MESSAGE_RAISE_LINES);
    }

    // The board's pixels: a plank the width and height of the board, without the soft shadow that Wood
    // adds beside and below it, darkened.
    internal static uint[] BoardPixels(int boardWidth, int boardLines)
    {
        uint[] plank = Wood.Plank(boardWidth, boardLines, BOARD_SEED);
        uint[] board = new uint[boardWidth * boardLines];

        for (int y = 0; y < boardLines; y++)
        {
            Array.Copy(plank, y * (boardWidth + Wood.SHADOW), board, y * boardWidth, boardWidth);
        }

        return board.Select(Darken).ToArray();
    }

    // A colour with red, green and blue cut down to the panel's wood brightness.
    private static uint Darken(uint color)
    {
        uint Part(int shift) => ((color >> shift) & 0xff) * WOOD_BRIGHTNESS_PERCENT / 100 << shift;

        return (color & 0xff000000) | Part(16) | Part(8) | Part(0);
    }
}
