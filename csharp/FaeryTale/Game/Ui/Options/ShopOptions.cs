using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Game.Ui.Options;

/// The Buy menu: what the bartender sells and whether the hero can pay.
public static class ShopOptions
{
    /// What a click in the Buy menu does.
    internal abstract record BuyGate
    {
        /// Nobody (or not a bartender) near: nothing happens, the menu stays.
        internal sealed record Nothing : BuyGate;

        /// An entry past the last one: not handled here.
        internal sealed record NotHandled : BuyGate;

        /// Offers an item for a price.
        internal sealed record Offer(int Item, short Price) : BuyGate;
    }

    internal enum Bought
    {
        Food,
        Arrows,
        Item,
    }

    internal abstract record Purchase
    {
        internal sealed record TooPoor : Purchase;

        internal sealed record Made(short WealthLeft, Bought What) : Purchase;
    }

    // The bartender is race 0x88; entries 5-11 pick a pair in Jtrans.
    private const byte BARTENDER_RACE = 0x88;
    private const short LAST_BUY_ENTRY = 11;
    internal const short FIRST_MENU_ENTRY = 5;
    private const int FOOD = 0;
    private const int ARROWS = 8;
    private const int BOUGHT_FOOD_EVENT = 22;
    private const int BOUGHT_ARROWS_EVENT = 23;
    private const int FOOD_HUNGER = 50;
    private const byte ARROWS_BOUGHT = 10;

    /// How many things the bartender sells.
    public const int ITEM_COUNT = LAST_BUY_ENTRY - FIRST_MENU_ENTRY + 1;

    /// Buys what the bartender sells, if the hero can pay.
    public static bool BuyOption(Data.Game game, IOutput output, short chosenEntry)
    {
        game.Actors.NearestObject = game.Actors.NearestPerson;

        return Trade(game, output, game.Actors.NearestObject, chosenEntry);
    }

    /// Buys the thing number itemNumber (0 is the first on the list) from one chosen person (Remastered:
    /// the one clicked on) instead of the nearest.
    public static void BuyFromActor(Data.Game game, IOutput output, int actorNumber, int itemNumber)
    {
        game.Actors.NearestObject = (short)actorNumber;
        Trade(game, output, (short)actorNumber, (short)(FIRST_MENU_ENTRY + itemNumber));
    }

    /// Whether a person sells things.
    public static bool IsBartender(Shape actor) => actor.Race == BARTENDER_RACE;

    /// What thing number itemNumber on the list costs (in gold).
    public static short PriceOf(int itemNumber) => ObjectTables.Jtrans[itemNumber * 2 + 1];

    // Does what a click on a Buy entry means with the person near.
    private static bool Trade(Data.Game game, IOutput output, short near, short chosenEntry)
    {
        byte race = near == 0 ? (byte)0 : game.Actors.List[near].Race;

        switch (BuyGateFor(near, race, chosenEntry, ObjectTables.Jtrans))
        {
            case BuyGate.Nothing:
                return true;

            case BuyGate.NotHandled:
                return false;

            case BuyGate.Offer offer:
                MakePurchase(game, output, offer);

                return true;

            default:
                return true;
        }
    }

    /// race is that of the nearest person (only looked at when person is not 0).
    internal static BuyGate BuyGateFor(short person, byte race, short entry, sbyte[] table)
    {
        if (person == 0 || race != BARTENDER_RACE)
        {
            return new BuyGate.Nothing();
        }

        if (entry > LAST_BUY_ENTRY)
        {
            return new BuyGate.NotHandled();
        }

        int pair = (entry - FIRST_MENU_ENTRY) * 2;

        return new BuyGate.Offer((byte)table[pair], table[pair + 1]);
    }

    /// The hero must have strictly more gold than the price.
    internal static bool CanAfford(short gold, short price) => gold > price;

    /// Buying item for price with wealth gold.
    internal static Purchase PurchaseFor(short wealth, short price, int item)
    {
        if (!CanAfford(wealth, price))
        {
            return new Purchase.TooPoor();
        }

        Bought what = item == FOOD ? Bought.Food : item == ARROWS ? Bought.Arrows : Bought.Item;

        return new Purchase.Made((short)(wealth - price), what);
    }

    private static void MakePurchase(Data.Game game, IOutput output, BuyGate.Offer offer)
    {
        if (PurchaseFor(game.Hero.Wealth, offer.Price, offer.Item) is not Purchase.Made made)
        {
            output.Print("Not enough money!");

            return;
        }

        game.Hero.Wealth = made.WealthLeft;
        TextRoutines.QueuePrint(game.Ui, MenuChoices.PRQ_STATS_LINE);
        ReceivePurchase(game, output, offer.Item, made.What);
    }

    // Hands over what was bought: food is eaten at once, arrows come in a bundle, anything else is
    // added to the pack with a message.
    private static void ReceivePurchase(Data.Game game, IOutput output, int item, Bought what)
    {
        switch (what)
        {
            case Bought.Food:
                TextRoutines.ShowEvent(game, output, BOUGHT_FOOD_EVENT);
                Food.EatFood(game, output, FOOD_HUNGER);

                break;

            case Bought.Arrows:
                ItemsMenu.AddStuff(game, item, ARROWS_BOUGHT);
                TextRoutines.ShowEvent(game, output, BOUGHT_ARROWS_EVENT);

                break;

            default:
                int bought = MagicSwap.Resolve(game, item);

                ItemsMenu.AddStuff(game, bought, 1);
                Messages.PrintWrapped(game, output, "% bought a "u8);
                output.PrintCont(ItemsMenu.ItemName(bought));
                output.PrintCont(".");

                break;
        }
    }
}
