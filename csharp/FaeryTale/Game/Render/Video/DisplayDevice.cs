using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The window the finished frames go to. The platform supplies the real one; the video only
/// describes what is on screen (a Scene) and asks for it to be shown.
public interface IDisplayDevice
{
    /// Ticks of 1/50 second waited since the game started.
    uint Ticks { get; }

    /// Shows the scene now.
    void Present(Scene scene);

    /// Waits one tick of 1/50 second, showing the scene as it is now.
    void WaitFrame(Scene scene);

    /// Sets the text and dark panels drawn over the picture (an empty overlay removes them).
    void SetOverlay(Overlay overlay);
}

/// A display that shows nothing and only counts ticks, for runs without a window.
public sealed class NullDisplayDevice : IDisplayDevice
{
    public uint Ticks { get; private set; }

    public void Present(Scene scene)
    {
    }

    public void WaitFrame(Scene scene) => Ticks++;

    public void SetOverlay(Overlay overlay)
    {
    }
}
