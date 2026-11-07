using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Files;
using FaeryTale.Game.Land;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Ui.Options;

/// What choosing an entry of a menu does: the Items and Magic menus (in ItemsMenu and Magic), and
/// here talking, buying, the game menu, using items, keys, saving and giving.
public static class MenuChoices
{
    // Menu numbers.
    private const int ITEMS = 0;
    private const int MAGIC = 1;
    private const int TALK = 2;
    private const int BUY = 3;
    private const int GAME = 4;
    private const int SAVEX = 5;
    private const int KEYS = 6;
    private const int GIVE = 7;
    private const int USE = 8;
    private const int FILE = 9;

    // Inventory slots.
    internal const int KEYBASE = 16;
    internal const int ARROWS = 8;
    internal const int BONE = 29;

    // Print queue entries (the numbers; the names differ between places in the original).
    internal const int PRQ_VITALITY_LINE = 4;
    internal const int PRQ_STATS_LINE = 7;

    // Game menu entries.
    private const short MUSIC = 6;
    private const short QUIT = 8;
    private const short LOAD = 9;

    // Save/exit menu entries.
    private const short SAVE = 5;
    private const short EXIT = 6;

    // Use menu entries.
    private const short KEY_ENTRY = 7;
    private const short SHELL_ENTRY = 6;
    private const short SUNSTONE_ENTRY = 8;
    private const short WEAPONS = 5;
    private const int SUNSTONE_SPEECH = 60;

    // No summoning the turtle inside this box (the swamp it lives in is elsewhere).
    private const ushort NO_TURTLE_X_MIN = 11194;
    private const ushort NO_TURTLE_X_MAX = 21373;
    private const ushort NO_TURTLE_Y_MIN = 10205;
    private const ushort NO_TURTLE_Y_MAX = 16208;

    // Keys: the nine directions to try a door in.
    private const int DIRECTIONS = 9;
    private const int DOOR_REACH = 16;

    // Give menu.
    private const short GIVE_GOLD = 5;
    private const short GIVE_BONE = 8;
    private const short GOLD_GIVEN = 2;
    private const byte BEGGAR_RACE = 0x8d;
    private const byte SPECTRE_RACE = 0x8a;
    private const int BEGGAR_SPEECH = 24;
    private const int BEGGAR_THANKS = 50;
    private const int SPECTRE_NOT_HERE = 21;
    private const int SPECTRE_THANKS = 48;
    private const short SPECTRE_BONE_DROP = 140;

    /// Does what menu entry chosenEntry of the current menu says.
    public static void ChooseMenuEntry(Data.Game game, IOutput output, short chosenEntry)
    {
        bool refresh = game.Ui.CurrentMenu switch
        {
            ITEMS => ItemsAndRefresh(game, output, chosenEntry),
            MAGIC => Magic.MagicOption(game, output, chosenEntry),
            TALK => TalkOptions.TalkOption(game, output, chosenEntry),
            BUY => ShopOptions.BuyOption(game, output, chosenEntry),
            GAME => GameOption(game, output, chosenEntry),
            USE => UseOption(game, output, chosenEntry),
            SAVEX => SaveExitOption(game, output, chosenEntry),
            FILE => FileOption(game, output, chosenEntry),
            KEYS => KeysOption(game, output, chosenEntry),
            GIVE => GiveOption(game, output, chosenEntry),
            _ => true,
        };

        if (refresh)
        {
            Menus.UpdateMenuAvailability(game);
        }
    }

    private static bool ItemsAndRefresh(Data.Game game, IOutput output, short chosenEntry)
    {
        ItemsMenu.ItemsOption(game, output, chosenEntry);

        return true;
    }

    // Music on/off, and going to the save/exit and load menus.
    private static bool GameOption(Data.Game game, IOutput output, short chosenEntry)
    {
        if (chosenEntry == MUSIC)
        {
            Music.SetMood(game, 1);
        }

        if (chosenEntry == QUIT)
        {
            Menus.GoToMenu(game, output, SAVEX);
        }

        if (chosenEntry == LOAD)
        {
            game.Ui.Saving = false;
            Menus.GoToMenu(game, output, FILE);
        }

        return true;
    }

    // Saving goes to the file menu; exiting ends the game.
    private static bool SaveExitOption(Data.Game game, IOutput output, short chosenEntry)
    {
        if (chosenEntry == EXIT)
        {
            game.Ui.QuitRequested = 1;
        }

        if (chosenEntry == SAVE)
        {
            game.Ui.Saving = true;
            Menus.GoToMenu(game, output, FILE);
        }

        return true;
    }

    // A save slot was chosen: save or load, then back to the game menu.
    private static bool FileOption(Data.Game game, IOutput output, short chosenEntry)
    {
        SaveGame.Run(game, output, chosenEntry, game.Ui.Saving);
        Menus.GoToMenu(game, output, GAME);

        return true;
    }

