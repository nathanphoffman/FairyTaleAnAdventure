using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// A click-to-move walk that ends by taking an item.
internal static class ItemWalks
{
    // Walks up to something lying about and takes it, or takes it at once when already near.
    internal static void WalkToItem(Data.Game game, IOutput output, int number)
    {
        ClickToMove.Cancel(game);

        if (ItemTargets.IsCloseEnough(game, number))
        {
            TakeItems.TakeFromActor(game, output, number);

            return;
        }

        Shape item = game.Actors.List[number];

        WalkRoutes.StartRoute(game, (item.AbsX, item.AbsY), null);
        game.Walk.Errands.ItemToTake = ClickToMove.Active(game) ? number : null;

        if (!ClickToMove.Active(game) && ItemTargets.IsCloseEnough(game, number))
        {
            TakeItems.TakeFromActor(game, output, number);
        }
    }

    // Takes the object walked to once the hero is near it, or when the walk ends short of it.
    internal static void TakeItemWhenThere(Data.Game game, IOutput output)
    {
        if (game.Walk.Errands.ItemToTake is { } number && ItemTargets.IsCloseEnough(game, number))
        {
            ClickToMove.Cancel(game);
            TakeIfStillThere(game, output, number);
        }
        else if (game.Walk.Errands.ArrivedToTake is { } arrived)
        {
            game.Walk.Errands.ArrivedToTake = null;
            TakeIfStillThere(game, output, arrived);
        }
    }

    internal static void TakeIfStillThere(Data.Game game, IOutput output, int number)
    {
        if (ItemTargets.StillThere(game, number))
        {
            TakeItems.TakeFromActor(game, output, number);
        }
    }
}
