using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen effects and panel updates: the compass, the item list screen, the print queue, the
/// intro zoom and the win sequence. (Fading the colours is in Fading.)
public static partial class ScreenEffects
{
    // The compass is drawn in the third plane of the text panel.
    private const int COMPASS_PLANE = 2;
    private const int COMPASS_X = 567;
    private const int COMPASS_Y = 15;
    private const int COMPASS_WIDTH = 48;
    private const int COMPASS_HEIGHT = 24;

    // The two compass pictures: the plain one, and the one with every direction lit.
    private const int PLAIN = 0;
    private const int LIT = 1;

    /// Compass directions below this are lit; the rest mean "not walking".
    private const short COMPASS_DIRECTIONS = 9;

    // The item list screen.
    private const int GOLDBASE = 31;
    private const byte OBJECTS_SHEET = 1;
    private const int ITEM_IMAGE_WIDTH = 16;
    private const int ITEM_IMAGE_BYTES = 32;
    private const int ITEM_IMAGE_PLANES = 5;
    private const int ITEM_IMAGE_ROWS = 8000;

    /// The shape memory the object pictures are read from has this many bytes.
    private const int SHAPE_AREA_BYTES = 78000;
    private const int ITEM_ROW_HEIGHT = 80;
    private const int ITEM_X_MARGIN = 20;
    private const uint ALL_PLANES = 0xff;
    private const int BACKGROUND_PEN = 31;

    // The print queue: field widths for printed numbers, and the map-coordinate shift (256 pixels to a sector).
    private const int FAKE_FREE_BYTES = 1_000_000;
    private const int COORD_DIGITS = 6;
    private const int SECTOR_DIGITS = 3;
    private const int EXTENT_DIGITS = 2;
    private const int STAT_DIGITS = 3;
    private const int SECTOR_PIXELS = 256;
    private const int REQUEST_DEBUG_COORDS = 2;
    private const int REQUEST_DEBUG_SECTOR = 3;
    private const int REQUEST_VITALITY = 4;
    private const int REQUEST_OPTIONS = 5;
    private const int REQUEST_STATS = 7;
    private const int REQUEST_TAKE_WHAT = 10;
    private const short QUEUE_MASK = 31;
    private const int VITALITY_X = 245;
    private const int STATS_Y = 52;
    private const int BRAVERY_X = 14;
    private const int LUCK_X = 90;
    private const int KINDNESS_X = 168;
    private const int WEALTH_X = 321;

    // The intro zoom: the picture's shape is 8:5.
    private const int ZOOM_ASPECT_HEIGHT = 5;
    private const int ZOOM_ASPECT_WIDTH = 8;
    private const int ZOOM_FADE_RED = 40;
    private const int ZOOM_FADE_GREEN = 70;
    private const int ZOOM_FADE_BLUE = 100;

    // The win sequence: a sunrise over the winning picture.
    private const int WIN_MESSAGE_FIRST = 6;
    private const int WIN_MESSAGE_SECOND = 7;
    private const int SUNRISE_START = 25;
    private const int SUNRISE_END = -30;
    private const int FIRST_SUNRISE_COLOR = 2;
    private const int LAST_SUNRISE_COLOR = 27;
    private const int SUN_GLOW_UNTIL = -14;
    private const int WIN_SCREEN_SIZE = 156;
    private const int WIN_MESSAGE_PAUSE = 80;
    private const int SUNRISE_FIRST_PAUSE = 60;
    private const int SUNRISE_STEP_PAUSE = 9;
    private const int SUNRISE_END_PAUSE = 30;
    private const ushort WHITE = 0xfff;
    private const int GLOW_FIRST = 29;
    private const int GLOW_SECOND = 30;
    private static readonly int[] WHITE_COLORS = [1, 28];
    private const ushort GLOW_BRIGHT_FIRST = 0x800;
    private const ushort GLOW_BRIGHT_SECOND = 0x400;
    private const int GLOW_FADE_OFFSET = 30;
    private const int RED_STEP = 0x100;

    /// Redraws the compass with direction dir lit. First the unlit compass is copied, then the lit
    /// part for that direction (if any) is copied on top from a second image.
    public static void DrawCompass(Screen screen, short direction)
    {
        CompassPart part = ObjectTables.Comptable[direction];

        screen.CopyCompass(PLAIN, new Rect(0, 0, COMPASS_WIDTH, COMPASS_HEIGHT), (COMPASS_X, COMPASS_Y), COMPASS_PLANE);

        if (direction < COMPASS_DIRECTIONS)
        {
            screen.CopyCompass(LIT, new Rect(part.Xrect, part.Yrect, part.Xsize, part.Ysize), (COMPASS_X + part.Xrect, COMPASS_Y + part.Yrect), COMPASS_PLANE);
        }
    }

