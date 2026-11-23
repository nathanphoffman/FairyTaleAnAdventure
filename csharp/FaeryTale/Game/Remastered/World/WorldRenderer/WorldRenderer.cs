using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The new world drawer of Remastered and Reimagined: draws the map tiles and the sprites the game
/// has already loaded straight into a picture of any size, at any zoom, from where the camera is.
/// It reads the same data as the old drawing (and the same palette, so the day and night colours and
/// fades come out the same) but needs none of the old screen, so the view can show much more.
///
/// A frame can be drawn between two game ticks: alpha is how far from the last tick to the present
/// one it is (1 is the present tick), and the camera and the sprites are drawn that far along.
public sealed partial class WorldRenderer
{
    // The camera looks a little above the hero's feet so the hero is in the middle of the picture.
    private const double CAMERA_LIFT = 12;

    private const int TILE_COLUMN_WRAP = 0x7ff;
    private const int TILE_ROW_WRAP = 0x7fff;

    // Outdoors the world is 32768 pixels across and high, but the rooms inside buildings are further
    // down the same map (their y positions go past 32768, up to 65536).
    private const int WORLD_WIDTH = 0x8000;
    private const int WORLD_HEIGHT = 0x10000;
    private const int PALETTE_SIZE = 32;

    /// The most bands of rows the map is drawn in at the same time.
    private const int MAX_BANDS = 16;
    private const uint OFF_MAP_COLOR = 0xff000000;
    private const uint OPAQUE = 0xff000000;

    // A tile is 16 pixels wide and 32 high; a sector is 16 x 8 tiles; a region is 64 x 32 sectors.
    private const int TILE_WIDTH_SHIFT = 4;
    private const int TILE_HEIGHT_SHIFT = 5;
    private const int SECTOR_COLUMN_SHIFT = 4;
    private const int SECTOR_ROW_SHIFT = 3;
    private const int REGION_SECTOR_COLUMNS = 64;
    private const int REGION_SECTOR_ROWS = 32;

    // Scenery that hides sprites: the terrain table entry is four bytes (the first picks the mask).
    private const int TERRAIN_ENTRY_BYTES = 4;
    private const int TERRAIN_TYPE_MASK = 15;
    private const int SHADOW_MASK_BYTES = 64;
    private const int SHADOW_ROW_BYTES = 2;
    private const int RAFT_SLOT = 1;

    private readonly WorldImages images = new();
    private readonly NeighborRegions neighbors = new();
    private readonly InteriorMask interior = new();
    private readonly uint[] palette = new uint[PALETTE_SIZE];
    private int[] columnToWorld = [];
    private int[] rowToWorld = [];

    /// The colours of the play field now (colour number to 0xAARRGGBB).
    public uint[] Palette => palette;

    /// The picture of a captured sprite, decoded (null when it is not in memory).
    public SpriteImage? PictureOf(Video video, CapturedSprite sprite) => images.Sprite(video.Graphics, sprite.Sheet, sprite.Inum);

    /// The picture of an object by its number (the pictures of the items lying about and of the pack),
    /// decoded (null when it is not in memory).
    public SpriteImage? ObjectPicture(Video video, int number) =>
        images.Sprite(video.Graphics, SpriteImages.SpriteSheet(video, SpriteImages.OBJECTS), number);

    /// Draws the world into frame (view.Width x view.Height pixels).
    public void Draw(uint[] frame, WorldView view, WorldCapture capture, Video video, double alpha)
    {
        if (capture.Game is not { } game)
        {
            Array.Fill(frame, OFF_MAP_COLOR);

            return;
        }

        capture.Prepare();
        LoadPalette(video);
        PlaceCamera(view, capture, alpha);
        MapPositions(view);
        interior.Prepare(game);
        DrawTiles(frame, view, game, video.Graphics);
        DrawSprites(frame, view, capture, game, video, alpha);
    }

    // The 12 bit colours of the play field as 8 bit ones.
    private void LoadPalette(Video video)
    {
        ushort[] colors = video.Screen.GetViewPort(ViewPortId.Page).Colors;

        for (int index = 0; index < PALETTE_SIZE; index++)
        {
            palette[index] = OPAQUE | Scene.Rgb4(index < colors.Length ? colors[index] : (ushort)0);
        }
    }

    // The camera follows the hero, a part of the last tick's move behind the present position.
    private static void PlaceCamera(WorldView view, WorldCapture capture, double alpha)
    {
        view.CenterX = capture.HeroNow.X + capture.HeroDelta.X * (1 - alpha);
        view.CenterY = capture.HeroNow.Y + capture.HeroDelta.Y * (1 - alpha) - CAMERA_LIFT;
        LogCamera(view, capture, alpha);
    }

    // FTA_CAMERA_LOG=file: one line per picture (milliseconds, camera x, game step, alpha) to see
    // whether the camera moves evenly from picture to picture.
    private static readonly string? cameraLogPath = Environment.GetEnvironmentVariable("FTA_CAMERA_LOG");

    private static void LogCamera(WorldView view, WorldCapture capture, double alpha)
    {
        if (cameraLogPath is not null)
        {
            File.AppendAllText(cameraLogPath, $"{System.Diagnostics.Stopwatch.GetTimestamp() * 1000.0 / System.Diagnostics.Stopwatch.Frequency:F2} {view.CenterX:F2} {capture.Tick} {alpha:F3}\n");
        }
    }
}
