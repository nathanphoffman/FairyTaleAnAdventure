using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Game.Remastered;

/// What the hero has and what a click on an item does. Equipping and using go through the game's
/// own menus (the Use, Magic and Keys menus), so every rule stays in one place.
public static class InventoryActions
{
    // The menus the choices belong to, and the entries of each.
    private const int KEYS_MENU = 6;
    private const int USE_MENU = 8;
    private const int FIRST_ITEM_ENTRY = 5;
    private const int FIRST_KEY_SLOT = 16;
    private const int SEA_SHELL = 6;
    private const int SUN_STONE = 7;
    private const short SEA_SHELL_ENTRY = 6;
    private const short SUN_STONE_ENTRY = 8;

    /// The weapon numbers are the slot plus one (the dirk is 1, the magic wand 5).
    private const int WEAPON_COUNT = 5;

    /// How many of an item the hero has.
    public static int CountOf(Data.Game game, int slot) => game.Hero.Stuff[slot];

    /// Whether an item is the weapon in hand.
    public static bool IsEquipped(Data.Game game, int slot) =>
        slot < WEAPON_COUNT && game.Actors.List[0].Weapon == slot + 1;

    /// The words for what a click does, for the line under the squares.
    public static string HintFor(InventoryItem item) => item.Use switch
    {
        ItemUse.Equip => "Click to equip",
        ItemUse.Cast => "To cast, use the hotbar",
        ItemUse.Unlock => "Click to try it on a door or chest",
        ItemUse.UseSpecial => "Click to use",
        _ => "",
    };

    /// Does what a click on an item says, if the hero has it.
    public static void Use(Data.Game game, IOutput output, InventoryItem item)
    {
        if (CountOf(game, item.Slot) == 0)
        {
            return;
        }

        switch (item.Use)
        {
            case ItemUse.Equip:
                game.Actors.List[0].Weapon = (sbyte)(item.Slot + 1);
                break;

            // A magic stone is not cast from here: the hotbar casts it (so there is no click to use up).
            case ItemUse.Cast:
                break;

            case ItemUse.Unlock:
                ChooseIn(game, output, KEYS_MENU, (short)(FIRST_ITEM_ENTRY + item.Slot - FIRST_KEY_SLOT));
                break;

            case ItemUse.UseSpecial:
                UseSpecial(game, output, item.Slot);
                break;
        }
    }

    // The sea shell and the sun stone are entries of the Use menu.
    private static void UseSpecial(Data.Game game, IOutput output, int slot)
    {
        if (slot == SEA_SHELL)
        {
            ChooseIn(game, output, USE_MENU, SEA_SHELL_ENTRY);
        }
        else if (slot == SUN_STONE)
        {
            ChooseIn(game, output, USE_MENU, SUN_STONE_ENTRY);
        }
    }

    // Makes a choice in one of the game's menus as if it had been clicked there, then puts the panel's
    // menu back the way it was. The menus only act on entries that are lit, and the hero has the item.
    private static void ChooseIn(Data.Game game, IOutput output, int menu, short entry)
    {
        short before = game.Ui.CurrentMenu;

        game.Ui.CurrentMenu = (short)menu;
        game.Ui.MenuHitEnabled = 1;
        MenuChoices.ChooseMenuEntry(game, output, entry);
        game.Ui.CurrentMenu = before;
        Menus.PrintOptions(game, output);
    }
}
