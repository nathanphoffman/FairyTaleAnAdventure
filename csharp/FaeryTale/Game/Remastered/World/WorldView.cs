namespace FaeryTale.Game.Remastered;

/// How the world is looked at: where the camera is, how far it is zoomed, and so how a position on
/// the screen and a position in the world belong together. The window, the mouse and the drawer all
/// use this one view.
public sealed class WorldView
{
    /// With zoom 1 the window shows as much of the world across as the original game's play field.
    public const double ORIGINAL_VISIBLE_WIDTH = 304;

    /// Out to four times as far as the original shows, and in to twice as close.
    public const double MIN_ZOOM = 0.25;
    public const double MAX_ZOOM = 2.0;

    /// Each step of the zoom keys or the mouse wheel is about this much closer or further.
    public const double ZOOM_STEP = 1.25;

    /// The zoom a window starts at, and the one fullscreen starts at (twice as far out).
    public const double WINDOW_ZOOM = 1.0;
    public const double FULLSCREEN_ZOOM = 0.5;

    /// The view the game uses (there is one window).
    public static WorldView Current { get; } = new();

    /// The size of the picture in pixels, and the line of it where the hero stands (the middle of the
    /// part not covered by the text panel).
    public int Width { get; private set; } = 640;
    public int Height { get; private set; } = 360;
    public int GroundY { get; private set; } = 180;

    /// How far zoomed: 1 shows as much across as the original game's play field.
    public double Zoom { get; private set; } = WINDOW_ZOOM;

    /// The world position the camera looks at.
    public double CenterX { get; set; }
    public double CenterY { get; set; }

    /// Screen pixels to a world pixel. It is always a whole number, so every pixel of the game is the
    /// same size square on the screen and nothing shimmers as the camera moves.
    public int Scale { get; private set; } = 2;

    /// The smallest and largest scale the zoom limits allow at this window width.
    public int MinScale => Math.Max(1, ScaleFor(MIN_ZOOM));
    public int MaxScale => Math.Max(MinScale, ScaleFor(MAX_ZOOM));

    /// Sets the size of the picture and how much of its bottom the text panel covers.
    public void Resize(int width, int height, int panelHeight)
    {
        Width = Math.Max(width, 1);
        Height = Math.Max(height, 1);
        GroundY = Math.Max((Height - panelHeight) / 2, 0);
        SetZoom(Zoom);
    }

    /// Sets the zoom (kept between the limits); the scale is the nearest whole number to it.
    public void SetZoom(double zoom)
    {
        Zoom = Math.Clamp(zoom, MIN_ZOOM, MAX_ZOOM);
        Scale = Math.Clamp(ScaleFor(Zoom), MinScale, MaxScale);
    }

    /// Zooms in (steps above 0) or out (below 0) by whole steps: each one makes the scale about a
    /// quarter bigger or smaller, and never less than one whole number.
    public void ZoomBySteps(int steps)
    {
        int scale = Scale;

        for (int step = 0; step < Math.Abs(steps); step++)
        {
            scale = steps > 0 ? Math.Max((int)Math.Round(scale * ZOOM_STEP), scale + 1) : Math.Min((int)Math.Round(scale / ZOOM_STEP), scale - 1);
            scale = Math.Clamp(scale, MinScale, MaxScale);
        }

        Scale = scale;
        Zoom = Scale * ORIGINAL_VISIBLE_WIDTH / Width;
    }

    /// The world position under a pixel of the picture.
    public (double X, double Y) ToWorld(double pixelX, double pixelY) =>
        (CenterX + (pixelX - Width / 2.0) / Scale, CenterY + (pixelY - GroundY) / Scale);

    /// The pixel of the picture a world position is drawn at.
    public (double X, double Y) ToScreen(double worldX, double worldY) =>
        ((worldX - CenterX) * Scale + Width / 2.0, (worldY - CenterY) * Scale + GroundY);

    // The whole number of screen pixels to a world pixel that fits a zoom.
    private int ScaleFor(double zoom) => (int)Math.Round(Width * zoom / ORIGINAL_VISIBLE_WIDTH);
}
