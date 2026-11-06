using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Ui;

/// The Items menu actions: showing the item list, taking what is nearby (objects, containers,
/// dead bodies) and looking around.
public static class ItemsMenu
{
    // Menu numbers and the entries of the Items menu.
    private const short USE = 8;
    private const short GIVE = 7;
    private const short LIST = 5;
    private const short TAKE = 6;
    private const short LOOK = 7;
    private const short USE_ENTRY = 8;
    private const short GIVE_ENTRY = 9;

    // Inventory slots.
    internal const int GOLDBASE = 31;
    internal const int ARROWBASE = 35;
    internal const int KEYBASE = 16;
    internal const int ARROWS = 8;

    // View status values while the item list is shown, and the end of the game.
    private const sbyte VIEW_ITEMS = 4;
    internal const sbyte VIEW_GAME_OVER = 2;
    private const int PRQ_LIST = 5;

    /// Look finds hidden objects closer than this.
    private const int LOOK_RANGE = 40;
    private const int EVENT_NOTHING_HIDDEN = 20;
    private const int EVENT_FOUND_HIDDEN = 38;

    /// The name of inventory item index.
    public static string ItemName(int index) => Tables.ItemList[index].Name;

    /// Adds amount to the hero's count of inventory slot slot.
    public static void AddStuff(Data.Game game, int slot, byte amount) =>
        game.Hero.Stuff[slot] = unchecked((byte)(game.Hero.Stuff[slot] + amount));

    /// Handles an entry of the Items menu.
    public static void ItemsOption(Data.Game game, IOutput output, short chosenEntry)
    {
        switch (chosenEntry)
        {
            case USE_ENTRY:
                Menus.GoToMenu(game, output, USE);
                break;

            case LIST:
                ShowItemList(game, output);
                break;

            case TAKE:
                TakeItems.TakeNearest(game, output);
                break;

            case LOOK:
                LookAround(game, output);
                break;

            case GIVE_ENTRY:
                Menus.GoToMenu(game, output, GIVE);
                break;
        }
    }

    // Draws every item the hero has, stacked, on a black page and shows it.
    private static void ShowItemList(Data.Game game, IOutput output)
    {
        output.DrawItemList(game.Hero);
        MessagePage.ShowStillScreen(game, output);
        output.LoadPageColors(game.PageColors);

        game.Ui.ViewStatus = VIEW_ITEMS;
        TextRoutines.QueuePrint(game.Ui, PRQ_LIST);
    }

    /// Reveals hidden objects within reach (the Look entry).
    public static void LookAround(Data.Game game, IOutput output)
    {
        bool found = false;

        for (int index = 0; index < game.Actors.Drawn; index++)
        {
            Shape actor = game.Actors.List[index];

            if (actor.Kind == ActorKind.OBJECTS && actor.Race == 0 && Blocking.ActorDistance(game, index, 0) < LOOK_RANGE)
            {
                found = true;
                Objects.ChangeObject(game.Objects, game.Actors, game.World, index, 1);
            }
        }

        TextRoutines.ShowEvent(game, output, found ? EVENT_FOUND_HIDDEN : EVENT_NOTHING_HIDDEN);
    }
}
