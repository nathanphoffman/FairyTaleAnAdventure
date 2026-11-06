using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// Animations that are not tied to the game's 50 ticks a second (the page turn of the intro): a
/// picture for every refresh of the monitor.
public sealed partial class SdlDisplay
{
    /// A gap this long means the animation stopped and starts again, not that pictures are late.
    private const int RESTART_AFTER_FRAMES = 4;

    private double nextAnimationMs;

    public double AnimationFrameMs => RefreshIntervalMs();

    public void ShowAnimationFrame(Scene scene)
    {
        double interval = RefreshIntervalMs();
        double now = clock.Elapsed.TotalMilliseconds;

        if (now - nextAnimationMs > interval * RESTART_AFTER_FRAMES)
        {
            nextAnimationMs = now;
        }

        ApplyNativeMode();
        HandleEvents();
        Ticks += (uint)Math.Round(interval / TICK_MILLISECONDS);
        ApplyCursorSetting();
        Draw(scene);

        nextAnimationMs += interval;
        WaitUntil(nextAnimationMs);
    }
}
