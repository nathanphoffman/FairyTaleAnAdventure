using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Remastered;

/// Clicks on the play field (Remastered and Reimagined): which kind of click it is, and what walk or order it starts.
public static class WalkClicks
{
    internal const sbyte VIEW_NORMAL = 0;

    /// While the game is paused a click still gives an order (a walk, or an attack on an enemy) that
    /// the hero carries out when the game goes on, or queues a hotbar item. Remastered and Reimagined only.
    public static void GiveOrdersWhilePaused(Data.Game game, IOutput output)
    {
        if (game.Experience == Experience.Original || game.Ui.ViewStatus != VIEW_NORMAL || InputHandler.TakeClick() is not { } click)
        {
            return;
        }

        // A click on the hotbar queues that item for when the game goes on.
        if (Hotbar.TryClick(game, output, click, WorldView.Current.Width, WorldView.Current.Height))
        {
            return;
        }

        (int X, int Y) target = PointerWorld.OfClick(game, click);

        if (click.Native && EnemyTargets.EnemyAt(game, target) is { } enemy)
        {
            ClickToMove.Cancel(game);
            ClickToAttack.Order(enemy);
        }
        else
        {
            ClickToAttack.Clear();
            Plan(game, target);
        }
    }

    // A click in the world: a button of an open conversation, else a person to walk up to and talk
    // to, else a place (or door) to walk to. Persons and the bubble only exist in the new drawer.
    internal static void Click(Data.Game game, IOutput output, InputHandler.Click click)
    {
        if (Hotbar.TryClick(game, output, click, WorldView.Current.Width, WorldView.Current.Height) || Inventory.TryClick(game, output, InputHandler.GridPointer) || Journal.TryClick(InputHandler.GridPointer) || Conversation.TakeButtonClick(game, output, click) || PanelBackButton.TryPress(game, output, InputHandler.GridPointer))
        {
            return;
        }

        (int X, int Y) target = PointerWorld.OfClick(game, click);

        ClickToAttack.Clear();

        if (InputHandler.ShiftHeld)
        {
            LookWalks.WalkToLook(game, output, target);
        }
        else if (click.Native && EnemyTargets.EnemyAt(game, target) is { } enemy)
        {
            ClickToMove.Cancel(game);
            ClickToAttack.Order(enemy);
        }
        else if (click.Native && PersonTargets.PersonAt(game, target) is { } person)
        {
            PersonWalks.WalkToPerson(game, person);
        }
        else if (click.Native && output is Video video && ItemTargets.ItemAt(video.Capture, target) is { } item)
        {
            ItemWalks.WalkToItem(game, output, item);
        }
        else
        {
            Plan(game, target);
        }
    }

    // A click on a door walks to the door and the hero pushes his way through. So does a click on
    // the black outside of the room the hero is in: it means "get me out of here".
    internal static void Plan(Data.Game game, (int X, int Y) target) =>
        WalkRoutes.StartRoute(game, target, DoorTargets.DoorAt(game, target) ?? DoorTargets.ExitForBlackClick(game, target));
}
