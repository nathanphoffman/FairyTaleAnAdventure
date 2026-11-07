using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class JournalTests
{
    [Fact]
    public void KeepsWhatWasSaidOnceWithTheBrothersName()
    {
        FaeryTale.Game.Data.Game game = new() { Experience = Experience.Remastered };

        game.Hero.Brother = 1;
        Journal.Reset();
        Journal.Record(game, "Go north, %."u8);
        Journal.Record(game, "Go north, %."u8);
        Journal.Record(game, ""u8);

        Assert.Equal(["Go north, Julian."], Journal.Entries());

        Journal.Reset();

        Assert.Empty(Journal.Entries());
    }
}
