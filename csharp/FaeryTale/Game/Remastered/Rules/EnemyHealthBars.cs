using FaeryTale.Game.Data;

namespace FaeryTale.Game.Remastered;

/// A health bar over each enemy the hero has hurt lately (Remastered and Reimagined). The game keeps
/// only an enemy's health now, not what it started with, so the most seen so far stands for the full
/// bar. The bar shows for a few seconds after each hit and goes away with the enemy. The player can
/// turn the bars off in the game menu; the choice is kept for the session.
public static class EnemyHealthBars
{
    /// One bar: where the enemy stands in the world, and how much of its health is left (0 to 1).
    public readonly record struct Bar(int X, int Y, double Fraction);

    // How long a bar stays after the enemy was last hurt (in 50ths of a second).
    private const uint SHOWN_TICKS = 250;

    // The bar floats this far above the enemy's feet.
    private const int HEIGHT_ABOVE_FEET = 40;

    private const int MAX_ACTORS = 32;

    private static readonly short[] mostHealth = new short[MAX_ACTORS];
    private static readonly short[] lastHealth = new short[MAX_ACTORS];
    private static readonly uint[] hurtAt = new uint[MAX_ACTORS];

    public static bool Enabled { get; set; } = true;

    /// The menu entry's words: what clicking it does.
    public static string MenuText => Enabled ? "Hide Health Bars" : "Show Health Bars";

    /// Turns the bars off if they are on, and on if they are off.
    public static void Toggle() => Enabled = !Enabled;

    /// Looks at every enemy this frame and returns the bars to draw (none when they are off).
    public static List<Bar> Update(Data.Game game, uint now)
    {
        List<Bar> bars = [];

        for (int number = 1; number < Math.Min((int)game.Actors.Drawn, MAX_ACTORS); number++)
        {
            Shape enemy = game.Actors.List[number];

            if (!EnemyTargets.IsAttackable(enemy))
            {
                Forget(number);

                continue;
            }

            Watch(number, enemy.Vitality, now);

            if (Enabled && now - hurtAt[number] < SHOWN_TICKS && mostHealth[number] > 0)
            {
                bars.Add(new Bar(enemy.AbsX, enemy.AbsY - HEIGHT_ABOVE_FEET, Math.Clamp((double)enemy.Vitality / mostHealth[number], 0, 1)));
            }
        }

        return bars;
    }

    // Notes an enemy's health: more than before means a new enemy (or healing) so the full bar is that
    // much; less means a hit just now.
    private static void Watch(int number, short health, uint now)
    {
        if (health > mostHealth[number])
        {
            mostHealth[number] = health;
        }
        else if (health < lastHealth[number])
        {
            hurtAt[number] = now;
        }

        lastHealth[number] = health;
    }

    // The slot has no live enemy: the next one to take it starts fresh.
    private static void Forget(int number)
    {
        mostHealth[number] = 0;
        lastHealth[number] = 0;
        hurtAt[number] = 0;
    }
}
