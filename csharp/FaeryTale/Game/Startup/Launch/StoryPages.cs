using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Startup;

/// The story pages of the intro: the title picture with the zoom, showing the pages and the
/// page-turn animation between them.
internal static class StoryPages
{
    private const uint ALL_PLANES = 0x1f;
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

    // The page-turn animation, 22 frames: strip step, strip width and delay for each.
    private static readonly short[] FLIP_RATE = [8, 6, 5, 4, 3, 2, 3, 5, 13, 0, 0, 13, 5, 3, 2, 3, 4, 5, 6, 8, 0, 0];
    private static readonly int[] FLIP_WIDE = [7, 5, 4, 3, 2, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 2, 3, 4, 5, 7, 0, 0];
    private static readonly int[] FLIP_DELAY = [12, 9, 6, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 6, 9, 0, 0];

    /// The first half of the frames turn the right page up, the rest lay the left one down.
    private const int FIRST_HALF = 11;
    private const int PAGE_HALF_WIDTH = 135;
    private const short PAGE_STRIP_END = 136;

    // The page turn is drawn around the book's spine at x = 160 (the right page starts one pixel
    // later); the left page starts at x = 24 and the outer edge of the right page is at x = 296.
    // The last strip is a pixel wide, 7 lines in from the top and bottom, which leaves 186 lines.
    private const int SPINE = 160;
    private const int SPINE_RIGHT = SPINE + 1;
    private const int PAGE_LEFT_EDGE = 24;
    private const int PAGE_RIGHT_EDGE = 296;
    private const int EDGE_INSET = 7;
    private const int PAGE_EDGE_HEIGHT = 186;

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

    // The page turn animation, 22 frames.
    //
    // The first 11 frames lift the right page: the old page is drawn as thin vertical strips that
    // get narrower and shorter (using PageTurnInset for the curl), on top of the new page. The
    // last 11 frames lay the curling page down on the left. FLIP_RATE, FLIP_WIDE and FLIP_DELAY
    // give the strip step, strip width and pause for each frame.
    private static void TurnPage(Data.Game game, Video video)
    {
        for (int frame = 0; frame < FLIP_RATE.Length; frame++)
        {
            if (Launch.LeaveIntroNow(game.Ui))
            {
                return;
            }

            BitmapId temp = video.Screen.DrawingBitmapId;

            if (frame < FIRST_HALF)
            {
                TurnRightPageFrame(video, temp, FLIP_RATE[frame], FLIP_WIDE[frame]);
            }
            else
            {
                TurnLeftPageFrame(video, temp, FLIP_RATE[frame], FLIP_WIDE[frame]);
            }

            Display.FlipPages(game, video);

            if (FLIP_DELAY[frame] != 0)
            {
                Display.Delay(game, video, FLIP_DELAY[frame]);
            }
        }
    }

    // One frame of the first half of the page turn: the right page B stays and the strips of the
    // old page A bend away from the spine, each wide pixels wide and rate apart.
    private static void TurnRightPageFrame(Video video, BitmapId temp, short rate, int wide)
    {
        int bColumn = SPINE_RIGHT - wide;
        int dColumn = 0;

        CopyHalfPage(video, BitmapId.PageB, temp, SPINE);

        if (rate == 0)
        {
            return;
        }

        int inset = 0;

        for (short sColumn = (short)wide; sColumn < PAGE_STRIP_END; sColumn += rate)
        {
            inset = MapCamera.PageTurnInset(sColumn);
            CopyStrip(video, BitmapId.PageA, temp, (bColumn + sColumn, SPINE_RIGHT + dColumn), inset, wide);
            dColumn += wide;
        }

        CopyRect(video, BitmapId.PageA, (PAGE_RIGHT_EDGE, EDGE_INSET), temp, (SPINE_RIGHT + dColumn, inset), (1, PAGE_EDGE_HEIGHT));
    }

    // One frame of the second half of the page turn: the strips of the new page B land on the left.
    private static void TurnLeftPageFrame(Video video, BitmapId temp, short rate, int wide)
    {
        if (rate == 0)
        {
            CopyHalfPage(video, BitmapId.PageB, temp, PAGE_LEFT_EDGE);

            return;
        }

        CopyHalfPage(video, BitmapId.PageA, temp, PAGE_LEFT_EDGE);
        int dColumn = 0;

        for (short sColumn = (short)wide; sColumn < PAGE_STRIP_END; sColumn += rate)
        {
            int inset = MapCamera.PageTurnInset(sColumn);

            dColumn += wide;
            CopyStrip(video, BitmapId.PageB, temp, (SPINE - sColumn, SPINE - dColumn), inset, wide);
        }

        CopyStrip(video, BitmapId.PageB, temp, (PAGE_LEFT_EDGE, SPINE - 1 - dColumn), EDGE_INSET, 1);
    }

    // Copies the half page that starts at column column onto the screen, in the same place.
    private static void CopyHalfPage(Video video, BitmapId source, BitmapId temp, int column) =>
        CopyRect(video, source, (column, 0), temp, (column, 0), (PAGE_HALF_WIDTH, SCREEN_HEIGHT));

    // Copies a strip wide columns wide from one column to another, inset lines in from the top and
    // bottom (the curl of the page).
    private static void CopyStrip(Video video, BitmapId source, BitmapId temp, (int From, int To) columns, int inset, int wide) =>
        CopyRect(video, source, (columns.From, inset), temp, (columns.To, inset), (wide, SCREEN_HEIGHT - inset - inset));

    // Copies a rectangle between bitmaps (all five planes, plain copy).
    private static void CopyRect(Video video, BitmapId source, (int X, int Y) from, BitmapId destination, (int X, int Y) to, (int Width, int Height) size) =>
        video.Screen.Copy(source, new Rect(from.X, from.Y, size.Width, size.Height), destination, to, ALL_PLANES);

    // Copies the whole title picture (loaded into PageB) onto the bitmap of a display page.
    private static void CopyTitlePictureTo(Screen screen, BitmapId page) =>
        screen.Copy(BitmapId.PageB, new Rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT), page, (0, 0), ALL_PLANES);
}
