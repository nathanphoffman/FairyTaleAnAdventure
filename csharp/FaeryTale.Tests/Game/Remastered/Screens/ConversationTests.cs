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

    private static List<Row> Rows(params RowKind[] kinds) =>
        kinds.Select((kind, index) => new Row("x", kind, index, true)).ToList();

    [Fact]
    public void EachRowIsFoundWhereItIsDrawn()
    {
        BubbleLayout layout = new(100, 50, 2);
        List<Row> rows = Rows(RowKind.Talk, RowKind.Talk, RowKind.Buy);

        for (int index = 0; index < rows.Count; index++)
        {
            int x = 100 + (BubbleLayout.ButtonLeft + 1) * 2;
            int y = 50 + (BubbleLayout.ButtonTop(index) + 1) * 2;

            Assert.Equal(index, layout.RowAt(x, y, rows));
        }
    }

    [Fact]
    public void TheFaceTheGapsAndTheEmptySpaceBesideNarrowBoardsAreNotRows()
    {
        BubbleLayout layout = new(100, 50, 2);
        List<Row> rows = Rows(RowKind.Talk, RowKind.Item);
        int gapY = 50 + (BubbleLayout.BUTTON_HEIGHT + 1) * 2;
        int beyondNarrow = 100 + (BubbleLayout.ButtonLeft + BubbleLayout.BUTTON_WIDTH + 2) * 2;

        Assert.Null(layout.RowAt(100 + 5, 50 + 5, rows));
        Assert.Null(layout.RowAt(100 + (BubbleLayout.ButtonLeft + 1) * 2, gapY, rows));
        Assert.Null(layout.RowAt(beyondNarrow, 50 + 5, rows));
        Assert.Equal(1, layout.RowAt(beyondNarrow, 50 + (BubbleLayout.ButtonTop(1) + 1) * 2, rows));
    }

    [Fact]
    public void OnlyABartenderHasABuyRow()
    {
        FaeryTale.Game.Data.Game game = GameWithPersonAt(1000, 2000);

        Conversation.Open(game, 1);
        Assert.DoesNotContain(Conversation.Rows(game), row => row.Kind == RowKind.Buy);

        game.Actors.List[1].Race = 0x88;
        Assert.Contains(Conversation.Rows(game), row => row.Kind == RowKind.Buy);
        Conversation.Close();
    }
}
