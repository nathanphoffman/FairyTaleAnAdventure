namespace FaeryTale.Platform;

/// How far along its frame the picture being made is. A game frame lasts one or two ticks; the
/// movement of a frame is drawn over the time the frame before it lasted, starting when the frame
/// begins (the length of the new frame is not known until the next one arrives).
public sealed class FrameSpan
{
    private const double MIN_SPAN_MS = 20;
    private const double MAX_SPAN_MS = 80;
    private const double FIRST_SPAN_MS = 40;

    private double startMs;
    private double spanMs = FIRST_SPAN_MS;

    /// The game made a new frame at this time (the start of its first tick).
    public void Begin(double frameStartMs)
    {
        double span = frameStartMs - startMs;

        if (span >= MIN_SPAN_MS && span <= MAX_SPAN_MS)
        {
            spanMs = span;
        }

        startMs = frameStartMs;
    }

    /// 0 at the start of the frame up to 1 at the end of the time it is expected to last.
    public double AlphaAt(double timeMs) => Math.Clamp((timeMs - startMs) / spanMs, 0.0, 1.0);
}
