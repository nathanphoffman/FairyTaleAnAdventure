using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// Which part of the world overview the big map shows, and how close: the middle of the view (a
/// tile column and row), and how many units each tile column is wide. A tile is twice as high as it
/// is wide, so each row is twice as many units tall. The view is drawn in units of the 640 x 400
/// overlay grid.
public sealed class MapView
{
    /// The view's size in units.
    public const int WIDTH = 592;
    public const int HEIGHT = 340;

    /// Out far enough to see the whole world, in as close as 8 units to a tile.
    public const double MIN_SCALE = 0.15;
    public const double MAX_SCALE = 8.0;

    /// Each step of the wheel changes the scale by this much.
    public const double ZOOM_STEP = 1.25;

    private const int ROW_HEIGHT_FACTOR = 2;

    // The hero's mark on the big map: a ring round a filled square, in whole units whatever the scale.
    private const int MARK_RADIUS = 4;
    private const uint MARK_RING = 0xffff_ffff;
    private const uint MARK_FILL = 0xffe0_1010;
    private const uint OFF_WORLD = 0xff00_0000;

    public double CenterColumn { get; private set; }

    public double CenterRow { get; private set; }

    public double Scale { get; private set; } = 1.0;

    /// Puts the middle of the view on a tile.
    public void CenterOn((int Column, int Row) tile)
    {
        CenterColumn = tile.Column;
        CenterRow = tile.Row;
    }

    /// Moves the view by units of the screen (the land follows a pointer that drags it).
    public void PanBy(double unitsAcross, double unitsDown)
    {
        CenterColumn = Math.Clamp(CenterColumn + unitsAcross / Scale, 0, WorldOverview.COLUMNS - 1);
        CenterRow = Math.Clamp(CenterRow + unitsDown / (Scale * ROW_HEIGHT_FACTOR), 0, WorldOverview.ROWS - 1);
    }

    /// Zooms in (steps above 0) or out (below 0).
    public void ZoomBySteps(int steps) => Scale = Math.Clamp(Scale * Math.Pow(ZOOM_STEP, steps), MIN_SCALE, MAX_SCALE);

    /// The picture of the view: the land, and a mark where the hero is (on the edge of the view
    /// when the hero is out of it, so it always says which way).
    public Image Picture(uint[] world, (int Column, int Row) hero, bool markShown)
    {
        uint[] pixels = new uint[WIDTH * HEIGHT];

        for (int y = 0; y < HEIGHT; y++)
        {
            for (int x = 0; x < WIDTH; x++)
            {
                pixels[y * WIDTH + x] = LandAt(world, x, y);
            }
        }

        if (markShown)
        {
            DrawMark(pixels, MarkPlace(hero));
        }

        return new Image(0, 0, WIDTH, HEIGHT, pixels);
    }

    // The colour of the land at a unit of the view (black outside the world).
    private uint LandAt(uint[] world, int x, int y)
    {
        int column = (int)Math.Floor(CenterColumn + (x - WIDTH / 2.0) / Scale);
        int row = (int)Math.Floor(CenterRow + (y - HEIGHT / 2.0) / (Scale * ROW_HEIGHT_FACTOR));
        bool inside = column >= 0 && column < WorldOverview.COLUMNS && row >= 0 && row < WorldOverview.ROWS;

        return inside ? world[row * WorldOverview.COLUMNS + column] : OFF_WORLD;
    }

    // Where the hero's tile is in the view, kept inside it.
    private (int X, int Y) MarkPlace((int Column, int Row) hero)
    {
        int x = (int)Math.Round(WIDTH / 2.0 + (hero.Column - CenterColumn) * Scale);
        int y = (int)Math.Round(HEIGHT / 2.0 + (hero.Row - CenterRow) * Scale * ROW_HEIGHT_FACTOR);

        return (Math.Clamp(x, MARK_RADIUS, WIDTH - 1 - MARK_RADIUS), Math.Clamp(y, MARK_RADIUS, HEIGHT - 1 - MARK_RADIUS));
    }

    private static void DrawMark(uint[] pixels, (int X, int Y) centre)
    {
        for (int dy = -MARK_RADIUS; dy <= MARK_RADIUS; dy++)
        {
            for (int dx = -MARK_RADIUS; dx <= MARK_RADIUS; dx++)
            {
                pixels[(centre.Y + dy) * WIDTH + centre.X + dx] = Math.Max(Math.Abs(dx), Math.Abs(dy)) == MARK_RADIUS ? MARK_RING : MARK_FILL;
            }
        }
    }
}
