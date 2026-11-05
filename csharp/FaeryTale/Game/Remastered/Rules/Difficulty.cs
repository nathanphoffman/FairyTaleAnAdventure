using FaeryTale.Game.Data;

namespace FaeryTale.Game.Remastered;

/// The numbers that make Remastered and Reimagined easier or harder. Hard is the original game
/// (everything 1). Each coefficient is a multiplier: a player health of 1.25 makes the hero as
/// tough as if he had a quarter more health, and an enemy damage of 0.75 makes enemy hits a
/// quarter weaker.
public static class Difficulty
{
    /// The four coefficients of one level.
    public readonly record struct Factors(double EnemyHealth, double EnemyDamage, double PlayerHealth, double PlayerDamage);

    private static readonly Factors[] FACTORS =
    [
        new(0.40, 0.25, 3.00, 2.00), // Explorer: extremely easy
        new(0.70, 0.50, 1.50, 1.50), // Easy
        new(0.80, 0.75, 1.25, 1.25), // Normal: easier than the original
        new(1.00, 1.00, 1.00, 1.00), // Hard: the original game's numbers
        new(1.30, 1.25, 0.90, 0.90), // Very Hard
        new(1.75, 1.60, 0.75, 0.80), // Extreme
    ];

    private static readonly string[] NAMES = ["Explorer", "Easy", "Normal", "Hard", "Very Hard", "Extreme"];

    /// The level a new game starts at (and keeps for the rest of the session once changed).
    public static DifficultyLevel Chosen { get; set; } = DifficultyLevel.Normal;

    public static Factors FactorsOf(DifficultyLevel level) => FACTORS[(int)level];

    public static string NameOf(DifficultyLevel level) => NAMES[(int)level];

    /// The next level, going round from Extreme back to Explorer.
    public static DifficultyLevel Next(DifficultyLevel level) => (DifficultyLevel)(((int)level + 1) % FACTORS.Length);

    /// How much a hit is changed: a hit on the hero is enemy damage over player health, a hit by the
    /// hero is player damage over enemy health, and monsters hitting each other are not changed.
    /// Original always plays at 1.
    public static double DamageFactor(Data.Game game, int attacker, int target)
    {
        if (game.Experience == Experience.Original)
        {
            return 1;
        }

        Factors factors = FactorsOf(game.Difficulty);

        if (target == 0)
        {
            return factors.EnemyDamage / factors.PlayerHealth;
        }

        // A hit by the hero, or by a missile (negative attacker, shot by the hero when it lands on
        // someone else). Monsters hitting each other are left alone.
        return attacker <= 0 ? factors.PlayerDamage / factors.EnemyHealth : 1;
    }

    /// The damage after the difficulty. The part of a point that does not come out whole is kept for
    /// the next hit on the same actor, so a weak hit is not lost and not always rounded up.
    public static short ScaleDamage(Data.Game game, int attacker, int target, short damage)
    {
        double factor = DamageFactor(game, attacker, target);

        if (factor == 1 || target < 0 || target >= game.DamageCarry.Length)
        {
            return damage;
        }

        double exact = damage * factor + game.DamageCarry[target];
        short whole = (short)Math.Floor(exact);

        game.DamageCarry[target] = exact - whole;

        return whole;
    }
}
