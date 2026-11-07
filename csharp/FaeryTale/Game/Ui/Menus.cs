using FaeryTale.Game.Reimagined;
using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Game.Ui;

/// The text-panel menus: which entries are switched on, and changing menu.
///
/// The game has 10 menus; CurrentMenu is the one being shown. Each entry's Enabled byte says how
/// it is drawn: 8 is greyed out, 10 is available, and other values come from whether the hero has
/// the matching item (ItemMenuState).
public static class Menus
{
    // Menu numbers.
    private const int MAGIC = 1;
    private const int GAME = 4;
    private const int SAVEX = 5;
    private const int KEYS = 6;
    private const int GIVE = 7;
    private const int USE = 8;
    private const int FILE = 9;

    // An entry that is shown but cannot be used, and one that can.
    private const sbyte GREYED = 8;
    private const sbyte AVAILABLE = 10;

    /// The game menu entry that pauses the game; bit 0 set means paused.
    private const int PAUSE_ENTRY = 5;

    // Inventory slots, counted from the start of the item counts, for each use/give entry.
    private const int MAGIC_ITEM_BASE = 9;
    private const int KEY_ITEM_BASE = 16;
    private const int SUNSTONE = 7;
    private const int WRIT = 28;
    private const int BONE = 29;

    // The first entries that depend on items, in each menu (the earlier ones are fixed commands),
    // and how many items each menu has.
    private const int ITEM_ENTRIES_START = 5;
    private const int MAGIC_ITEMS = 7;
    private const int KEY_ITEMS = 6;

    // Entries of the use and give menus.
    private const int USE_KEY_ENTRY = 7;
    private const int USE_SUNSTONE_ENTRY = 8;
    private const int GIVE_GOLD_ENTRY = 5;
    private const int GIVE_BOOK_ENTRY = 6;
    private const int GIVE_WRIT_ENTRY = 7;
    private const int GIVE_BONE_ENTRY = 8;

    /// The hero must have more than this much gold to be able to give some away.
    private const short GIVE_GOLD_MIN = 2;

    /// Most entries a menu shows at once (two columns of six).
    private const int SLOTS = 12;

    /// An entry with this bit is shown.
    private const sbyte SHOWN = 2;

    /// Each label is 5 characters.
    private const int LABEL_LENGTH = 5;

    /// The first 5 entries of every menu are the main menu names, taken from Label1.
    private const int MAIN_ENTRIES = 5;
    private const byte FILE_PEN = 13;
    private const byte USE_PEN = 14;
    private const byte MAIN_PEN = 4;

    /// Switches the use, magic, keys and give entries on or off to match what the hero is carrying.
    public static void UpdateMenuAvailability(Data.Game game)
    {
        for (int index = 0; index < MAGIC_ITEMS; index++)
        {
            game.Menus[MAGIC].Enabled[index + ITEM_ENTRIES_START] = (sbyte)TextRoutines.ItemMenuState(game.Hero, index + MAGIC_ITEM_BASE);
            game.Menus[USE].Enabled[index] = (sbyte)TextRoutines.ItemMenuState(game.Hero, index);
        }

        // "Use a key" is available if any key slot reports available.
        sbyte keys = GREYED;

        for (int index = 0; index < KEY_ITEMS; index++)
        {
            sbyte flag = (sbyte)TextRoutines.ItemMenuState(game.Hero, index + KEY_ITEM_BASE);

            game.Menus[KEYS].Enabled[index + ITEM_ENTRIES_START] = flag;

            if (flag == AVAILABLE)
            {
                keys = AVAILABLE;
            }
        }

        game.Menus[USE].Enabled[USE_KEY_ENTRY] = keys;
        game.Menus[USE].Enabled[USE_SUNSTONE_ENTRY] = (sbyte)TextRoutines.ItemMenuState(game.Hero, SUNSTONE);
        game.Menus[GIVE].Enabled[GIVE_GOLD_ENTRY] = game.Hero.Wealth > GIVE_GOLD_MIN ? AVAILABLE : GREYED;
        game.Menus[GIVE].Enabled[GIVE_BOOK_ENTRY] = GREYED;
        game.Menus[GIVE].Enabled[GIVE_WRIT_ENTRY] = (sbyte)TextRoutines.ItemMenuState(game.Hero, WRIT);
        game.Menus[GIVE].Enabled[GIVE_BONE_ENTRY] = (sbyte)TextRoutines.ItemMenuState(game.Hero, BONE);
    }

