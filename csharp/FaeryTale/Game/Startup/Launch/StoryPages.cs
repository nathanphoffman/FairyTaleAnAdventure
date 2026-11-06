using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Startup;

/// The story pages of the intro: the title picture with the zoom, showing the pages and the
/// page-turn animation between them.
internal static class StoryPages
{
    private const uint ALL_PLANES = 0x1f;
    private const uint OPAQUE = 0xff00_0000;
    private const int SCREEN_WIDTH = 320;
    private const int SCREEN_HEIGHT = 200;

    // Intro: how big the picture zooms (half the width in pixels), and the step.
    private const int ZOOM_FULL = 160;
    private const int ZOOM_NORMAL = 156;
    private const int ZOOM_STEP = 4;

    /// The story pages: file names and the delays in ticks before and after each is shown.
    private static readonly (string First, string Second, short FirstWait, short SecondWait)[] STORY_PAGES =
    [
        ("p1a", "p1b", 21, 29),
        ("p2a", "p2b", 20, 29),
        ("p3a", "p3b", 20, 33),
    ];

    private const int INTRO_PAGES_WAIT = 190;

    // Where the first story picture goes on the page.
    private const int PICTURE_X = 4;
    private const int PICTURE_Y = 24;

    /// Ticks to show a page before turning it.
    private const int PAGE_DELAY = 350;

    /// How long the page turn takes (it turns at one picture for every refresh of the monitor).
    private const double TURN_MS = 1100;

    /// The story pictures with the zoom in and out (skipped if the player presses Space).
    public static void PlayIntro(Data.Game game, Video video)
    {
        Files.IffPicture.LoadPicture("page0", video.Screen.GetBitmap(BitmapId.PageB), 0, 0);
        CopyTitlePictureTo(video.Screen, BitmapId.Page1);
        CopyTitlePictureTo(video.Screen, BitmapId.Page2);

        video.Screen.Pages[1].Ras = 0;

        for (int x = 0; x <= ZOOM_FULL; x += ZOOM_STEP)
        {
            if (Launch.LeaveIntroNow(game.Ui))
            {
                break;
            }

            ScreenEffects.SetIntroZoom(game, video, x);
        }

        video.Screen.Pages[1].Ras = 1;

        if (Launch.IntroSkipped(game.Ui) == 0)
        {
            foreach ((string first, string second, short firstWait, short secondWait) in STORY_PAGES)
            {
                ShowStoryPage(game, video, first, second, firstWait, secondWait);
            }

            if (game.Ui.IntroSkipRequested == 0)
            {
                Launch.SkippableDelay(game, video, INTRO_PAGES_WAIT);
            }
        }

        video.Screen.Pages[1].Ras = 0;

        for (int x = ZOOM_NORMAL - ZOOM_NORMAL % ZOOM_STEP; x >= 0; x -= ZOOM_STEP)
        {
            if (Launch.LeaveIntroNow(game.Ui))
            {
                break;
            }

            ScreenEffects.SetIntroZoom(game, video, x);
        }
    }

    /// Shows two pictures on the story page, then turns it over. The new pictures are drawn onto
    /// PageB while PageA keeps the old page for the turn.
    internal static void ShowStoryPage(Data.Game game, Video video, string first, string second, short x, short y)
    {
        if (game.Ui.IntroSkipRequested != 0)
        {
            return;
        }

        Launch.SkippableDelay(game, video, PAGE_DELAY);

        if (Launch.LeaveIntroNow(game.Ui))
        {
            return;
        }

        video.Screen.Copy(BitmapId.PageB, new Rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT), BitmapId.PageA, (0, 0), ALL_PLANES);

        Bitmap newPage = video.Screen.GetBitmap(BitmapId.PageB);

        Files.IffPicture.LoadPicture(first, newPage, PICTURE_X, PICTURE_Y);
        Files.IffPicture.LoadPicture(second, newPage, x, y);

        if (Launch.IntroSkipped(game.Ui) != 0)
        {
            return;
        }

        TurnPage(game, video);
        Launch.IntroSkipped(game.Ui);
    }

    // The page turn: the pictures of PageTurn from the old page (PageA) to the new one (PageB),
    // shown over the screen one for every refresh of the monitor, with an easy start and end.
    private static void TurnPage(Data.Game game, Video video)
    {
        ushort[] colors = video.Screen.GetViewPort(ViewPortId.Page).Colors;
        PageTurn turn = new(DecodePage(video.Screen.GetBitmap(BitmapId.PageA), colors), DecodePage(video.Screen.GetBitmap(BitmapId.PageB), colors));
        int frames = Math.Max(2, (int)Math.Ceiling(TURN_MS / video.AnimationFrameMs));

        for (int frame = 1; frame < frames; frame++)
        {
            if (Launch.LeaveIntroNow(game.Ui))
            {
                break;
            }

            double progress = frame / (double)frames;
            uint[] pixels = turn.Render(progress * progress * (3 - 2 * progress));

            video.SetOverlay(new Overlay { Images = { new Image(0, 0, PageTurn.OUTPUT_WIDTH, PageTurn.OUTPUT_HEIGHT, pixels) } });
            Display.AnimationFrame(game, video);
        }

        video.SetOverlay(new Overlay());
        CopyRect(video, BitmapId.PageB, (PageTurn.BOOK_LEFT_EDGE, 0), video.Screen.DrawingBitmapId, (PageTurn.BOOK_LEFT_EDGE, 0), (PageTurn.BOOK_RIGHT_EDGE - PageTurn.BOOK_LEFT_EDGE, SCREEN_HEIGHT));
        Display.FlipPages(game, video);
    }

    // A story page as colours (0xAARRGGBB), 320 x 200.
    private static uint[] DecodePage(Bitmap page, ushort[] colors)
    {
        uint[] pixels = new uint[SCREEN_WIDTH * SCREEN_HEIGHT];

        for (int y = 0; y < SCREEN_HEIGHT; y++)
        {
            for (int x = 0; x < SCREEN_WIDTH; x++)
            {
                int index = page.Pixel(x, y);

                pixels[y * SCREEN_WIDTH + x] = OPAQUE | Scene.Rgb4(index < colors.Length ? colors[index] : (ushort)0);
            }
        }

        return pixels;
    }

    // Copies a rectangle between bitmaps (all five planes, plain copy).
    private static void CopyRect(Video video, BitmapId source, (int X, int Y) from, BitmapId destination, (int X, int Y) to, (int Width, int Height) size) =>
        video.Screen.Copy(source, new Rect(from.X, from.Y, size.Width, size.Height), destination, to, ALL_PLANES);

    // Copies the whole title picture (loaded into PageB) onto the bitmap of a display page.
    private static void CopyTitlePictureTo(Screen screen, BitmapId page) =>
        screen.Copy(BitmapId.PageB, new Rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT), page, (0, 0), ALL_PLANES);
}
