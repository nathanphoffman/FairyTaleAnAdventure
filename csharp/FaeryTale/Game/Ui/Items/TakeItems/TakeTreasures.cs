using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Game.Ui;

/// TakeItems: treasure.
public static partial class TakeItems
{
    // Says what other treasure was found and takes it: gold into the purse, anything else into the
    // pack. Says "nothing" if there was neither weapon nor treasure.
    private static void TakeTreasure(Data.Game game, IOutput output, BodyLoot loot)
    {
        if (loot.Treasure == 0)
        {
            if (loot.Weapon == 0)
            {
                output.PrintCont(game.Experience == Experience.Original ? "nothing" : "nothing in their pockets");
            }

            return;
        }

        if (loot.Weapon != 0)
        {
            output.PrintCont(" and ");
        }

        int treasure = MagicSwap.Resolve(game, (int)loot.Treasure);

        if (treasure < ItemsMenu.GOLDBASE)
        {
            output.PrintCont("a ");
        }

        output.PrintCont(ItemsMenu.ItemName(treasure));

        if (treasure >= ItemsMenu.GOLDBASE)
        {
            game.Hero.Wealth += Tables.ItemList[treasure].Maxshown;
        }
        else
        {
            ItemsMenu.AddStuff(game, treasure, 1);
        }
    }

    // Picks what a dead enemy carries besides its weapon, from the treasure table for its race.
    // Returns an item number (0 for nothing); characters like shopkeepers (a set figure) carry none.
    private static uint RollBodyTreasure(byte race, Rng rng)
    {
        if ((race & SETFIG_RACE_BIT) != 0)
        {
            return 0;
        }

        sbyte treasureClass = Tables.Entry(Tables.EncounterChart, race).Treasure;

        return (uint)(int)ObjectTables.TreasureProbs[treasureClass * TREASURE_CLASS_SIZE + rng.Rand8()];
    }
}
