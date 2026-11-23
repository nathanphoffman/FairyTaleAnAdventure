using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Ui;

/// TakeItems: picking up one kind of thing.
public static partial class TakeItems
{
    /// The inventory slot for object id in the (id, slot) pair table Itrans, which ends with a 0 id.
    internal static int? ItemSlot(uint identifier, byte[] table)
    {
        for (int offset = 0; table[offset] != 0; offset += 2)
        {
            if (identifier == table[offset])
            {
                return table[offset + 1];
            }
        }

        return null;
    }

    // Removes the picked-up object from the world, turns any arrows picked up into the arrow
    // count, and ends the game if the talisman was found.
    private static void FinishPickup(Data.Game game, IOutput output)
    {
        Objects.ChangeObject(game.Objects, game.Actors, game.World, game.Actors.NearestObject, 2);
        ItemsMenu.AddStuff(game, ItemsMenu.ARROWS, (byte)(game.Hero.Stuff[ItemsMenu.ARROWBASE] * ARROWS_PER_QUIVER));

        if (game.Hero.Stuff[TALISMAN] != 0)
        {
            game.Ui.QuitRequested = 1;
            game.Ui.ViewStatus = ItemsMenu.VIEW_GAME_OVER;
            MessagePage.MapMessage(game, output);
            output.UseStoryFont();
            ScreenEffects.WinColors(game, output);
        }
    }

    // Handles taking the object NearestObject. Returns true if it was taken (so it must be removed).
    private static bool TakeObject(Data.Game game, IOutput output)
    {
        Shape actor = game.Actors.List[game.Actors.NearestObject];
        uint identifier = (byte)actor.Index;
        uint owner = (uint)actor.Vitality & OWNER_MASK;

        switch (ClassifyTake(identifier, game.Hero.Hunger, game.World.Region))
        {
            case Take.Gold:
                Messages.AnnounceTreasure(game, output, "50 gold pieces");
                game.Hero.Wealth += GOLD_FOUND;

                return true;

            case Take.Paper paper:
                TextRoutines.ShowEvent(game, output, EVENT_PAPER);
                TextRoutines.ShowEvent(game, output, paper.Indoors ? EVENT_PAPER_INDOORS : EVENT_PAPER_OUTDOORS);

                return true;

            case Take.Fruit fruit:
                return TakeFruit(game, output, fruit.Keep);

            case Take.Bones:
                return TakeBrothersGoods(game, output, owner);

            case Take.Container container:
                Messages.AnnounceContainer(game, output, container.Name);

                return ContainerContents.Open(game, output);

            case Take.Plain:
                return TakePlainItem(game, output, identifier);

            default:
                return false; // eggs and left things are not taken
        }
    }

    // Fruit is kept in the pack when the hero is not hungry, and eaten otherwise.
    private static bool TakeFruit(Data.Game game, IOutput output, bool keep)
    {
        if (keep)
        {
            ItemsMenu.AddStuff(game, FRUIT_SLOT, 1);
            TextRoutines.ShowEvent(game, output, EVENT_FRUIT_KEPT);
        }
        else
        {
            Food.EatFood(game, output, FRUIT_HUNGER);
        }

        return true;
    }

    // Picks up a fallen brother's bones and everything he carried (owner 1 is Julian, otherwise Phillip).
    private static bool TakeBrothersGoods(Data.Game game, IOutput output, uint owner)
    {
        Messages.AnnounceTreasure(game, output, "his brother's bones.");
        game.Objects.Global[FIRST_BONES].ObStat = 0;
        game.Objects.Global[SECOND_BONES].ObStat = 0;

        for (int slot = 0; slot < ItemsMenu.GOLDBASE; slot++)
        {
            ItemsMenu.AddStuff(game, slot, owner == 1 ? game.Hero.Julstuff[slot] : game.Hero.Philstuff[slot]);
        }

        return true;
    }

    // A single item lying about: the slot is found in the Itrans table. Unknown objects are left.
    private static bool TakePlainItem(Data.Game game, IOutput output, uint identifier)
    {
        if (ItemSlot(identifier, ObjectTables.Itrans) is not { } found)
        {
            return false;
        }

        int slot = MagicSwap.Resolve(game, found);

        ItemsMenu.AddStuff(game, slot, 1);
        Messages.AnnounceTreasure(game, output, "a ");
        output.PrintCont(ItemsMenu.ItemName(slot));
        output.PrintCont(".");

        return true;
    }
}