    // Picks a weapon, the key menu, the turtle shell, or the sunstone.
    private static bool UseOption(Data.Game game, IOutput output, short chosenEntry)
    {
        if (chosenEntry == KEY_ENTRY)
        {
            if (game.Ui.MenuHitEnabled != 0)
            {
                Menus.GoToMenu(game, output, KEYS);
            }
            else
            {
                Messages.PrintWrapped(game, output, "% has no keys!"u8);
            }

            return false;
        }

        if (chosenEntry < WEAPONS)
        {
            if (game.Ui.MenuHitEnabled != 0)
            {
                game.Actors.List[0].Weapon = (sbyte)(chosenEntry + 1);
            }
            else
            {
                Messages.PrintWrapped(game, output, "% doesn't have one."u8);
            }
        }

        if (chosenEntry == SHELL_ENTRY && game.Ui.MenuHitEnabled != 0)
        {
            // The original keeps the shell from working in the middle of the land; Remastered and Reimagined let it
            // work anywhere there is deep water near.
            if (game.Experience == Data.Experience.Original && InNoTurtleArea(game.Hero))
            {
                return true;
            }

            Extents.SummonTurtle(game, output);
        }

        if (chosenEntry == SUNSTONE_ENTRY && game.Witch.OnScreen != 0)
        {
            TextRoutines.Speak(game, output, SUNSTONE_SPEECH);
        }

        Menus.GoToMenu(game, output, ITEMS);

        return true;
    }

    // Tries a key on every door and chest around the hero.
    private static bool KeysOption(Data.Game game, IOutput output, short chosenEntry)
    {
        int key = chosenEntry - ShopOptions.FIRST_MENU_ENTRY;

        game.World.BumpedLockedDoor = 0;

        if (game.Hero.Stuff[key + KEYBASE] != 0)
        {
            int direction = 0;

            while (direction < DIRECTIONS)
            {
                ushort x = (ushort)MapSteps.StepX(game.Hero.X, direction, DOOR_REACH);
                ushort y = (ushort)MapSteps.StepY(game.Hero.Y, direction, DOOR_REACH);

                if (KeyDoors.OpenDoorNear(game, output, x, y, (uint)key + 1) != 0)
                {
                    game.Hero.Stuff[key + KEYBASE]--;

                    break;
                }

                direction++;
            }

            if (direction >= DIRECTIONS)
            {
                Messages.PrintWrapped(game, output, "% tried a "u8);
                output.PrintCont(ItemsMenu.ItemName(KEYBASE + key));
                output.PrintCont(" but it didn't");
                output.Print("fit.");
            }
        }

        Menus.GoToMenu(game, output, ITEMS);

        return true;
    }

    // Gives gold to a beggar, or the bone to the spectre.
    private static bool GiveOption(Data.Game game, IOutput output, short chosenEntry)
    {
        if (game.Actors.NearestPerson == 0)
        {
            return true;
        }

        Data.Shape person = game.Actors.List[game.Actors.NearestPerson];

        if (chosenEntry == GIVE_GOLD && game.Hero.Wealth > GOLD_GIVEN)
        {
            game.Hero.Wealth -= GOLD_GIVEN;

            if (game.Rng.Rand64() > game.Hero.Kindness)
            {
                game.Hero.Kindness++;
            }

            TextRoutines.QueuePrint(game.Ui, PRQ_VITALITY_LINE);
            TextRoutines.QueuePrint(game.Ui, PRQ_STATS_LINE);
            TextRoutines.Speak(game, output, person.Race == BEGGAR_RACE ? BEGGAR_SPEECH + person.Goal : BEGGAR_THANKS);
        }
        else if (chosenEntry == GIVE_BONE && game.Hero.Stuff[BONE] != 0)
        {
            if (person.Race != SPECTRE_RACE)
            {
                TextRoutines.Speak(game, output, SPECTRE_NOT_HERE);
            }
            else
            {
                TextRoutines.Speak(game, output, SPECTRE_THANKS);
                game.Hero.Stuff[BONE] = 0;
                Objects.LeaveItem(game.Objects, game.Actors, game.Actors.NearestPerson, SPECTRE_BONE_DROP);
            }
        }

        Menus.GoToMenu(game, output, ITEMS);

        return true;
    }

    // Whether the hero is in the box where the turtle cannot be called.
    private static bool InNoTurtleArea(Data.Hero hero) =>
        hero.X < NO_TURTLE_X_MAX && hero.X > NO_TURTLE_X_MIN && hero.Y < NO_TURTLE_Y_MAX && hero.Y > NO_TURTLE_Y_MIN;
}
