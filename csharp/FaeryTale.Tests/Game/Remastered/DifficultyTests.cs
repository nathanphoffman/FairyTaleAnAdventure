using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class DifficultyTests
{
    private static FaeryTale.Game.Data.Game GameAt(DifficultyLevel level, Experience experience = Experience.Remastered) =>
        new() { Experience = experience, Difficulty = level };

    [Fact]
    public void HardIsTheOriginalGamesNumbers()
    {
        Difficulty.Factors hard = Difficulty.FactorsOf(DifficultyLevel.Hard);

        Assert.Equal(new Difficulty.Factors(1, 1, 1, 1), hard);
    }

    [Fact]
    public void EachStepIsHarderForTheHero()
    {
        // The damage a hero takes per point of enemy damage grows with every level.
        double[] taken = Enum.GetValues<DifficultyLevel>().Select(level => Difficulty.DamageFactor(GameAt(level), 4, 0)).ToArray();

        Assert.Equal(taken.OrderBy(value => value), taken);
        Assert.True(taken[(int)DifficultyLevel.Normal] < 1 && taken[(int)DifficultyLevel.VeryHard] > 1);
    }

    [Fact]
    public void EachStepIsHarderOnTheEnemiesToo()
    {
        // The damage the hero deals per point shrinks with every level.
        double[] dealt = Enum.GetValues<DifficultyLevel>().Select(level => Difficulty.DamageFactor(GameAt(level), 0, 4)).ToArray();

        Assert.Equal(dealt.OrderByDescending(value => value), dealt);
    }

    [Fact]
    public void OriginalIgnoresTheDifficulty()
    {
        Assert.Equal(1, Difficulty.DamageFactor(GameAt(DifficultyLevel.Explorer, Experience.Original), 4, 0));
        Assert.Equal(4, Difficulty.ScaleDamage(GameAt(DifficultyLevel.Extreme, Experience.Original), 0, 4, 4));
    }

    [Fact]
    public void MonstersHittingEachOtherAreNotChanged() =>
        Assert.Equal(1, Difficulty.DamageFactor(GameAt(DifficultyLevel.Explorer), 4, 3));

    [Fact]
    public void WeakHitsAddUpInsteadOfBeingLost()
    {
        FaeryTale.Game.Data.Game game = GameAt(DifficultyLevel.Explorer);

        // Explorer: an enemy hit of 4 on the hero counts for a twelfth of that, so twelve hits make 4.
        int total = Enumerable.Range(0, 12).Sum(_ => Difficulty.ScaleDamage(game, 4, 0, 4));

        Assert.InRange(total, 3, 4);
    }

    [Fact]
    public void TheNextLevelGoesRoundAfterExtreme()
    {
        Assert.Equal(DifficultyLevel.Hard, Difficulty.Next(DifficultyLevel.Normal));
        Assert.Equal(DifficultyLevel.Explorer, Difficulty.Next(DifficultyLevel.Extreme));
    }
}
