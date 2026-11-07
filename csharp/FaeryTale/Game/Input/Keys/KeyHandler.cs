using FaeryTale.Game.Creatures;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Game.Input.Keys;

/// Keys and the hero's controls: HandleKey reacts to one key code from the key buffer (menus,
/// direction keys, the fight key, debug keys).
public static class KeyHandler
{
    // Menu numbers.
    private const short ITEMS = 0;
    private const short SAVEX = 5;
    private const short KEYS = 6;

    /// Number of shortcut letters in MenuShortcuts.
    private const int LETTER_COUNT = 37;

    // A key code with this bit set is the key being released (or the mouse button going up).
    private const int RELEASED = 0x80;
    private const int CODE_MASK = 0x7f;

    // Codes 20-29 are the direction keys.
    private const int FIRST_DIRECTION_KEY = 20;
    private const int LAST_DIRECTION_KEY = 29;

    /// Menu clicks arrive as 'a' for the first entry slot, 'b' for the next and so on.
    private const int FIRST_MENU_CLICK = 0x61;
    private const int FIGHT_KEY = '0';

    /// The key menu's entries start at 5: key 1 is entry 5.
    private const int KEY_ENTRY_BASE = 5;
    private const short FIRST_ACTION_MENU = 5;
    private const int PRQ_OPTIONS = 5;

    // Bits in a menu entry's Enabled byte.
    private const sbyte SELECTED = 1;
    private const sbyte DISPLAYED = 2;
    private const int TYPE_MASK = 0xfc;
    private const int TYPE_TOGGLE = 4;
    private const int TYPE_IMMEDIATE = 8;
    private const int TYPE_RADIO = 12;

    // Debug keys.
    private const int CHEAT_CARRIER = 'B';
    private const int CHEAT_STUFF = '.';
    private const int CHEAT_RESCUE = 'R';
    private const int CHEAT_PRQ_2 = '=';
    private const int CHEAT_PRQ_3 = 19;
    private const int CHEAT_TIME = 18;
    private const int CHEAT_NORTH = 1;
    private const int CHEAT_SOUTH = 2;
    private const int CHEAT_EAST = 3;
    private const int CHEAT_WEST = 4;
    private const ushort CHEAT_STEP_Y = 150;
    private const ushort CHEAT_STEP_X = 280;
    private const ushort CHEAT_TIME_STEP = 1000;
    private const int GOLDBASE = 31;
    private const short CARRIER_BIRD = 11;
    private const short CARRIER_OFFSET = 20;
    private const int RAFT_STONE_SLOT = 5;
    private const int VIAL_SLOT = 22;

    /// The stuff cheat adds this many of a random item; the print requests show debug information.
    private const byte CHEAT_STUFF_COUNT = 3;
    private const int PRQ_DEBUG_COORDS = 2;
    private const int PRQ_DEBUG_SECTOR = 3;

    private const int VIEW_REDRAW = 99;

    /// Handles one key code. notPause is true unless the game is paused. Codes with bit 7 set are
    /// releases. Keys that do nothing in the current state fall through to the menu shortcut
    /// letters at the end.
    public static void HandleKey(Data.Game game, IOutput output, int keyCode, int notPause)
    {
        sbyte key = unchecked((sbyte)keyCode);
        bool running = (short)notPause != 0;

        if (game.Ui.ViewStatus != 0 && running)
        {
            if ((key & RELEASED) == 0)
            {
                game.Ui.ViewStatus = VIEW_REDRAW;
            }
        }
        else if (game.Actors.List[0].State == ActorState.DEAD)
        {
            // Ignored while dead.
        }
        else if (key >= FIRST_DIRECTION_KEY && key <= LAST_DIRECTION_KEY)
        {
            game.Ui.HeldDirection = key;
        }
        else if ((key & CODE_MASK) == game.Ui.HeldDirection)
        {
            game.Ui.HeldDirection = 0;
        }
        else if (key == FIGHT_KEY)
        {
            game.Ui.FightKeyHeld = 1;
        }
        else if ((key & CODE_MASK) == FIGHT_KEY)
        {
            game.Ui.FightKeyHeld = 0;
        }
        else
        {
            HandleOtherKey(game, output, key, running);
        }
    }

