using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class PauseMenuTests
{
    private const short GAME_MENU = 4;

    [Fact]
    public void WhenTheGameGoesOnThePanelGoesBackToTheFirstMenu()
    {
        FaeryTale.Game.Data.Game game = new();

        game.Experience = Experience.Remastered;
        game.Ui.CurrentMenu = GAME_MENU;
        PauseMenu.Reset();

        PauseMenu.Update(game, new Recorder(), true);
        Assert.Equal(GAME_MENU, game.Ui.CurrentMenu);

        PauseMenu.Update(game, new Recorder(), false);
        Assert.Equal(0, game.Ui.CurrentMenu);
    }

    [Fact]
    public void TheOriginalIsLeftAlone()
    {
        FaeryTale.Game.Data.Game game = new();

        game.Experience = Experience.Original;
        game.Ui.CurrentMenu = GAME_MENU;
        PauseMenu.Reset();

        PauseMenu.Update(game, new Recorder(), true);
        PauseMenu.Update(game, new Recorder(), false);

        Assert.Equal(GAME_MENU, game.Ui.CurrentMenu);
    }
}
