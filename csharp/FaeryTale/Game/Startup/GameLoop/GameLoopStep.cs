using FaeryTale.Game.Creatures.Ai;
using FaeryTale.Game.Data;
using FaeryTale.Game.Creatures;
using FaeryTale.Game.Creatures.Movement;
using FaeryTale.Game.Input;
using FaeryTale.Game.Input.Keys;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Startup;

/// GameLoop: the keyboard and the world update of a frame.
public static partial class GameLoop
{
    // Reads one key and handles it.
    private static void ReadKeyboard(Data.Game game, IOutput output)
    {
        sbyte key = unchecked((sbyte)InputHandler.NextKey());
        bool notPause = !PauseMenu.IsPaused(game);

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
}
