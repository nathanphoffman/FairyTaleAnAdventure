using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;

namespace FaeryTale.Tests;

public class LookTextTests
{
    private const int HERO_SPOT = 1000;
    private const int NEAR_SPOT = 1010;

    // A game with the hero at a spot and one enemy of a race right beside him.
    private static FaeryTale.Game.Data.Game GameWithEnemyNearby(Experience experience, byte race)
    {
        FaeryTale.Game.Data.Game game = new() { Experience = experience };

        game.Hero.Brother = 1;

        game.Actors.List[0].AbsX = HERO_SPOT;
        game.Actors.List[0].AbsY = HERO_SPOT;
        game.Actors.List[1].AbsX = NEAR_SPOT;
        game.Actors.List[1].AbsY = HERO_SPOT;
        game.Actors.List[1].Kind = ActorKind.ENEMY;
        game.Actors.List[1].Race = race;
        game.Actors.List[1].State = ActorState.WALKING;
        game.Actors.Drawn = 2;

        return game;
    }

    [Fact]
    public void EverySubjectHasThreeDifferentLines()
    {
        string[] subjects = ["tree", "water", "rock", "wall", "body", "ogre", "wizard", "turtle", "chest", "key", "item"];

        foreach (string subject in subjects)
        {
            string[] lines = Enumerable.Range(0, 3).Select(draw => LookText.Pick(subject, draw)).ToArray();

            Assert.Equal(3, lines.Distinct().Count());
        }
    }

    [Fact]
    public void ARemasteredLookAtAnEnemySaysSomethingAboutIt()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyNearby(Experience.Remastered, 0);
        Recorder output = new();

        ItemsMenu.LookAround(game, output);

        Assert.Contains("ogre", string.Join(" ", output.PrintedLines));
    }

    [Fact]
    public void AnOriginalLookAtAnEnemyStillDiscoversNothing()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyNearby(Experience.Original, 0);
        Recorder output = new();

        ItemsMenu.LookAround(game, output);

        Assert.Contains("nothing", string.Join(" ", output.PrintedLines));
    }

    [Fact]
    public void BareGroundKeepsTheOldNothingMessageInRemastered()
    {
        FaeryTale.Game.Data.Game game = GameWithEnemyNearby(Experience.Remastered, 0);
        Recorder output = new();

        game.Actors.Drawn = 1;
        ItemsMenu.LookAround(game, output);

        Assert.Contains("nothing", string.Join(" ", output.PrintedLines));
    }
}
