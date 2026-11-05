namespace FaeryTale.Game.Render;

/// The decorative border drawn around story pages.
public static class PlacardBorder
{
    // The border is a closed path made of 16 small steps repeated with changing offsets.
    private static readonly int[] BORDER_X_STEPS = [-4, -4, -4, 0, 0, 0, 4, 4, 0, -4, 0, 4, 4, 0, 0, 0];
    private static readonly int[] BORDER_Y_STEPS = [0, 0, 0, 4, 4, 4, 0, 0, -4, 0, -4, 0, 0, 4, 4, 4];
    private const int BORDER_START_X = 12;
    private const int BORDER_PASSES = 17;
    private const int BORDER_STEPS = 16;

    // The page the border goes around is 284 x 124 (the long edges) and 268 x 112 (the short ones,
    // drawn turned a quarter), with the short edges offset by 16 and 12.
    private const int LONG_EDGE_FAR_X = 284;
    private const int LONG_EDGE_FAR_Y = 124;
    private const int SHORT_EDGE_NEAR_X = 16;
    private const int SHORT_EDGE_NEAR_Y = 12;
    private const int SHORT_EDGE_FAR_X = 268;
    private const int SHORT_EDGE_FAR_Y = 112;

    /// Passes numbered above this also draw the long (top and bottom) edges.
    private const int LONG_EDGE_ABOVE_PASS = 9;
    private const byte BORDER_PEN = 1;
    private const byte BORDER_HIGHLIGHT_PEN = 24;

    /// Strokes per step; the last one is drawn in the highlight pen.
    private const int STROKES_PER_STEP = 5;

    /// Draws the decorative border around a story page: many short strokes, five each with the
    /// last in a highlight colour, winding around the four edges. The first passes also draw the
    /// top and bottom.
    public static void DrawPlacardBorder(Screen screen)
    {
        int xOrigin = BORDER_START_X;
        int yOrigin = 0;

        for (int pass = BORDER_PASSES - 1; pass >= 0; pass--)
        {
            for (int step = 0; step < BORDER_STEPS; step++)
            {
                int dx = xOrigin + BORDER_X_STEPS[step];
                int dy = yOrigin + BORDER_Y_STEPS[step];

                for (int stroke = STROKES_PER_STEP - 1; stroke >= 0; stroke--)
                {
                    screen.SetAPen(PortId.Map, stroke == 0 ? BORDER_HIGHLIGHT_PEN : BORDER_PEN);
                    DrawBorderStroke(screen, pass > LONG_EDGE_ABOVE_PASS, (xOrigin, yOrigin), (dx, dy));
                }

                xOrigin = dx;
                yOrigin = dy;
            }
        }
    }

    // One stroke of the border, repeated on the opposite edge, and on the short edges (turned a
    // quarter) with the x and y swapped.
    private static void DrawBorderStroke(Screen screen, bool longEdges, (int X, int Y) origin, (int X, int Y) to)
    {
        if (longEdges)
        {
            Line(screen, origin.X, origin.Y, to.X, to.Y);
            Line(screen, LONG_EDGE_FAR_X - origin.X, LONG_EDGE_FAR_Y - origin.Y, LONG_EDGE_FAR_X - to.X, LONG_EDGE_FAR_Y - to.Y);
        }

        Line(screen, SHORT_EDGE_NEAR_X + origin.Y, SHORT_EDGE_NEAR_Y - origin.X, SHORT_EDGE_NEAR_X + to.Y, SHORT_EDGE_NEAR_Y - to.X);
        Line(screen, SHORT_EDGE_FAR_X - origin.Y, SHORT_EDGE_FAR_Y + origin.X, SHORT_EDGE_FAR_X - to.Y, SHORT_EDGE_FAR_Y + to.X);
    }

    // Draws a line from (x0, y0) to (x1, y1).
    private static void Line(Screen screen, int x0, int y0, int x1, int y1)
    {
        screen.MoveTo(PortId.Map, x0, y0);
        screen.LineTo(PortId.Map, x1, y1);
    }
}