    // The keys after the direction and fight keys: cheats, menu clicks, the key menu and shortcut letters.
    private static void HandleOtherKey(Data.Game game, IOutput output, sbyte key, bool running)
    {
        if (CheatItems(game, output, key))
        {
            return;
        }

        if ((key & CODE_MASK) >= FIRST_MENU_CLICK)
        {
            MenuClick(game, output, key, running);
        }
        else if (CheatMove(game, output, key))
        {
            // Handled.
        }
        else if (game.Ui.CurrentMenu == KEYS && (key & RELEASED) == 0)
        {
            PressInKeyMenu(game, output, key);
        }
        else if (key == ' ' || running)
        {
            MenuLetter(game, output, key);
        }
    }

    // A key pressed in the key menu: 1 to 6 choose that key, anything else goes back to the items.
    private static void PressInKeyMenu(Data.Game game, IOutput output, sbyte key)
    {
        if (key >= '1' && key <= '6')
        {
            MenuChoices.ChooseMenuEntry(game, output, (short)(key - '1' + KEY_ENTRY_BASE));
        }
        else
        {
            Menus.GoToMenu(game, output, ITEMS);
        }
    }

    // A click on a menu slot (the mouse sends 'a' + slot for the button going down, and the same
    // with bit 7 set for it coming up).
    private static void MenuClick(Data.Game game, IOutput output, sbyte key, bool running)
    {
        short slot = (short)((key & CODE_MASK) - FIRST_MENU_CLICK);

        game.Ui.ChosenEntry = game.Ui.SlotEntries[slot];
        short chosenEntry = game.Ui.ChosenEntry;
        int menu = game.Ui.CurrentMenu;
        bool valid = chosenEntry >= 0 && chosenEntry < game.Menus[menu].Num;

        if ((key & RELEASED) != 0)
        {
            if (valid)
            {
                Menus.PrintMenuEntry(game, output, slot, 0);
            }

            return;
        }

        // A greyed out box (Remastered and Reimagined) cannot be pressed.
        if (valid && MenuGreying.IsGreyed(game, menu, chosenEntry))
        {
            return;
        }

        game.Ui.MenuHitEnabled = 1;

        if (valid)
        {
            PressEntry(game, output, slot, chosenEntry, running);
        }
    }

    // Presses a valid menu entry according to its type: toggles, immediate actions and radio
    // buttons work even when paused; the rest (menu names) only when not.
    private static void PressEntry(Data.Game game, IOutput output, short slot, short chosenEntry, bool running)
    {
        int menu = game.Ui.CurrentMenu;
        sbyte[] enabled = game.Menus[menu].Enabled;
        int kind = enabled[chosenEntry] & TYPE_MASK;

        if (kind != TYPE_IMMEDIATE)
        {
            InputHandler.ForgetLastMenu();
        }

        if (kind == TYPE_TOGGLE)
        {
            enabled[chosenEntry] ^= SELECTED;
            Menus.PrintMenuEntry(game, output, slot, (short)(enabled[chosenEntry] & SELECTED));
            MenuChoices.ChooseMenuEntry(game, output, chosenEntry);
        }
        else if (kind == TYPE_IMMEDIATE)
        {
            Menus.PrintMenuEntry(game, output, slot, 1);
            MenuChoices.ChooseMenuEntry(game, output, chosenEntry);
        }
        else if (kind == TYPE_RADIO)
        {
            enabled[chosenEntry] |= SELECTED;
            Menus.PrintMenuEntry(game, output, slot, 1);
            MenuChoices.ChooseMenuEntry(game, output, chosenEntry);
        }
        else if (!running)
        {
            // Paused: only the menu types above work.
        }
        else if (kind == 0 && chosenEntry < FIRST_ACTION_MENU)
        {
            game.Ui.CurrentMenu = chosenEntry;
            TextRoutines.QueuePrint(game.Ui, PRQ_OPTIONS);
        }
        else
        {
            Menus.PrintMenuEntry(game, output, slot, (short)(enabled[chosenEntry] & SELECTED));
        }
    }

