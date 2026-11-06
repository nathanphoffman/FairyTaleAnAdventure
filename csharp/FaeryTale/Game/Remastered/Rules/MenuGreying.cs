using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Game.Remastered;

/// Greys out the classic Buy and Give boxes of the text panel (Remastered and Reimagined) when they
/// cannot be used: Buy needs a bartender near and Give needs someone near. The box is drawn dark
/// grey instead of its usual colour, and the panel is drawn again whenever that changes.
public static class MenuGreying
{
    /// The panel's dark grey (pen 15 of the panel colours); the black label on it is hard to read.
    public const byte GREY_PEN = 15;

    // Menus before this one start with the five main names; Buy is the fourth of them.
    private const int FIRST_NON_MAIN_MENU = 8;
    private const int BUY_ENTRY = 3;

    // Give is the last entry of the first menu (the Items menu).
    private const int ITEMS_MENU = 0;
    private const int GIVE_ENTRY = 9;

    private const sbyte VIEW_NORMAL = 0;

    private static bool? buyWasGreyed;
    private static bool? giveWasGreyed;

    /// Forgets what was drawn (a new game starts with a fresh panel).
    public static void Reset()
    {
        buyWasGreyed = null;
        giveWasGreyed = null;
    }

    /// Whether an entry of a menu is drawn grey now.
    public static bool IsGreyed(Data.Game game, int menu, int entry)
    {
        if (game.Experience == Data.Experience.Original)
        {
            return false;
        }

        if (menu < FIRST_NON_MAIN_MENU && entry == BUY_ENTRY)
        {
            return !BartenderIsNear(game);
        }

        return menu == ITEMS_MENU && entry == GIVE_ENTRY && game.Actors.NearestPerson == 0;
    }

    /// Draws the panel's menu again when Buy or Give has just become usable or not. Called every frame.
    public static void Refresh(Data.Game game, IOutput output)
    {
        if (game.Experience == Data.Experience.Original || game.Ui.ViewStatus != VIEW_NORMAL)
        {
            return;
        }

        bool buy = IsGreyed(game, ITEMS_MENU, BUY_ENTRY);
        bool give = IsGreyed(game, ITEMS_MENU, GIVE_ENTRY);

        if (buy == buyWasGreyed && give == giveWasGreyed)
        {
            return;
        }

        (buyWasGreyed, giveWasGreyed) = (buy, give);
        Menus.PrintOptions(game, output);
    }

    // The person nearest the hero is a bartender (the one Buy would buy from).
    private static bool BartenderIsNear(Data.Game game) =>
        game.Actors.NearestPerson != 0 && ShopOptions.IsBartender(game.Actors.List[game.Actors.NearestPerson]);
}
