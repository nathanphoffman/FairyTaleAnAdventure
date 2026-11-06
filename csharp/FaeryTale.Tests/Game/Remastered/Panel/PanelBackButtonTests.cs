using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class PanelBackButtonTests
{
    private static FaeryTale.Game.Data.Game GameIn(Experience experience, short menu)
    {
        FaeryTale.Game.Data.Game game = new();

        game.Experience = experience;
        game.Ui.CurrentMenu = menu;

        return game;
    }

    [Fact]
    public void ItIsShownInASubmenuAndNotInTheFirstMenu()
    {
        Assert.True(PanelBackButton.IsShown(GameIn(Experience.Remastered, 2)));
        Assert.False(PanelBackButton.IsShown(GameIn(Experience.Remastered, 0)));
    }

    [Fact]
    public void ItIsNeverShownInTheOriginal()
    {
        Assert.False(PanelBackButton.IsShown(GameIn(Experience.Original, 2)));
    }

    [Fact]
    public void ItIsFoundWhereItIsDrawn()
    {
        Assert.True(PanelBackButton.Contains((480, 275)));
        Assert.False(PanelBackButton.Contains((480, 300)));
        Assert.False(PanelBackButton.Contains((400, 275)));
    }
}
