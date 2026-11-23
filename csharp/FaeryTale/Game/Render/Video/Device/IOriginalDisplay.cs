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
