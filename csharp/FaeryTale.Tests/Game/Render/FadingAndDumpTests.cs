using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Testing;

namespace FaeryTale.Tests;

public class FadingAndDumpTests
{
    [Fact]
    public void FullLightLeavesColoursAlone()
    {
        Assert.Equal(0x0a5c, Fading.FadedColor(0, 0x0a5c, (100, 100, 100), 0, false, false));
    }

    [Fact]
    public void NoLightIsBlack()
    {
        Assert.Equal(0, Fading.FadedColor(0, 0x0fff, (0, 0, 0), 0, false, false));
    }

    [Fact]
    public void MagicLightRaisesRedToGreen()
    {
        Assert.Equal(0x0ff0, Fading.FadedColor(0, 0x00f0, (100, 100, 100), 0, false, true));
    }

    [Fact]
    public void NightTintsTheLakeColoursButCapsBlue()
    {
        Assert.Equal(2, Fading.FadedColor(20, 0x0000, (100, 40, 100), 0, true, false) & 0xf);
        Assert.Equal(15, Fading.FadedColor(0, 0x000f, (100, 100, 100), 100, true, false) & 0xf);
    }

    [Fact]
    public void FadingDownTakesTwentyOneStepsOfOneTickEach()
    {
        FaeryTale.Game.Data.Game game = new();
        Recorder output = new();

        Fading.FadeDown(game, output);

        Assert.Equal(21u, output.Ticks);
        Assert.Equal(21, output.Calls.Count(call => call.Name == "LoadPageColors"));
    }

    [Fact]
    public void DayFadeOnlyRunsEveryFourthTick()
    {
        FaeryTale.Game.Data.Game game = new();
        Recorder output = new();

        game.World.DayClock = 1;
        Fading.DayFade(game, output);
        Assert.Empty(output.Calls);

        game.World.DayClock = 4;
        Fading.DayFade(game, output);
        Assert.Single(output.Calls);
    }

    [Fact]
    public void ActorLineListsEveryField()
    {
        Shape actor = new() { AbsX = 100, AbsY = 200, Vitality = -3, VelX = -1 };

        Assert.Equal(
            "actor 4 pos 100,200 type 0 race 0 idx 0 wpn 0 env 0 goal 0 tac 0 state 0 face 0 vit -3 vel -1,0\n",
            StateDump.ActorLine(4, actor));
    }

    [Fact]
    public void TheStateTextStartsWithTheHeroAndEndsWithTheStuff()
    {
        FaeryTale.Game.Data.Game game = new();
        string[] lines = StateDump.StateText(game).Split('\n');

        Assert.StartsWith("brother 0 region 3 hero 0,0 map 0,0", lines[0]);
        Assert.StartsWith("seed 19837325 23098324", lines[3]);
        Assert.Equal(3 + 1 + 20 + 1 + 1, lines.Length); // hero lines, seed, actors, stuff, then the empty text after the last newline
        Assert.StartsWith("stuff 0 0", lines[^2]);
    }
}
