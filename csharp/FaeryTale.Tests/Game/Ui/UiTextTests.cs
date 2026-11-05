using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Tests;

public class UiTextTests
{
    [Fact]
    public void NumbersAreRightAlignedWithATrailingSpace()
    {
        Assert.Equal("       123 ", TextEncoding.FromBytes(TextRoutines.FormatNumber(123)));
        Assert.Equal("         0 ", TextEncoding.FromBytes(TextRoutines.FormatNumber(0)));
    }

    [Fact]
    public void ShowErrorPrintsTheErrorNumberAtAFixedPlace()
    {
        Recorder output = new();

        TextRoutines.ShowError(output, 5);

        Assert.Equal(
            new[] { "SetTextPosition(200, 90)", "PrintContBytes(\"ERROR \")", "PrintNumber(5, 2)" },
            output.Calls.Select(call => call.ToString()));
    }

    [Fact]
    public void ShowErrorWithoutGraphicsPrintsNothing()
    {
        Recorder output = new();

        TextRoutines.ShowError(output, 2);

        Assert.Empty(output.Calls);
    }

    [Fact]
    public void AnEventMessageIsPrintedAsTextLines()
    {
        FaeryTale.Game.Data.Game game = new();
        game.Hero.Brother = 1;
        Recorder output = new();

        TextRoutines.ShowEvent(game, output, 0);

        Assert.Equal(new[] { "Julian was getting rather hungry." }, output.PrintedLines);
    }

    [Fact]
    public void LongMessagesWrapAtSpacesWithin37Characters()
    {
        FaeryTale.Game.Data.Game game = new();
        game.Hero.Brother = 2;
        Recorder output = new();

        TextRoutines.ShowEvent(game, output, 15); // the long weapon-in-here message

        Assert.True(output.PrintedLines.Count > 1);
        Assert.All(output.PrintedLines, line => Assert.True(line.Length <= 38, line));
        Assert.Contains("Phillip", string.Join(' ', output.PrintedLines));
    }

    [Fact]
    public void PositionEscapesMoveTheTextCursor()
    {
        Recorder output = new();

        TextRoutines.PrintPositioned(output, [0x80, 0x0a, 0x1c, (byte)'H', (byte)'i', 0]);

        Assert.Equal(new[] { "SetTextPosition(20, 28)", "PrintContBytes(\"Hi\")" }, output.Calls.Select(call => call.ToString()));
    }

    [Fact]
    public void ThePrintQueueIgnoresRequestsWhenFull()
    {
        FaeryTale.Game.Data.Ui ui = new();

        for (int request = 0; request < 40; request++)
        {
            TextRoutines.QueuePrint(ui, request);
        }

        Assert.Equal(31, ui.PrintQueue.Write); // 31 fit; one slot stays empty so full differs from empty
    }

    [Fact]
    public void MusicDependsOnWhatTheHeroIsDoing()
    {
        FaeryTale.Game.Data.Game game = new();
        game.Actors.List[0].Vitality = 10;
        game.World.LightLevel = 200;

        Assert.Equal(Music.TUNE_DAY, Music.PickTune(game.Actors, game.World, game.Hero));

        game.World.LightLevel = 50;
        Assert.Equal(Music.TUNE_NIGHT, Music.PickTune(game.Actors, game.World, game.Hero));

        game.Actors.BattleFlag = 1;
        Assert.Equal(Music.TUNE_BATTLE, Music.PickTune(game.Actors, game.World, game.Hero));

        game.Actors.List[0].Vitality = 0;
        Assert.Equal(Music.TUNE_DEATH, Music.PickTune(game.Actors, game.World, game.Hero));
    }
}
