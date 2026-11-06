using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class CheatCodeTests
{
    private static void TypeWord(string word)
    {
        foreach (char letter in word)
        {
            CheatCode.Type(letter);
        }
    }

    [Fact]
    public void TypingTheWordCompletesTheCheatOnceEvenAfterOtherLetters()
    {
        CheatCode.TakeCompleted();

        TypeWord("xyzgraphix");

        Assert.True(CheatCode.TakeCompleted());
        Assert.False(CheatCode.TakeCompleted());
    }

    [Fact]
    public void AWrongLetterOrAShortWordDoesNotCompleteIt()
    {
        CheatCode.TakeCompleted();

        TypeWord("graphx");
        TypeWord("grap");

        Assert.False(CheatCode.TakeCompleted());
    }

    [Fact]
    public void TheCheatGivesEveryItemButTheTalisman()
    {
        FaeryTale.Game.Data.Game game = new();
        short goldBefore = game.Hero.Wealth;

        CheatCode.GiveEverything(game, new Recorder());

        Assert.Equal(1, game.Hero.Stuff[2]);
        Assert.Equal(5, game.Hero.Stuff[10]);
        Assert.Equal(3, game.Hero.Stuff[16]);
        Assert.Equal(0, game.Hero.Stuff[22]);
        Assert.True(game.Hero.Wealth > goldBefore);
    }
}
