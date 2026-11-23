using FaeryTale.Game.Data;
using FaeryTale.Game.Creatures.Combat;
using FaeryTale.Game.Creatures.Movement;
using FaeryTale.Game.Files;
using FaeryTale.Game.Input;
using FaeryTale.Game.Input.Keys;
using FaeryTale.Game.Land;
using FaeryTale.Game.Land.TravelDoors;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Reimagined;
using FaeryTale.Game.Render;
using FaeryTale.Game.Testing;

namespace FaeryTale.Game.Startup;

/// The main loop: one pass per frame, reading input, moving the actors, updating the world and drawing.
public static partial class GameLoop
{
    // Screen modes (ViewStatus).
    private const sbyte VIEW_NORMAL = 0;
    private const sbyte VIEW_CHANGED = 3;
    private const sbyte VIEW_BIG = 1;
    private const sbyte VIEW_BIG_STILL = 4;
    private const sbyte VIEW_PAUSED = 2;
    private const sbyte VIEW_REDRAW_2 = 98;
    private const sbyte VIEW_REDRAW = 99;

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

    // One frame: input, moving everything, updating the map and the world, then drawing.
    private static void RunFrame(Data.Game game, Video video, FrameState state)
    {
        ReadInputs(game, video, state);

        if (HandleViewStatus(game, video))
        {
            return;
        }

        InputHandler.ReadWalkDirection(game.Ui, video);

        if (!RunPausedFrame(game, video))
        {
            PlayFrame(game, video, state);
        }
    }

    // The start of a frame: the frame counters, test commands, the keyboard, the map key and the cheat word.
    private static void ReadInputs(Data.Game game, Video video, FrameState state)
    {
        state.Cycle = unchecked((short)(state.Cycle + 1));
        video.Screen.Flasher = unchecked((sbyte)(video.Screen.Flasher + 1));
        TestCommands.RunPendingCommand(game, video); // test commands, does nothing normally

        ReadKeyboard(game, video);
        OpenMapIfAsked(game, video);

        if (CheatCode.TakeCompleted() && game.Experience != Experience.Original)
        {
            CheatCode.GiveEverything(game, video);
        }
    }

    // While the game is paused only orders and the overlay are handled. Returns whether it was paused.
    private static bool RunPausedFrame(Data.Game game, Video video)
    {
        bool paused = PauseMenu.IsPaused(game);

        PauseMenu.Update(game, video, paused);

        if (!paused)
        {
            return false;
        }

        WalkClicks.GiveOrdersWhilePaused(game, video);
        ClickOverlay.Update(game, video);
        Display.Delay(game, video, 1);

        return true;
    }

    // The rest of a running frame: orders, hero, creatures, doors, the map and the drawing.
    private static void PlayFrame(Data.Game game, Video video, FrameState state)
    {
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

    // Fights, missiles and drawing, then shows the page that was just drawn.
    private static void DrawFrame(Data.Game game, Video video, FrameState state, short scrollX, short scrollY)
    {
        FPage page = video.Screen.DrawingPage;

        page.IsvX = game.World.TileX;
        page.IsvY = game.World.TileY;

        video.Screen.SetAPen(PortId.Map, SPRITE_PEN);

        Melee.ResolveMelee(game, video);
        Missiles.AdvanceMissiles(game, video);
        InfernoSkull.Advance(game, video);

        game.Actors.Drawn = game.Actors.Active;
        Objects.SpawnObjectActors(game, video);

        // The flying missiles, then sort the actors by position.
        Missiles.QueueMissileSprites(game);
        InfernoSkull.QueueSprites(game);
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
