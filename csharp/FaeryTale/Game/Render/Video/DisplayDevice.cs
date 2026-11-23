using FaeryTale.Game.Remastered;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The parts of a display that every game uses: the clock, showing a scene, and animations.
public interface IOriginalDisplay
{
    /// Ticks of 1/50 second waited since the game started.
    uint Ticks { get; }

    /// Shows the scene now.
    void Present(Scene scene);

    /// Waits one tick of 1/50 second, showing the scene as it is now.
    void WaitFrame(Scene scene);

    /// How long a picture of an animation lasts: a refresh of the monitor in the window, a tick otherwise.
    double AnimationFrameMs { get; }

    /// Shows the scene with its overlay now and waits for the next picture of an animation, which
    /// does not keep to the game's 50 ticks a second.
    void ShowAnimationFrame(Scene scene);
}

/// The window the finished frames go to. The platform supplies the real one; the video only
/// describes what is on screen (a Scene) and asks for it to be shown. It is the original parts
/// and the Remastered parts together.
public interface IDisplayDevice : IOriginalDisplay, IRemasteredDisplay
{
}

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
