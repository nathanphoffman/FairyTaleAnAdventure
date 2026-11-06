using FaeryTale.Game.Creatures.Ai;
using FaeryTale.Game.Data;
using FaeryTale.Game.Creatures.Combat;
using FaeryTale.Game.Creatures;
using FaeryTale.Game.Creatures.Movement;
using FaeryTale.Game.Files;
using FaeryTale.Game.Input;
using FaeryTale.Game.Input.Keys;
using FaeryTale.Game.Land;
using FaeryTale.Game.Land.TravelDoors;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Game.Testing;

namespace FaeryTale.Game.Startup;

/// The main loop: one pass per frame, reading input, moving the actors, updating the world and drawing.
public static class GameLoop
{
    // Screen modes (ViewStatus).
    private const sbyte VIEW_NORMAL = 0;
    private const sbyte VIEW_CHANGED = 3;
    private const sbyte VIEW_BIG = 1;
    private const sbyte VIEW_BIG_STILL = 4;
    private const sbyte VIEW_PAUSED = 2;
    private const sbyte VIEW_REDRAW_2 = 98;
    private const sbyte VIEW_REDRAW = 99;

    // The game menu entry that pauses the game; bit 0 set means paused.
    private const int GAME_MENU = 4;
    private const int PAUSE_ENTRY = 5;

    /// A region number below this means the next region is still being loaded.
    private const ushort NO_REGION = 10;

    // The colour of the flashing pointer and the sprite pen.
    private const uint FLASH_COLOR = 15;
    private const byte SPRITE_PEN = 31;

    /// Waits, in fiftieths of a second.
    private const int PAUSE_WAIT = 200;

    // How the horizontal scroll step wraps: a 10 bit signed value.
    private const int DIF_SIGN_BIT = 0x200;
    private const int DIF_SIGN_EXTEND = 0xfc00;
    private const int DIF_MASK = 0x03ff;

    // The scroll directions ScrollMap takes, by (difX, difY).
    internal const int SCROLL_UP_LEFT = 5;
    internal const int SCROLL_LEFT = 4;
    internal const int SCROLL_DOWN_LEFT = 3;
    internal const int SCROLL_UP = 6;
    internal const int SCROLL_DOWN = 2;
    internal const int SCROLL_UP_RIGHT = 7;
    internal const int SCROLL_RIGHT = 0;
    internal const int SCROLL_DOWN_RIGHT = 1;

    // Tiles are 16 pixels wide and 32 high: the shifts that turn map positions into tile positions,
    // and the masks of the part of a tile the map is offset by.
    private const int TILE_WIDTH_SHIFT = 4;
    private const int TILE_HEIGHT_SHIFT = 5;
    private const int TILE_X_MASK = 15;
    private const int TILE_Y_MASK = 31;

    /// The status colour (31) flashes on and off with this bit of the flasher counter.
    private const sbyte FLASH_BIT = 16;
    private const int STATUS_COLOR = 31;

    /// What the frame loop carries from one frame to the next.
    private sealed class FrameState
    {
        /// The frame counter (the original left it uninitialised).
        public short Cycle;

        /// The terrain of the last actor moved. It carries over to the next actor (a dead actor
        /// reads it without setting it), and from the drawing loop to the next frame.
        public short LastTerrain;
    }

    /// The main loop: one pass per frame until the player quits.
    public static void Run(Data.Game game, Video video)
    {
        FrameState state = new();

        game.Ui.CheatEnabled = 0;
        game.Ui.QuitRequested = 0;

        while (game.Ui.QuitRequested == 0)
        {
            RunFrame(game, video, state);
        }
    }

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

    // One frame: input, moving everything, updating the map and the world, then drawing.
    private static void RunFrame(Data.Game game, Video video, FrameState state)
    {
        state.Cycle = unchecked((short)(state.Cycle + 1));
        video.Screen.Flasher = unchecked((sbyte)(video.Screen.Flasher + 1));
        TestCommands.RunPendingCommand(game, video); // test commands, does nothing normally

        ReadKeyboard(game, video);
        OpenMapIfAsked(game, video);

        if (HandleViewStatus(game, video))
        {
            return;
        }

        InputHandler.ReadWalkDirection(game.Ui, video);

        bool paused = (game.Menus[GAME_MENU].Enabled[PAUSE_ENTRY] & 1) != 0;

        PauseMenu.Update(game, video, paused);

        if (paused)
        {
            ClickToMove.GiveOrdersWhilePaused(game);
            ClickOverlay.Update(game, video);
            Display.Delay(game, video, 1);

            return;
        }

        ClickToMove.Drive(game, video);
        ClickOverlay.Update(game, video);
        MenuGreying.Refresh(game, video);
        video.ShowAskedMessageScroll();
        HeroControls.UpdateHero(game, video, state.Cycle);
        MoveEverything(game, video, state);
        Bed.CheckBed(game, video);
        TravelDoors.CheckDoor(game, video);

        game.Hero.X = game.Actors.List[0].AbsX;
        game.Hero.Y = game.Actors.List[0].AbsY;

        ActorOrder.UndoSprites(game, video);
        (short scrollX, short scrollY, bool worldUpdate) = UpdateMapView(game, video);

        if (worldUpdate)
        {
            RunWorldUpdate(game, video);
        }

        DrawFrame(game, video, state, scrollX, scrollY);
    }

