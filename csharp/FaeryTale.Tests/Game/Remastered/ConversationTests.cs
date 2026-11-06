using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class ConversationTests
{
    // A game with one living person standing where the hero cannot be (actor 1).
    private static FaeryTale.Game.Data.Game GameWithPersonAt(int x, int y)
    {
        FaeryTale.Game.Data.Game game = new();
        Shape person = game.Actors.List[1];

        (person.Kind, person.State, person.AbsX, person.AbsY) = (ActorKind.SETFIG, 0, (ushort)x, (ushort)y);
        game.Actors.Drawn = 2;

        return game;
    }

    [Fact]
    public void APersonIsFoundWhereTheirPictureIs()
    {
        FaeryTale.Game.Data.Game game = GameWithPersonAt(1000, 2000);

        Assert.Equal(1, PersonTargets.PersonAt(game, (1004, 1980)));
        Assert.Null(PersonTargets.PersonAt(game, (1040, 1980)));
        Assert.Null(PersonTargets.PersonAt(game, (1004, 1900)));
    }

    [Fact]
    public void ADeadPersonCannotBeTalkedTo()
    {
        FaeryTale.Game.Data.Game game = GameWithPersonAt(1000, 2000);

        game.Actors.List[1].State = ActorState.DEAD;

        Assert.Null(PersonTargets.PersonAt(game, (1004, 1980)));
    }

    [Fact]
    public void EachButtonIsFoundWhereItIsDrawn()
    {
        BubbleLayout layout = new(100, 50, 2);

        for (int index = 0; index < BubbleLayout.BUTTON_COUNT; index++)
        {
            int x = 100 + (BubbleLayout.ButtonLeft + 1) * 2;
            int y = 50 + (BubbleLayout.ButtonTop(index) + 1) * 2;

            Assert.Equal(index, layout.ButtonAt(x, y));
        }
    }

    [Fact]
    public void TheFaceAndTheGapsBetweenButtonsAreNotButtons()
    {
        BubbleLayout layout = new(100, 50, 2);
        int gapY = 50 + (BubbleLayout.BUTTON_HEIGHT + 1) * 2;

        Assert.Null(layout.ButtonAt(100 + 5, 50 + 5));
        Assert.Null(layout.ButtonAt(100 + (BubbleLayout.ButtonLeft + 1) * 2, gapY));
    }
}