    /// Draws the current menu: the entries that are switched on, in the order they are shown, and
    /// blanks over the slots that are left. SlotEntries remembers which entry each slot is.
    public static void PrintOptions(Data.Game game, IOutput output)
    {
        Menu menu = game.Menus[game.Ui.CurrentMenu];
        int slot = 0;

        for (int entry = 0; entry < menu.Num; entry++)
        {
            sbyte enabled = menu.Enabled[entry];

            if ((enabled & SHOWN) == 0)
            {
                continue;
            }

            game.Ui.SlotEntries[slot] = (sbyte)entry;
            PrintMenuEntry(game, output, (short)slot, (short)(enabled & 1));
            slot++;

            if (slot > SLOTS - 1)
            {
                break;
            }
        }

        BlankSlotsFrom(game.Ui, output, slot);
    }

    /// Switches to menu mode and redraws it, unless the game is paused.
    public static void GoToMenu(Data.Game game, IOutput output, short mode)
    {
        if ((game.Menus[GAME].Enabled[PAUSE_ENTRY] & 1) != 0)
        {
            return;
        }

        game.Ui.CurrentMenu = mode;
        InputHandler.ForgetLastMenu();
        PrintOptions(game, output);
    }

    /// Draws entry slot j (0-11) of the current menu in the panel. pena is the text colour: 1 for
    /// a selected entry, 0 for a plain one. The box colour depends on the menu.
    public static void PrintMenuEntry(Data.Game game, IOutput output, short j, short pena)
    {
        int entry = game.Ui.SlotEntries[j];
        byte boxPen = EntryBoxPen(game, entry);
        byte[] label = EntryLabel(game, entry * LABEL_LENGTH);

        output.DrawMenuEntry(j, label, pena != 0, boxPen);
    }

    // Blanks the slots from first to the last one: nothing is shown there.
    private static void BlankSlotsFrom(Data.Ui ui, IOutput output, int first)
    {
        for (int slot = first; slot < SLOTS; slot++)
        {
            output.BlankMenuEntry(slot);
            ui.SlotEntries[slot] = -1;
        }
    }

    // The colour of the box behind an entry of the current menu: fixed for the use and file menus
    // and the main names, from a table for keys and the save slots, else the menu's own.
    private static byte EntryBoxPen(Data.Game game, int entry)
    {
        int menu = game.Ui.CurrentMenu;

        if (MenuGreying.IsGreyed(game, menu, entry))
        {
            return MenuGreying.GREY_PEN;
        }

        if (menu == USE)
        {
            return USE_PEN;
        }

        if (menu == FILE)
        {
            return FILE_PEN;
        }

        if (entry < MAIN_ENTRIES)
        {
            return MAIN_PEN;
        }

        if (menu == KEYS)
        {
            return (byte)Tables.KeyColors[entry - MAIN_ENTRIES];
        }

        return menu == SAVEX ? (byte)entry : (byte)game.Menus[menu].Color;
    }

    // The label text offset bytes into the current menu's labels. The main names come from Label1;
    // the other menus have their own list, which for the first 5 slots is skipped over.
    private static byte[] EntryLabel(Data.Game game, int offset)
    {
        int current = game.Ui.CurrentMenu;
        byte[] labels = InfernoSkull.MenuLabels(game, game.Menus[current].LabelList);
        byte[] list = labels;
        int from = offset;

        if (current < USE)
        {
            if (offset < MAIN_ENTRIES * LABEL_LENGTH)
            {
                list = Tables.Label1;
            }
            else
            {
                from = offset - MAIN_ENTRIES * LABEL_LENGTH;
            }
        }

        if (from < 0 || from + LABEL_LENGTH > list.Length)
        {
            return Enumerable.Repeat((byte)' ', LABEL_LENGTH).ToArray();
        }

        return list[from..(from + LABEL_LENGTH)];
    }
}
