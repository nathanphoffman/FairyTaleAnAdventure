using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// A click-to-move walk that ends by talking to a person.
internal static class PersonWalks
{
    // Walks towards a person, or opens the conversation at once when already near.
    internal static void WalkToPerson(Data.Game game, int number)
    {
        ClickToMove.Cancel(game);

        if (PersonTargets.IsCloseEnough(game, number))
        {
            Conversation.Open(game, number);

            return;
        }

        Shape person = game.Actors.List[number];

        WalkClicks.Plan(game, (person.AbsX, person.AbsY));
        game.Walk.Errands.PersonToTalkTo = ClickToMove.Active(game) ? number : null;

        if (!ClickToMove.Active(game) && PersonTargets.IsInTalkRange(game, number))
        {
            Conversation.Open(game, number);
        }
    }

    // Once the hero is near the person walked to, stops and opens the conversation.
    internal static void OpenConversationWhenNear(Data.Game game)
    {
        if (game.Walk.Errands.PersonToTalkTo is not { } number || !PersonTargets.IsCloseEnough(game, number))
        {
            return;
        }

        ClickToMove.Cancel(game);
        Conversation.Open(game, number);
    }
}
