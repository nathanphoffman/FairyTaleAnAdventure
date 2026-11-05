using System.Diagnostics;

namespace FaeryTale.Platform;

/// The slowest times of each second for FTA_FPS_LOG=1: making a picture, showing it, and the game
/// step between pictures, with the garbage collections and the time they paused the program.
public sealed class FrameTimings
{
    private double slowestMake;
    private double slowestShow;
    private double slowestStep;
    private int collectionsAtLastReport;
    private TimeSpan pauseAtLastReport;

    public void RecordMake(double milliseconds) => slowestMake = Math.Max(slowestMake, milliseconds);

    public void RecordShow(double milliseconds) => slowestShow = Math.Max(slowestShow, milliseconds);

    public void RecordStep(double milliseconds) => slowestStep = Math.Max(slowestStep, milliseconds);

    /// One line of text for the last second, and the start of the next.
    public string TakeReport()
    {
        int collections = GC.CollectionCount(0) + GC.CollectionCount(1) + GC.CollectionCount(2);
        TimeSpan pause = GC.GetTotalPauseDuration();
        string text = $"slowest: make {slowestMake:F1} ms, show {slowestShow:F1} ms, game step {slowestStep:F1} ms; "
            + $"{collections - collectionsAtLastReport} garbage collections ({(pause - pauseAtLastReport).TotalMilliseconds:F1} ms paused)";

        slowestMake = 0;
        slowestShow = 0;
        slowestStep = 0;
        collectionsAtLastReport = collections;
        pauseAtLastReport = pause;

        return text;
    }

    /// Milliseconds on a stopwatch since a start timestamp.
    public static double Since(long start) => Stopwatch.GetElapsedTime(start).TotalMilliseconds;
}
