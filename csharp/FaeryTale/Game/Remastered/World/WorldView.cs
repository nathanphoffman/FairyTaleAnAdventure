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

    /// Each step of the zoom keys or the mouse wheel multiplies the zoom by this.
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

    public double Zoom { get; private set; } = WINDOW_ZOOM;

    /// The world position the camera looks at.
    public double CenterX { get; set; }
    public double CenterY { get; set; }

    /// Screen pixels to a world pixel.
    public double Scale => Width * Zoom / ORIGINAL_VISIBLE_WIDTH;

    /// Sets the size of the picture and how much of its bottom the text panel covers.
    public void Resize(int width, int height, int panelHeight)
    {
        Width = Math.Max(width, 1);
        Height = Math.Max(height, 1);
        GroundY = Math.Max((Height - panelHeight) / 2, 0);
    }

    public void SetZoom(double zoom) => Zoom = Math.Clamp(zoom, MIN_ZOOM, MAX_ZOOM);

    /// Zooms in (steps above 0) or out (below 0) by whole steps.
    public void ZoomBySteps(int steps) => SetZoom(Zoom * Math.Pow(ZOOM_STEP, steps));

    /// The world position under a pixel of the picture.
    public (double X, double Y) ToWorld(double pixelX, double pixelY) =>
        (CenterX + (pixelX - Width / 2.0) / Scale, CenterY + (pixelY - GroundY) / Scale);

    /// The pixel of the picture a world position is drawn at.
    public (double X, double Y) ToScreen(double worldX, double worldY) =>
        ((worldX - CenterX) * Scale + Width / 2.0, (worldY - CenterY) * Scale + GroundY);
}