    // M (Remastered and Reimagined) opens the big map where there is an outdoor world to show.
    private static void OpenMapIfAsked(Data.Game game, Video video)
    {
        if (MapScreen.TakeRequest() && MapSettings.Enabled && game.Experience != Experience.Original && game.World.Region < Regions.FIRST_INDOOR_REGION)
        {
            MapScreen.Show(game, video);
        }
    }

    // Reads one key and handles it.
    private static void ReadKeyboard(Data.Game game, IOutput output)
    {
        sbyte key = unchecked((sbyte)InputHandler.NextKey());
        bool notPause = (game.Menus[GAME_MENU].Enabled[PAUSE_ENTRY] & 1) == 0;

        if (key == InputHandler.ESCAPE_KEY && output is Video video)
        {
            PressEscape(game, video);
        }
        else if (key != 0)
        {
            KeyHandler.HandleKey(game, output, key, notPause ? 1 : 0);
        }
    }

    // Escape leaves this game for the game selection. Remastered and Reimagined ask first with the
    // game menu, which can also resume or leave the program.
    private static void PressEscape(Data.Game game, Video video)
    {
        GameMenuChoice choice = game.Experience == Experience.Original ? GameMenuChoice.BackToGameSelection : GameMenu.Show(game, video);

        if (choice == GameMenuChoice.ExitGame)
        {
            game.Ui.ExitProgram = true;
        }

        if (choice != GameMenuChoice.Resume)
        {
            game.Ui.QuitRequested = 1;
        }
    }

    // The screen modes that stop a normal frame. Returns true if this frame ends here (the item
    // list and the like only flash the pointer and handle the print queue).
    private static bool HandleViewStatus(Data.Game game, Video video)
    {
        if (game.Ui.ViewStatus == VIEW_PAUSED)
        {
            Display.Delay(game, video, PAUSE_WAIT);
            game.Ui.ViewStatus = VIEW_REDRAW;
        }
        else if (game.Ui.ViewStatus == VIEW_BIG || game.Ui.ViewStatus == VIEW_BIG_STILL)
        {
            bool flashOn = (video.Screen.Flasher & FLASH_BIT) != 0 && game.Ui.ViewStatus == VIEW_BIG;

            video.Screen.SetColor(ViewPortId.Page, STATUS_COLOR, flashOn ? (FLASH_COLOR, FLASH_COLOR, FLASH_COLOR) : (0u, 0u, 0u));
            ScreenEffects.HandlePrintRequest(game, video);

            return true;
        }

        return false;
    }

    // Moves the hero, the raft and every monster and person (the actor count is read each time
    // round, as in the original).
    private static void MoveEverything(Data.Game game, IOutput output, FrameState state)
    {
        Carriers.RaftProximity(game);

        for (int index = 0; index < game.Actors.Active; index++)
        {
            ActorMovement.MoveActor(game, output, index, state.Cycle, ref state.LastTerrain);
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

    // The once-per-frame world update: the clock, then (unless time is frozen) new monsters,
    // talking, what the monsters want to do, and hunger.
    private static void RunWorldUpdate(Data.Game game, IOutput output)
    {
        Clock.DayNightTick(game, output);

        if (game.World.FreezeTimer == 0)
        {
            Spawning.SpawnEncounters(game, output);
            NearbySpeech.SpeakNearby(game, output);
            EnemyGoals.UpdateEnemyGoals(game, output);
            Survival.SurvivalTick(game, output);
        }
    }

    // Fights, missiles and drawing, then shows the page that was just drawn.
    private static void DrawFrame(Data.Game game, Video video, FrameState state, short scrollX, short scrollY)
    {
        FPage page = video.Screen.DrawingPage;

        page.IsvX = game.World.TileX;
        page.IsvY = game.World.TileY;

        video.Screen.SetAPen(PortId.Map, SPRITE_PEN);

        Melee.ResolveMelee(game, video);
        Missiles.AdvanceMissiles(game, video);

        game.Actors.Drawn = game.Actors.Active;
        Objects.SpawnObjectActors(game, video);

        // The flying missiles, then sort the actors by position.
        Missiles.QueueMissileSprites(game);
        ActorOrder.SortActors(game);

        // Now that the scroll blit is done, finish repairing the map.
        MapEdge.RepairEdges(game, video, scrollX, scrollY);
        SpriteDrawing.DrawSprites(game, video, state.Cycle);

        // The drawing loop used to leave j here; the next frame's actor loop reads it.
        state.LastTerrain = game.Actors.Drawn;
        int ras = video.Screen.DrawingPage.Ras;

        video.Screen.Ras[ras].X = (short)(game.World.MapX & TILE_X_MASK);
        video.Screen.Ras[ras].Y = (short)(game.World.MapY & TILE_Y_MASK);
        Display.FlipPages(game, video);

        if (game.Ui.ViewStatus == VIEW_CHANGED)
        {
            Fading.FadeNormal(game, video);
            game.Ui.ViewStatus = VIEW_NORMAL;
        }
    }
}
