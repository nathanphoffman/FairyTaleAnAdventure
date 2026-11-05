namespace FaeryTale.Game.Land.Map;

/// Stepping a position in one of the 8 directions, with the 16-bit wrap-around of the original.
public static class MapSteps
{
    /// World coordinates are 15 bit.
    private const int COORD_MASK = 0x7fff;
    private const int COORD_SIGN = 0x8000;

    // Steps for the 8 directions (and two "no move" entries) per half-unit of speed.
    internal static readonly short[] X_STEPS = [-2, 0, 2, 3, 2, 0, -2, -3, 0, 0];
    internal static readonly short[] Y_STEPS = [-2, -3, -2, 0, 2, 3, 2, 0, 0, 0];

    // Directions are the low byte of the value; a step is half the table value times the speed.
    private const int DIRECTION_MASK = 0xff;
    private const uint WORD_MASK = 0xffff;
    private const uint HIGH_WORD_MASK = 0xffff_0000;
    private const uint SIGN_BIT_14 = 0x4000;
    private const uint SIGN_BIT_15 = 0x8000;
    private const uint LOW_15_BITS = 0x7fff;

    /// Directions above this mean "no move".
    private const int MAX_DIRECTION = 7;

    // Moves value one step in direction dir at speed; the result stays within 15 bits. The step is
    // table value * speed / 2, so speed 2 moves the table's amount in pixels.
    private static int Step(int value, int dir, int speed, short[] table)
    {
        int product = table[dir & DIRECTION_MASK] * (short)speed;
        int half = (ushort)product >> 1;

        return (ushort)(value + half) & COORD_MASK;
    }

    /// The x position after one step in direction dir (0-7) at speed. Directions above 7 mean "no
    /// move". Directions go clockwise from north-west.
    public static int StepX(int x, int dir, int speed)
    {
        if ((dir & DIRECTION_MASK) > MAX_DIRECTION)
        {
            return x;
        }

        return Step(x, dir, speed, X_STEPS);
    }

    /// Like StepX, but for y; it keeps bit 15 of y (a flag the game stores there).
    public static int StepY(int y, int dir, int speed)
    {
        if ((dir & DIRECTION_MASK) > MAX_DIRECTION)
        {
            return y;
        }

        return Step(y, dir, speed, Y_STEPS) | (y & COORD_SIGN);
    }

    /// Copies bit 14 into bit 15, giving a signed 15 bit value (the world wraps around at the edges).
    public static int WrapCoordinate(int x)
    {
        uint low = (uint)x & WORD_MASK;

        low = (low & SIGN_BIT_14) != 0 ? low | SIGN_BIT_15 : low & LOW_15_BITS;

        return unchecked((int)(((uint)x & HIGH_WORD_MASK) | low));
    }
}
