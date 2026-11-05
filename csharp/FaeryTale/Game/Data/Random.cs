namespace FaeryTale.Game.Data;

/// The game's own random number generator (from fsubs.asm). It must match the original exactly:
/// the tests replay recorded sessions and compare the game state, which only works if the same
/// seed gives the same numbers. It is a small linear congruential generator: multiply the seed by
/// 45821, add 1, then rotate the bits to get the result.
public sealed class Rng
{
    private const uint SEED_MULTIPLIER = 45821;
    private const uint RESULT_MASK = 0x7fff_ffff;
    private const int ROTATE_BITS = 6;
    private const int LOW_WORD_MASK = 0xffff;

    /// Only Seed1 is used; Seed2 is saved with the game.
    public int Seed1 = 19837325;
    public int Seed2 = 23098324;

    /// Steps the generator once: returns the new seed and the random value.
    public static (int Seed, int Value) Step(int seed)
    {
        // The 68000 mulu.w instruction multiplies only the low 16 bits of the seed.
        uint product = unchecked((ushort)seed * SEED_MULTIPLIER);
        int next = unchecked((int)product + 1);
        uint rotated = uint.RotateRight((uint)next, ROTATE_BITS);

        return (next, (int)(rotated & RESULT_MASK));
    }

    /// A value in 0..n for n up to 16 bits, as the 68000 divu instruction did it: only the low 16
    /// bits of the value and of n are used.
    public static int Bounded(int value, int n) => (ushort)value % (n & LOW_WORD_MASK);

    /// The next random number (0 to 2^31 - 1).
    public int Next()
    {
        (int next, int value) = Step(Seed1);
        Seed1 = next;

        return value;
    }

    /// A random number with only the bits in mask kept, e.g. BitRand(0x3fff) is 0 to 16383.
    public int BitRand(int mask) => Next() & mask;

    /// Random 0 or 1.
    public int Rand2() => Next() & 1;

    /// Random 0 to 3.
    public int Rand4() => Next() & 3;

    /// Random 0 to 7 (also used as a random compass direction).
    public int Rand8() => Next() & 7;

    /// Random 0 to 63.
    public int Rand64() => Next() & 63;

    /// Random 0 to 255.
    public int Rand256() => Next() & 255;

    /// A random number from 0 to n - 1, for any n up to 16 bits.
    public int Rnd(int n) => Bounded(Next(), n);
}
