using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Files;

/// Saving and loading the game from the menu. Games are named A.faery, B.faery ... (one letter per
/// menu choice) and the platform layer puts the files in the saves folder.
public static class SaveGame
{
    /// Value of ViewStatus meaning the screen must be redrawn from scratch.
    private const sbyte VIEW_CORRUPT = 99;

    // Print queue requests that redraw the vitality line and the stats line.
    private const int PRINT_VITALITY = 4;
    private const int PRINT_STATS = 7;

    /// Saves the game into slot slot (0 = A ...) when saving, or loads it from there.
    public static void Run(Data.Game game, IOutput output, short slot, bool saving)
    {
        output.UsePanelFont();

        string name = SaveFiles.NameForSlot(slot);

        try
        {
            if (saving)
            {
                SaveFiles.Write(game, name);
            }
            else
            {
                SaveFiles.Read(game, name);
            }
        }
        catch (IOException)
        {
            output.Print(saving ? "ERROR: Couldn't save game." : "ERROR: Couldn't load game.");
        }

        if (!saving)
        {
            ResetAfterLoad(game, output);
        }

        output.UseStoryFont();
    }

    // After loading a game: clears the encounter state and redraws the status panel for the new hero.
    private static void ResetAfterLoad(Data.Game game, IOutput output)
    {
        game.Encounter.WeaponType = 0;
        game.Encounter.Number = 0;
        SpriteLoading.LoadStartSprites(game, output);
        Menus.UpdateMenuAvailability(game);
        game.Ui.ViewStatus = VIEW_CORRUPT;
        TextRoutines.QueuePrint(game.Ui, PRINT_VITALITY);
        TextRoutines.QueuePrint(game.Ui, PRINT_STATS);
        output.Print("");
        output.Print("");
        output.Print("");
        game.Encounter.Kind = 0;
        game.Actors.Loading = 0;
    }
}
