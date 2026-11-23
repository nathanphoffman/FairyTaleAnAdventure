using FaeryTale.Game.Data;
using FaeryTale.Game.Creatures;
using FaeryTale.Game.Creatures.Movement;
using FaeryTale.Game.Files;
using FaeryTale.Game.Input.Keys;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Startup;

/// GameLoop: the map view and its scrolling.
public static partial class GameLoop
{
    /// The map's horizontal step as a 10 bit signed number (the map wraps round at the edges).
    internal static short WrapScrollDifference(short difference) =>
        (difference & DIF_SIGN_BIT) != 0 ? unchecked((short)(difference | DIF_SIGN_EXTEND)) : (short)(difference & DIF_MASK);

    /// The scroll direction ScrollMap takes for the map moving one tile by (x, y), if it can scroll.
    internal static int? ScrollDirection(short scrollX, short scrollY) => (scrollX, scrollY) switch
    {
        (1, 1) => SCROLL_UP_LEFT,
        (1, 0) => SCROLL_LEFT,
        (1, -1) => SCROLL_DOWN_LEFT,
        (0, 1) => SCROLL_UP,
        (0, -1) => SCROLL_DOWN,
        (-1, 1) => SCROLL_UP_RIGHT,
        (-1, 0) => SCROLL_RIGHT,
        (-1, -1) => SCROLL_DOWN_RIGHT,
        _ => null,
    };

    // M (Remastered and Reimagined) opens the big map where there is an outdoor world to show.
    private static void OpenMapIfAsked(Data.Game game, Video video)
    {
        if (MapScreen.TakeRequest() && MapSettings.Enabled && game.Experience != Experience.Original && game.World.Region < Regions.FIRST_INDOOR_REGION)
        {
            MapScreen.Show(game, video);
        }
    }

    // Redraws the whole map when the view asks for it (two frames in a row after a change, so both
    // pages get it). Returns true if it did.
    private static bool RedrawWholeMap(Data.Game game, Video video)
    {
        if (game.Ui.ViewStatus != VIEW_REDRAW && game.Ui.ViewStatus != VIEW_REDRAW_2 && game.Ui.ViewStatus != VIEW_CHANGED)
        {
            return false;
        }

        Regions.RefreshMinimap(game, video);
        MapTiles.DrawMap(game, video);

        if (game.Ui.ViewStatus == VIEW_REDRAW)
        {
            game.Ui.ViewStatus = VIEW_REDRAW_2;
        }
        else if (game.Ui.ViewStatus == VIEW_REDRAW_2)
        {
            game.Ui.ViewStatus = VIEW_NORMAL;
        }

        return true;
    }

    // Brings the map in line with the camera: loads a waiting region, then redraws the whole map
    // or scrolls it by one tile. Returns the scroll step used and whether the world gets its turn
    // this frame (it does when the map is redrawn from scratch, or did not move at all).
    private static (short X, short Y, bool WorldUpdate) UpdateMapView(Data.Game game, Video video)
    {
        (short scrollX, short scrollY) = ScrollStep(video.Screen, game.World);

        if (game.World.NewRegion < NO_REGION)
        {
            SpriteLoading.LoadPendingRegion(game, video);
        }

        if (RedrawWholeMap(game, video))
        {
            return (0, 0, true);
        }

        if (scrollX != 0 || scrollY != 0)
        {
            Regions.RefreshMinimap(game, video);
        }

        if (ScrollDirection(scrollX, scrollY) is { } direction)
        {
            MapTiles.ScrollMap(video.Screen, direction);
            ScreenEffects.KeepFrameLength(game, video);

            return (scrollX, scrollY, false);
        }

        if (scrollX == 0 && scrollY == 0)
        {
            // No motion: the world gets its turn here.
            ScreenEffects.HandlePrintRequest(game, video);

            return (scrollX, scrollY, true);
        }

        // Moved too far to scroll: redraw the whole map.
        MapTiles.DrawMap(game, video);
        ScreenEffects.KeepFrameLength(game, video);

        return (0, 0, false);
    }

    // How the map moved since the last frame, in tiles (x wraps at 10 bits).
    private static (short X, short Y) ScrollStep(Screen screen, World world)
    {
        // Pixels to tiles: a shift is a divide by the tile width / height.
        world.TileX = (ushort)(world.MapX >> TILE_WIDTH_SHIFT);
        world.TileY = (ushort)(world.MapY >> TILE_HEIGHT_SHIFT);

        FPage page = screen.DrawingPage;
        short difX = unchecked((short)(world.TileX - page.IsvX));
        short difY = unchecked((short)(world.TileY - page.IsvY));

        return (WrapScrollDifference(difX), difY);
    }
}
