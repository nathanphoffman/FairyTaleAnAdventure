using FaeryTale.Game.Input;
using FaeryTale.Game.Land;
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
internal sealed class NativeComposer
{
    // The panel is the bottom of the old picture: game lines from 143 down (the grid has every
    // line twice).
    private const int PANEL_FIRST_LINE = 143;
    private const int GRID_ROWS_PER_LINE = 2;
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
    private readonly PanelBackground panelBackground = new(Scene.FRAME_WIDTH, Scene.FRAME_HEIGHT - PANEL_FIRST_LINE);
    private readonly uint[] legacyFrame = new uint[Scene.FRAME_WIDTH * Scene.FRAME_HEIGHT];
    private uint[] picture = [];
    private uint[] scaledRow = [];

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
            InputHandler.NativeMouseLeftWorld();
            InputBridge.MouseMove(Math.Clamp(x, 0, GRID_WIDTH - 1), Math.Clamp(y, 0, GRID_HEIGHT - 1) / GRID_ROWS_PER_LINE);
        }

        // The menus are on the grid whatever is under the pointer.
        InputHandler.NativeGridMoved(x, FloorDivide(pixelY - OverlayTop, Scale));
    }

    /// Where a window pixel is on the 640 x 400 grid (not clamped).
    public (int X, int Y) ToGrid(int pixelX, int pixelY) =>
        (FloorDivide(pixelX - GridLeft, Scale), FloorDivide(pixelY - GridTop, Scale));

    /// Whether a window pixel is on the text panel (only while the world is shown).
    public bool IsOverPanel(int pixelX, int pixelY)
    {
        (int x, int y) = ToGrid(pixelX, pixelY);

        return ShowsWorld && x >= 0 && x < GRID_WIDTH && y >= PANEL_FIRST_ROW;
    }

    // The world, then the click marker, the panel, the menus and the pointer.
    private void ComposeWorld(int width, int height, Scene scene, Overlay overlay, Video video, double alpha, (int X, int Y) pointer)
    {
        WorldView view = WorldView.Current;

        view.Resize(width, height, (GRID_HEIGHT - PANEL_FIRST_ROW) * Scale);
        renderer.Draw(picture, view, video.Capture, video, alpha);
        DrawMarker(width, view, video.Capture);
        DrawMinimap(width, view, video.Capture);
        DrawAttackTarget(width, view, video.Capture);

        scene.ComposeFromLine(legacyFrame, PANEL_FIRST_LINE);
        DrawPanel(width);
        DrawLitButtons(width, video.Capture);
        DrawBackButton(width, video.Capture);
        DrawInventory(width, video);
        MessageScrollBar.Of(video.History).DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
        overlay.DrawAt(picture, width, new Overlay.Placement(GridLeft, OverlayTop, Scale));

        DrawConversation(width, video, pointer);
        DrawPointer(width, scene, overlay, video, pointer);
    }

    // The pointer: a door or a face when over one, else the game's own.
    private void DrawPointer(int width, Scene scene, Overlay overlay, Video video, (int X, int Y) pointer)
    {
        WorldCapture capture = video.Capture;
        Image? face = capture.TalkingTo is null && capture.HoverPerson is { } person ? FaceOf(video, person) : null;

        if (overlay.HidePointer)
        {
            return;
        }

        if (capture.LookMode)
        {
            new Overlay { Images = { EyeCursor.At((0, 0)) } }.DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else if (capture.EnemyHover)
        {
            new Overlay { Images = { Crosshair.At((0, 0), POINTER_CROSSHAIR_RADIUS) } }.DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else if (capture.ItemHover && capture.TalkingTo is null)
        {
            new Overlay { Images = { HandCursor.At((0, 0)) } }.DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else if (face is not null)
        {
            new Overlay { Images = { face with { X = -face.Width / 2, Y = -face.Height / 2 } } }
                .DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else if (capture.DoorHover)
        {
            new Overlay { Images = { DoorCursor.At((0, 0)) } }.DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else
        {
            scene.DrawPointerAtPixel(picture, width, pointer, POINTER_PIXELS_PER_GAME_PIXEL * Scale);
        }
    }

    // The conversation bubble over the person the hero is talking to.
    private void DrawConversation(int width, Video video, (int X, int Y) pointer)
    {
        if (video.Capture.TalkingTo is not { } number || FaceOf(video, number) is not { } face)
        {
            return;
        }

        FaeryTale.Game.Data.Game game = video.Capture.Game!;
        BubbleLayout layout = Conversation.Layout(game);
        List<Row> rows = Conversation.Rows(game);
        Overlay bubble = ConversationPicture.Draw(face, rows, layout.RowAt(pointer.X, pointer.Y, rows));

        bubble.DrawAt(picture, width, new Overlay.Placement(layout.Left, layout.Top, layout.Scale));
    }

    // The face of the person with an actor number, from the sprite just drawn.
    private Image? FaceOf(Video video, int number)
    {
        CapturedSprite? sprite = video.Capture.Sprites.Find(found => found.Slot == number && found.Pass == 0);

        return sprite is not null && renderer.PictureOf(video, sprite) is { } sheetPicture
            ? FacePicture.Face(renderer.Palette, sheetPicture)
            : null;
    }

    // The red crosshair over the enemy being attacked.
    private void DrawAttackTarget(int width, WorldView view, WorldCapture capture)
    {
        if (capture.AttackTarget is not { } target)
        {
            return;
        }

        (double x, double y) = view.ToScreen(target.X, target.Y);

        new Overlay { Images = { Crosshair.At(((int)Math.Round(x), (int)Math.Round(y)), TARGET_RADIUS_WORLD_PIXELS * view.Scale) } }.Draw(picture, width);
    }

    // The inventory screen over the world, while it is open (the game goes on behind it).
    private void DrawInventory(int width, Video video)
    {
        if (!Inventory.IsOpen || video.Capture.Game is not { } game)
        {
            return;
        }

        InventoryPicture.Of(game, number => renderer.ObjectPicture(video, number), InputHandler.GridPointer)
            .DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
    }

    // The "< Back" button above the classic buttons while a submenu is open.
    private void DrawBackButton(int width, WorldCapture capture)
    {
        if (capture.Game is { } game && PanelBackButton.IsShown(game))
        {
            PanelBackButton.Of(PanelBackButton.Contains(InputHandler.GridPointer)).DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
        }
    }

    // A red square round each classic button the new controls are using.
    private void DrawLitButtons(int width, WorldCapture capture)
    {
        if (capture.Game is { } game)
        {
            PanelButtonOutline.Of(game, capture.LitButtons).DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
        }
    }

    // The small map in the top right corner of the world (outdoors only).
    private void DrawMinimap(int width, WorldView view, WorldCapture capture)
    {
        if (!MapSettings.Enabled || capture.Game is not { } game || game.World.Region >= Regions.FIRST_INDOOR_REGION)
        {
            return;
        }

        int unit = Math.Max(1, (int)Math.Round(view.Height / UNIT_HEIGHT));
        (int column, int row) = WorldOverview.TileOf(capture.HeroNow.X, capture.HeroNow.Y);
        Image map = MinimapPicture.Of(WorldOverview.Pixels, (column, row));

        new Overlay { Images = { map } }.DrawAt(picture, width, new Overlay.Placement(width - (MinimapPicture.WIDTH + MINIMAP_MARGIN) * unit, MINIMAP_MARGIN * unit, unit));
    }

    // The circle where the player clicked, over the ground it is on.
    private void DrawMarker(int width, WorldView view, WorldCapture capture)
    {
        if (capture.Marker is not { } marker)
        {
            return;
        }

        (double x, double y) = view.ToScreen(marker.World.X, marker.World.Y);

        new Overlay { Images = { ClickRing.PictureAtPixel(((int)Math.Round(x), (int)Math.Round(y)), marker.Walkable, view.Scale) } }
            .Draw(picture, width);
    }

    // The bottom of the old picture, each game pixel a Scale wide and 2 x Scale tall. Each line is
    // scaled across once and then copied down.
    private void DrawPanel(int width)
    {
        int height = picture.Length / width;
        int scaledWidth = Scene.FRAME_WIDTH * Scale;

        if (scaledRow.Length != scaledWidth)
        {
            scaledRow = new uint[scaledWidth];
        }

        panelBackground.Prepare(legacyFrame, Scene.FRAME_WIDTH, PANEL_FIRST_LINE);

        for (int line = PANEL_FIRST_LINE; line < Scene.FRAME_HEIGHT; line++)
        {
            for (int x = 0; x < Scene.FRAME_WIDTH; x++)
            {
                Array.Fill(scaledRow, panelBackground.Pixel(legacyFrame, Scene.FRAME_WIDTH, PANEL_FIRST_LINE, x, line - PANEL_FIRST_LINE), x * Scale, Scale);
            }

            for (int copy = 0; copy < GRID_ROWS_PER_LINE * Scale; copy++)
            {
                int y = GridTop + line * GRID_ROWS_PER_LINE * Scale + copy;

                if (y >= 0 && y < height)
                {
                    CopyRowInto(y, width, scaledWidth);
                }
            }
        }
    }

    // Puts the scaled row on a line of the picture, where it is on the picture.
    private void CopyRowInto(int y, int width, int scaledWidth)
    {
        int from = Math.Max(0, -GridLeft);
        int to = Math.Min(scaledWidth, width - GridLeft);

        if (to > from)
        {
            Array.Copy(scaledRow, from, picture, y * width + GridLeft + from, to - from);
        }
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

    private static int FloorDivide(int value, int by) => (int)Math.Floor((double)value / by);
}