    /// Draws every item the hero has, stacked, on a black page.
    public static void DrawItemList(Video video, Hero hero)
    {
        BitmapId page = video.Screen.DrawingBitmapId;

        video.Screen.GetPort(PortId.Map).Target = page;
        video.Screen.Clear(PortId.Map, 0);
        video.Screen.SetBitmap(BitmapId.Items, ItemPictures(video));
        DrawItemIcons(video.Screen, hero, page);
        video.Screen.SetColor(ViewPortId.Page, BACKGROUND_PEN, (0, 0, 0));
    }

    /// Handles one request from the print queue (or waits a tick if there is none). Requests: 2
    /// and 3 debug information, 4 the vitality figure, 5 the menu, 7 the stats line (bravery,
    /// luck, kindness, wealth), 10 "Take What?".
    public static void HandlePrintRequest(Data.Game game, IOutput output)
    {
        PrintQueue queue = game.Ui.PrintQueue;

        if (queue.Write == queue.Read)
        {
            Display.Delay(game, output, 1);

            return;
        }

        int request = queue.Items[queue.Read];

        queue.Read = (short)((queue.Read + 1) & QUEUE_MASK);

        switch (request)
        {
            case REQUEST_DEBUG_COORDS:
                ShowDebugCoords(game.Hero, output);

                break;

            case REQUEST_DEBUG_SECTOR:
                ShowDebugSector(game.World, game.Hero, output);

                break;

            case REQUEST_VITALITY:
                DrawVitality(game.Actors, output);

                break;

            case REQUEST_OPTIONS:
                Menus.PrintOptions(game, output);

                break;

            case REQUEST_STATS:
                DrawStats(game.Hero, output);

                break;

            case REQUEST_TAKE_WHAT:
                output.Print("Take What?");

                break;
        }

        KeepFrameLength(game, output);
    }

    // The original's frame took two ticks when the world had its turn (the print queue routine
    // waited) and only one when it handled a request or when the map scrolled a tile, so the game
    // ran twice as fast for those frames. Remastered and Reimagined wait the missing tick, so
    // every frame is the same length and walking stays at one even speed.
    public static void KeepFrameLength(Data.Game game, IOutput output)
    {
        if (game.Experience != Experience.Original)
        {
            Display.Delay(game, output, 1);
        }
    }

    // The pictures of the objects as one long bitmap: the object sprite sheet holds each plane of
    // an image 32 bytes after the one before, so plane p starts p * 32 bytes into the sheet.
    private static Bitmap ItemPictures(Video video)
    {
        int sheet = video.Graphics.SeqList[OBJECTS_SHEET].Location;
        int available = Math.Max(SHAPE_AREA_BYTES - sheet, 0);
        Bitmap pictures = new(ITEM_IMAGE_WIDTH, ITEM_IMAGE_ROWS, ITEM_IMAGE_PLANES);

        for (int plane = 0; plane < ITEM_IMAGE_PLANES; plane++)
        {
            int start = plane * ITEM_IMAGE_BYTES;
            int bytes = Math.Min(pictures.Plane(plane).Length, Math.Max(available - start, 0));

            video.Graphics.Shape.AsSpan(sheet + start, bytes).CopyTo(pictures.Plane(plane));
        }

        return pictures;
    }

    // Draws one small picture for each item the hero holds (up to the most the screen shows),
    // stacked down the item's column. The pictures are read from Items.
    private static void DrawItemIcons(Screen screen, Hero hero, BitmapId page)
    {
        for (int slot = 0; slot < GOLDBASE; slot++)
        {
            InvItem item = Tables.ItemList[slot];
            int shown = Math.Min(hero.Stuff[slot], (int)item.Maxshown);
            int positionX = item.Xoff + ITEM_X_MARGIN;
            ushort positionY = item.Yoff;
            Rect picture = new(0, item.ImageNumber * ITEM_ROW_HEIGHT + item.ImgOff, ITEM_IMAGE_WIDTH, item.ImgHeight);

            for (int count = 0; count < shown; count++)
            {
                screen.Copy(BitmapId.Items, picture, page, (positionX, positionY), ALL_PLANES);
                positionY = unchecked((ushort)(positionY + item.Ydelta));
            }
        }
    }
}
