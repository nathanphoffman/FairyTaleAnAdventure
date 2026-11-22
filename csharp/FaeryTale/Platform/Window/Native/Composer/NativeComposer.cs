using FaeryTale.Game.Input;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// Builds the picture of a window of any size for Remastered and Reimagined: the world drawn by the
/// new world drawer fills the window, with the text panel (menus, compass, messages) over the bottom
/// middle and the menus over everything. Pages that are not the world (the story pages, the item
/// list, the map) are the old 640 x 400 picture, scaled up and centred.
///
/// The old picture is a 640 x 400 grid (320 x 200 game pixels, drawn twice as wide and as tall);
/// the panel and the pages are scaled by a whole number so their pixels stay square.
internal sealed partial class NativeComposer
{
    // The panel is the bottom of the old picture: game lines from 143 down (the grid has every
    // line twice).
    private const int PANEL_FIRST_LINE = 143;
    private const int GRID_ROWS_PER_LINE = 2;

    // An enemy's health bar, in world pixels, and its colours.
    private const int HEALTH_BAR_WIDTH = 24;
    private const int HEALTH_BAR_HEIGHT = 3;
    private const uint HEALTH_BAR_EDGE = 0xff10_0808;
    private const uint HEALTH_BAR_GONE = 0xff70_1010;
    private const uint HEALTH_BAR_LEFT = 0xff30_c040;
    private const int GRID_WIDTH = Overlay.WIDTH;
    private const int GRID_HEIGHT = Overlay.HEIGHT;
    private const int PANEL_FIRST_ROW = PANEL_FIRST_LINE * GRID_ROWS_PER_LINE;
    private const int POINTER_PIXELS_PER_GAME_PIXEL = 2;

    // The small map's units are a whole number of pixels, one for each 400 of the window's height.
    private const double UNIT_HEIGHT = 400.0;
    private const int MINIMAP_MARGIN = 6;

    // The crosshair as the pointer (in overlay units) and over the target (in world pixels).
    private const int POINTER_CROSSHAIR_RADIUS = 7;
    private const int TARGET_RADIUS_WORLD_PIXELS = 11;

    private readonly WorldRenderer renderer = new();
    private readonly Dictionary<(int Blocks, int Rows), uint[]> hotbarWood = [];
    private readonly PanelLayout panelLayout = new(Scene.FRAME_WIDTH, Scene.FRAME_HEIGHT - PANEL_FIRST_LINE);
    private readonly uint[] legacyFrame = new uint[Scene.FRAME_WIDTH * Scene.FRAME_HEIGHT];
    private uint[] picture = [];

    /// Window pixels per grid pixel, and where the grid's top left corner is in the window.
    public int Scale { get; private set; } = 1;
    public int GridLeft { get; private set; }
    public int GridTop { get; private set; }

    /// Where the overlay's grid starts: the same as GridTop, or centred on the window for an
    /// overlay that asks for that.
    public int OverlayTop { get; private set; }

    /// Where the pointer was last, in window pixels (the pointer is drawn there).
    public (int X, int Y) Pointer { get; private set; } = (GRID_WIDTH / 2, GRID_HEIGHT / 2);

    /// Whether the last picture was the world (else an old page).
    public bool ShowsWorld { get; private set; }

    /// Builds the picture for a window of width x height pixels. alpha is how far between the last
    /// game tick and the present one the picture is (1 for the present).
    public uint[] Compose(int width, int height, Scene scene, Overlay overlay, Video? video, double alpha, (int X, int Y) pointer)
    {
        if (picture.Length != width * height)
        {
            picture = new uint[width * height];
        }

        Scale = Math.Max(1, Math.Min(width / GRID_WIDTH, height / GRID_HEIGHT));
        GridLeft = (width - GRID_WIDTH * Scale) / 2;
        ShowsWorld = video is not null && WindowControls.NativeRendering && video.Capture.Enabled && video.Capture.WorldVisible;

        if (ShowsWorld)
        {
            GridTop = height - GRID_HEIGHT * Scale;
            OverlayTop = overlay.CenterOnScreen ? (height - GRID_HEIGHT * Scale) / 2 : GridTop;
            ComposeWorld(width, height, scene, overlay, video!, alpha, pointer);
        }
        else
        {
            GridTop = (height - GRID_HEIGHT * Scale) / 2;
            OverlayTop = GridTop;
            ComposePage(width, height, scene, overlay);
        }

        return picture;
    }

