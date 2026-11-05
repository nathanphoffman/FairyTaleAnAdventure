using FaeryTale.Game;
using FaeryTale.Testing;

namespace FaeryTale.Tests;

public class TestScriptTests
{
    [Fact]
    public void KeysAreHeldFourTicksUnlessToldOtherwise()
    {
        List<KeyEvent> events = TestScript.ParseKeys("100:Space,200:Right:60");

        Assert.Equal(
            new[]
            {
                new KeyEvent(100, "Space", true), new KeyEvent(104, "Space", false),
                new KeyEvent(200, "Right", true), new KeyEvent(260, "Right", false),
            },
            events);
    }

    [Fact]
    public void ClicksNeedThreeNumbersAndOthersAreSkipped()
    {
        List<Click> clicks = TestScript.ParseClicks("200:225:166,bad,260:225");

        Assert.Equal(new[] { new Click(200, 225, 166) }, clicks);
    }

    [Fact]
    public void CommandsMapToTheirKinds()
    {
        List<TimedCommand> commands = TestScript.ParseCommands("900:tp:18310:15969,1200:kill:0,5:nope:1");

        Assert.Equal(
            new[]
            {
                new TimedCommand(900, new Command(CommandKind.Teleport, 18310, 15969)),
                new TimedCommand(1200, new Command(CommandKind.KillHero, 0, 0)),
            },
            commands);
    }

    [Fact]
    public void EmptyOrMissingScriptsGiveNothing()
    {
        Assert.Empty(TestScript.ParseKeys(null));
        Assert.Empty(TestScript.ParseClicks(""));
        Assert.Empty(TestScript.ParseCommands(null));
    }

    [Fact]
    public void ALaterQueuedCommandReplacesAnEarlierOne()
    {
        CommandQueue.Queue(new Command(CommandKind.SlayEnemies, 0, 0));
        CommandQueue.Queue(new Command(CommandKind.Give, 1, 2));

        Assert.Equal(new Command(CommandKind.Give, 1, 2), CommandQueue.Take());
        Assert.Null(CommandQueue.Take());
    }
}
