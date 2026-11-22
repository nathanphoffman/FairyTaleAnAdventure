using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// A click-to-move walk that ends by looking around (shift and click).
internal static class LookWalks
{
    // Shift and click: walks to the spot (not through a door there) and then looks around.
    internal static void WalkToLook(Data.Game game, IOutput output, (int X, int Y) target)
    {
        ClickToMove.Cancel(game);
        game.Walk.Errands.LookSpot = target;

        // A spot within half the reach of a look is looked at from where the hero stands.
        if (LooksFromHere(game.Actors.List[0], target))
        {
            ItemsMenu.LookAround(game, output, game.Walk.Errands.LookSpot);

            return;
        }

        WalkRoutes.StartRoute(game, target, null);

        if (ClickToMove.Active(game))
        {
            game.Walk.Errands.LookWhenThere = true;
        }
        else
        {
            ItemsMenu.LookAround(game, output, game.Walk.Errands.LookSpot);
        }
    }

    /// Whether a spot is so near the hero (within half the reach of a look) that the look is made without
    /// walking there.
    internal static bool LooksFromHere(Shape hero, (int X, int Y) target) =>
        Math.Max(Math.Abs(target.X - hero.AbsX), Math.Abs(target.Y - hero.AbsY)) <= ItemsMenu.LOOK_RANGE / 2;

    // Looks around once a walk that was meant to end with a look is over.
    internal static void LookIfArrived(Data.Game game, IOutput output)
    {
        if (game.Walk.Errands.ArrivedToLook)
        {
            game.Walk.Errands.ArrivedToLook = false;
            ItemsMenu.LookAround(game, output, game.Walk.Errands.LookSpot);
        }
    }
}