    // A keyboard shortcut for a menu entry. Letters only work for the Save/Exit menu while it is open.
    private static void MenuLetter(Data.Game game, IOutput output, sbyte key)
    {
        for (int index = 0; index < Math.Min(LETTER_COUNT, Tables.MenuShortcuts.Length); index++)
        {
            Letter shortcut = Tables.MenuShortcuts[index];

            if (shortcut.Key != key)
            {
                continue;
            }

            if (shortcut.Menu == SAVEX && game.Ui.CurrentMenu != SAVEX)
            {
                break;
            }

            if (MenuGreying.BlocksShortcut(game, shortcut.Menu, shortcut.Choice))
            {
                break;
            }

            game.Ui.CurrentMenu = shortcut.Menu;
            game.Ui.ChosenEntry = shortcut.Choice;
            sbyte[] enabled = game.Menus[game.Ui.CurrentMenu].Enabled;

            game.Ui.MenuHitEnabled = (sbyte)(enabled[game.Ui.ChosenEntry] & DISPLAYED);

            if ((enabled[game.Ui.ChosenEntry] & TYPE_MASK) == TYPE_TOGGLE)
            {
                enabled[game.Ui.ChosenEntry] ^= SELECTED;
            }

            MenuChoices.ChooseMenuEntry(game, output, game.Ui.ChosenEntry);
            Menus.PrintOptions(game, output);

            break;
        }
    }

    // Debug keys that give the hero a carrier or items. Returns true if the key was one.
    private static bool CheatItems(Data.Game game, IOutput output, sbyte key)
    {
        if (game.Ui.CheatEnabled == 0)
        {
            return false;
        }

        if (key == CHEAT_CARRIER)
        {
            if (game.Actors.ActiveCarrier == CARRIER_BIRD)
            {
                game.Hero.Stuff[RAFT_STONE_SLOT] = 1;
            }

            Extents.CenterExtentOn(game, 0, (short)((short)game.Hero.X + CARRIER_OFFSET), (short)((short)game.Hero.Y + CARRIER_OFFSET));
            Encounters.LoadCarrier(game, output, CARRIER_BIRD);

            return true;
        }

        if (key == CHEAT_STUFF)
        {
            int slot = game.Rng.Rnd(GOLDBASE);

            game.Hero.Stuff[slot] = unchecked((byte)(game.Hero.Stuff[slot] + CHEAT_STUFF_COUNT));
            Menus.UpdateMenuAvailability(game);
            game.Hero.Stuff[VIAL_SLOT] = 0;

            return true;
        }

        return false;
    }

    // Debug keys that rescue, redraw or move the hero about. Returns true if the key was one.
    private static bool CheatMove(Data.Game game, IOutput output, sbyte key) =>
        game.Ui.CheatEnabled != 0 && (CheatShow(game, output, key) || CheatNudge(game, key));

    // The debug keys that rescue the princess, print debug lines or move the clock on.
    private static bool CheatShow(Data.Game game, IOutput output, sbyte key)
    {
        switch (key)
        {
            case CHEAT_RESCUE:
                Rescue.RescuePrincess(game, output);

                return true;

            case CHEAT_PRQ_2:
                TextRoutines.QueuePrint(game.Ui, PRQ_DEBUG_COORDS);

                return true;

            case CHEAT_PRQ_3:
                TextRoutines.QueuePrint(game.Ui, PRQ_DEBUG_SECTOR);

                return true;

            case CHEAT_TIME:
                game.World.DayClock = unchecked((ushort)(game.World.DayClock + CHEAT_TIME_STEP));

                return true;

            default:
                return false;
        }
    }

    // The debug keys that move the hero (and the map with him) a step in a direction.
    private static bool CheatNudge(Data.Game game, sbyte key)
    {
        (int dx, int dy) = key switch
        {
            CHEAT_NORTH => (0, -CHEAT_STEP_Y),
            CHEAT_SOUTH => (0, CHEAT_STEP_Y),
            CHEAT_EAST => (CHEAT_STEP_X, 0),
            CHEAT_WEST => (-CHEAT_STEP_X, 0),
            _ => (0, 0),
        };

        if (dx == 0 && dy == 0)
        {
            return false;
        }

        Shape hero = game.Actors.List[0];

        hero.AbsX = unchecked((ushort)(hero.AbsX + dx));
        hero.AbsY = unchecked((ushort)(hero.AbsY + dy));
        game.World.MapX = unchecked((ushort)(game.World.MapX + dx));
        game.World.MapY = unchecked((ushort)(game.World.MapY + dy));

        return true;
    }
}
