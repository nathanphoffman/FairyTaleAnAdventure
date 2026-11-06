using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class MenuGreyingTests
{
    private const int ITEMS_MENU = 0;
    private const int BUY_ENTRY = 3;
    private const int GIVE_ENTRY = 9;

    private static FaeryTale.Game.Data.Game RemasteredGame()
    {
        FaeryTale.Game.Data.Game game = new();

        game.Experience = Experience.Remastered;

        return game;
    }

    [Fact]
    public void BuyAndGiveAreGreyWithNobodyNear()
    {
        FaeryTale.Game.Data.Game game = RemasteredGame();

        Assert.True(MenuGreying.IsGreyed(game, ITEMS_MENU, BUY_ENTRY));
        Assert.True(MenuGreying.IsGreyed(game, ITEMS_MENU, GIVE_ENTRY));
    }

    [Fact]
    public void ABartenderNearMakesBuyAndGiveUsable()
    {
        FaeryTale.Game.Data.Game game = RemasteredGame();

        game.Actors.List[1].Race = 0x88;
        game.Actors.NearestPerson = 1;

        Assert.False(MenuGreying.IsGreyed(game, ITEMS_MENU, BUY_ENTRY));
        Assert.False(MenuGreying.IsGreyed(game, ITEMS_MENU, GIVE_ENTRY));
    }

    [Fact]
    public void SomeoneElseNearOnlyMakesGiveUsable()
    {
        FaeryTale.Game.Data.Game game = RemasteredGame();

        game.Actors.List[1].Race = 0x8d;
        game.Actors.NearestPerson = 1;

        Assert.True(MenuGreying.IsGreyed(game, ITEMS_MENU, BUY_ENTRY));
        Assert.False(MenuGreying.IsGreyed(game, ITEMS_MENU, GIVE_ENTRY));
    }

    [Fact]
    public void TheOriginalIsNeverGreyed()
    {
        FaeryTale.Game.Data.Game game = new();

        game.Experience = Experience.Original;

        Assert.False(MenuGreying.IsGreyed(game, ITEMS_MENU, BUY_ENTRY));
    }
}
