using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// A display that shows nothing and only counts ticks, for runs without a window.
public sealed class NullDisplayDevice : IDisplayDevice
{
    // A picture of an animation lasts one tick of 1/50 second.
    private const double TICK_MILLISECONDS = 20;

    public Video? Source { get; set; }

    public uint Ticks { get; private set; }

    public void Present(Scene scene)
    {
    }

    public void WaitFrame(Scene scene) => Ticks++;

    public void SetOverlay(Overlay overlay)
    {
    }

    public double AnimationFrameMs => TICK_MILLISECONDS;

    public void ShowAnimationFrame(Scene scene) => Ticks++;
}
