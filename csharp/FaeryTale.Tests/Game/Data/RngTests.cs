using FaeryTale.Game.Data;

namespace FaeryTale.Tests;

public class RngTests
{
    private const int START_SEED = 19_837_325;

    // Values produced by the C original with the game's starting seed.
    [Fact]
    public void MatchesTheCGenerator()
    {
        int seed = START_SEED;
        List<int> values = new();

        for (int count = 0; count < 5; count++)
        {
            (seed, int value) = Rng.Step(seed);
            values.Add(value);
        }

        Assert.Equal(new[] { 1777372681, 1298959771, 555739856, 636999534, 442890465 }, values);
        Assert.Equal(-1719781274, seed);
    }

    [Fact]
    public void BoundedValuesMatchTheCOriginal()
    {
        int seed = START_SEED;
        List<int> values = new();

        for (int count = 0; count < 5; count++)
        {
            (seed, int value) = Rng.Step(seed);
            values.Add(Rng.Bounded(value, 100));
        }

        Assert.Equal(new[] { 61, 51, 12, 50, 13 }, values);
    }

    [Fact]
    public void NextStepsTheSeedOfTheInstance()
    {
        Rng rng = new();
        int[] values = Enumerable.Range(0, 5).Select(_ => rng.Next()).ToArray();

        Assert.Equal(new[] { 1777372681, 1298959771, 555739856, 636999534, 442890465 }, values);
        Assert.Equal(-1719781274, rng.Seed1);
    }
}
