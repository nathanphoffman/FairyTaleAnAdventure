using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Game.Ui;

/// What is inside a chest, urn or sacks: nothing, an item, two things, three keys or three of one item.
public static class ContainerContents
{
    /// What a container holds: nothing, an item, two things (the first may be 100 gold pieces),
    /// three keys, or three of one item.
    internal abstract record Contents
    {
        internal sealed record Nothing : Contents;

        internal sealed record One(int Index) : Contents;

        internal sealed record Two(int First, int Second) : Contents;

        internal sealed record ThreeKeys(int[] Keys) : Contents;

        internal sealed record ThreeOf(int Index) : Contents;
    }

    /// The container items start at this entry of the item list; a roll of 0 there means arrows
    /// (or 100 gold when it is the first of two things).
    private const int FIRST_CONTAINER_ITEM = 8;

    /// How many different things a roll can pick from.
    private const int ITEM_CHOICES = 8;

    /// How many different kinds of contents a container can roll.
    private const int CONTENT_KINDS = 4;

    private const int ROLL_ARROWS_OR_GOLD = FIRST_CONTAINER_ITEM;

    /// How many of one item, or keys, a container with several holds.
    private const int MANY = 3;

    /// The 100 gold pieces entry of the item list.
    internal const int HUNDRED_GOLD = ItemsMenu.GOLDBASE + 3;

    private const short CONTAINER_GOLD = 100;

    /// What was inside a container. Returns true (the container is always used up).
    public static bool Open(Data.Game game, IOutput output)
    {
        switch (RollContents(n => game.Rng.Next() & (n - 1)))
        {
            case Contents.Nothing:
                output.Print("nothing.");
                break;

            case Contents.One one:
                int found = MagicSwap.Resolve(game, one.Index);

                output.Print("a ");
                output.PrintCont(ItemsMenu.ItemName(found));
                output.PrintCont(".");
                ItemsMenu.AddStuff(game, found, 1);
                break;

            case Contents.Two two:
                TakeTwoThings(game, output, two.First, two.Second);
                break;

            case Contents.ThreeKeys keys:
                output.Print("3 keys.");

                foreach (int key in keys.Keys)
                {
                    ItemsMenu.AddStuff(game, key, 1);
                }

                break;

            case Contents.ThreeOf three:
                int many = MagicSwap.Resolve(game, three.Index);

                output.Print("3 ");
                output.PrintCont(ItemsMenu.ItemName(many));
                output.PrintCont("s.");
                ItemsMenu.AddStuff(game, many, MANY);
                break;
        }

        return true;
    }

    /// The item a container holds when it rolls roll (0-7) for one item: slot 8 means arrows.
    internal static int ContainerItem(int roll)
    {
        int index = roll + FIRST_CONTAINER_ITEM;

        return index == ROLL_ARROWS_OR_GOLD ? ItemsMenu.ARROWBASE : index;
    }

    /// A key found in a container: the two odd key types fold onto the first of their kind.
    internal static int ContainerKey(int roll)
    {
        int key = roll + ItemsMenu.KEYBASE;

        return key switch
        {
            22 => 16,
            23 => 20,
            _ => key,
        };
    }

    /// Rolls the contents with the game's random draws (pick(n) is 0 to n - 1), in the same order
    /// as the original: the kind (a draw of 4) first, then the item rolls (draws of 8).
    internal static Contents RollContents(Func<int, int> pick)
    {
        switch (pick(CONTENT_KINDS))
        {
            case 0:
                return new Contents.Nothing();

            case 1:
                return new Contents.One(ContainerItem(pick(ITEM_CHOICES)));

            case 2:
                return RollTwoThings(pick);

            default:
                int index = pick(ITEM_CHOICES) + FIRST_CONTAINER_ITEM;

                if (index != ROLL_ARROWS_OR_GOLD)
                {
                    return new Contents.ThreeOf(index);
                }

                int[] keys = new int[MANY];

                for (int count = 0; count < MANY; count++)
                {
                    keys[count] = ContainerKey(pick(ITEM_CHOICES));
                }

                return new Contents.ThreeKeys(keys);
        }
    }

    // Two different things: the first may be 100 gold pieces (a roll of 0), the second is rolled
    // again until it differs and may be arrows (a roll of 0).
    private static Contents RollTwoThings(Func<int, int> pick)
    {
        int first = pick(ITEM_CHOICES) + FIRST_CONTAINER_ITEM;

        if (first == ROLL_ARROWS_OR_GOLD)
        {
            first = HUNDRED_GOLD;
        }

        int second;

        do
        {
            second = pick(ITEM_CHOICES) + FIRST_CONTAINER_ITEM;
        }
        while (second == first);

        if (second == ROLL_ARROWS_OR_GOLD)
        {
            second = ItemsMenu.ARROWBASE;
        }

        return new Contents.Two(first, second);
    }

    // Tells what two things were found and adds them (the first may be 100 gold pieces).
    private static void TakeTwoThings(Data.Game game, IOutput output, int first, int second)
    {
        first = MagicSwap.Resolve(game, first);
        second = MagicSwap.Resolve(game, second);

        if (first == HUNDRED_GOLD)
        {
            game.Hero.Wealth += CONTAINER_GOLD;
        }
        else
        {
            output.PrintCont(" a");
        }

        output.Print(ItemsMenu.ItemName(first));
        output.PrintCont(" and a ");
        output.PrintCont(ItemsMenu.ItemName(second));

        if (first < ItemsMenu.GOLDBASE)
        {
            ItemsMenu.AddStuff(game, first, 1);
        }

        ItemsMenu.AddStuff(game, second, 1);
    }
}
