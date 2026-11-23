namespace FaeryTale.Game.Data;

/// The text printing queue: requests wait here until the screen code takes them.
public sealed class PrintQueue
{
    /// How many requests fit in the queue (a power of two, so the position can wrap with a mask).
    private const int CAPACITY = 32;

    public sbyte[] Items = new sbyte[CAPACITY];

    /// Where the next request is added, and where the next one is taken from.
    public short Write;
    public short Read;
}