    /// The mouse moved to a window pixel: over the world the game is told the pixel (clicks there
    /// walk the hero), over the panel or an old page it is told the place on the 640 x 400 grid.
    public void RouteMouse(int pixelX, int pixelY)
    {
        Pointer = (pixelX, pixelY);

        (int x, int y) = ToGrid(pixelX, pixelY);

        if (ShowsWorld && !IsOverPanel(pixelX, pixelY))
        {
            InputHandler.NativeMouseMoved(pixelX, pixelY);
        }
        else
        {
            (int classicX, int classicY) = ShowsWorld ? ToClassicPanel(x, y) : (x, y);

            InputHandler.NativeMouseLeftWorld();
            InputBridge.MouseMove(Math.Clamp(classicX, 0, GRID_WIDTH - 1), Math.Clamp(classicY, 0, GRID_HEIGHT - 1) / GRID_ROWS_PER_LINE);
        }

        // The menus are on the grid whatever is under the pointer.
        InputHandler.NativeGridMoved(x, FloorDivide(pixelY - OverlayTop, Scale));
    }

    // A place of the grid over the panel, turned back into where it is on the classic panel (the pieces of
    // the panel are not where they were, but the menus and the compass still think in the old places).
    private static (int X, int Y) ToClassicPanel(int x, int y)
    {
        (int classicX, int classicRows) = PanelLayout.ToClassic(x, y - PANEL_FIRST_ROW);

        return (classicX, classicRows + PANEL_FIRST_ROW);
    }

    /// Where a window pixel is on the 640 x 400 grid (not clamped).
    public (int X, int Y) ToGrid(int pixelX, int pixelY) =>
        (FloorDivide(pixelX - GridLeft, Scale), FloorDivide(pixelY - GridTop, Scale));

    // The world, then the click marker, the panel, the menus and the pointer.
    private void ComposeWorld(int width, int height, Scene scene, Overlay overlay, Video video, double alpha, (int X, int Y) pointer)
    {
        WorldView view = WorldView.Current;

        view.Resize(width, height, (GRID_HEIGHT - PANEL_FIRST_ROW) * Scale);
        renderer.Draw(picture, view, video.Capture, video, alpha);
        DrawMarker(width, view, video.Capture);
        DrawMinimap(width, view, video.Capture);
        DrawAttackTarget(width, view, video.Capture);
        DrawHealthBars(width, view, video.Capture);

        scene.ComposeFromLine(legacyFrame, PANEL_FIRST_LINE);
        DrawPanel(width);
        DrawLitButtons(width, video.Capture);
        DrawBackButton(width, video.Capture);
        DrawHotbar(width, video, pointer);
        DrawInventory(width, video);
        DrawJournal(width);
        MessageScrollBar.Of(video.History).DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
        DrawItemTip(width, video);
        overlay.DrawAt(picture, width, new Overlay.Placement(GridLeft, OverlayTop, Scale));

        DrawConversation(width, video, pointer);
        DrawPointer(width, scene, overlay, video, pointer);
    }

    // An old page: black all round and the 640 x 400 picture scaled up in the middle.
    private void ComposePage(int width, int height, Scene scene, Overlay overlay)
    {
        (uint[] pixels, int rows) = FrameComposer.Build(scene, overlay, legacyFrame);
        int rowScale = GRID_HEIGHT / rows * Scale;

        Array.Fill(picture, 0xff00_0000u);

        for (int row = 0; row < rows; row++)
        {
            for (int copy = 0; copy < rowScale; copy++)
            {
                int y = GridTop + row * rowScale + copy;

                if (y < 0 || y >= height)
                {
                    continue;
                }

                for (int x = 0; x < GRID_WIDTH * Scale; x++)
                {
                    int screenX = GridLeft + x;

                    if (screenX >= 0 && screenX < width)
                    {
                        picture[y * width + screenX] = pixels[row * GRID_WIDTH + x / Scale];
                    }
                }
            }
        }
    }
}
