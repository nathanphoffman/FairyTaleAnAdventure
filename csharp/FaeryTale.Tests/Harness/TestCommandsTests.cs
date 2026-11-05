using FaeryTale.Game;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Testing;

namespace FaeryTale.Tests;

public class TestCommandsTests
{
    [Fact]
    public void GiveAndStatCommandsChangeTheHero()
    {
        FaeryTale.Game.Data.Game game = new();
        game.Hero.Brother = 1;
        Recorder output = new();

        CommandQueue.Queue(new Command(CommandKind.Give, 22, 1));
        TestCommands.RunPendingCommand(game, output);
        CommandQueue.Queue(new Command(CommandKind.SetStat, 0, 99));
        TestCommands.RunPendingCommand(game, output);

        Assert.Equal(1, game.Hero.Stuff[22]);
        Assert.Equal(99, game.Hero.Hunger);
    }

    [Fact]
    public void KillingTheHeroSetsLuckAndStartsTheDeath()
    {
        FaeryTale.Game.Data.Game game = new();
        game.Hero.Brother = 1;
        game.Actors.List[0].Vitality = 50;
        Recorder output = new();

        CommandQueue.Queue(new Command(CommandKind.KillHero, 3, 0));
        TestCommands.RunPendingCommand(game, output);

        Assert.Equal(ActorState.DYING, game.Actors.List[0].State);
        Assert.Equal(3 - 5, game.Hero.Luck);
    }

    [Fact]
    public void NothingHappensWithoutACommand()
    {
        FaeryTale.Game.Data.Game game = new();
        Recorder output = new();

        TestCommands.RunPendingCommand(game, output);

        Assert.Empty(output.Calls);
    }
}
