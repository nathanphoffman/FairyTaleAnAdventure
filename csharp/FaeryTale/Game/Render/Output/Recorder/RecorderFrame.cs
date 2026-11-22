using FaeryTale.Gfx;

namespace FaeryTale.Game.Output;

/// Recorder: frame timing, page swaps and the overlay.
public sealed partial class Recorder
{
    public void SwapPages() => Record("SwapPages");

    public void Present() => Record("Present");

    public void WaitFrame()
    {
        Ticks++;
        Record("WaitFrame");
    }

    public double AnimationFrameMs => 20;

    public void ShowAnimationFrame() => WaitFrame();

    public void SetOverlay(Overlay overlay) =>
        Record("SetOverlay", $"{overlay.Shades.Count} shades", $"{overlay.Images.Count} images", $"{overlay.Texts.Count} texts");
}
