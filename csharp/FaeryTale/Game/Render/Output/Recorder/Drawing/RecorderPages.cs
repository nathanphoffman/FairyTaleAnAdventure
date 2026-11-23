
namespace FaeryTale.Game.Output;

/// Recorder: the drawing page and the panel.
public sealed partial class Recorder
{
    public void ClearDrawingPage() => Record("ClearDrawingPage");

    public void ResetPageScroll() => Record("ResetPageScroll");

    public void PrepareMessagePage() => Record("PrepareMessagePage");

    public void SetPanelVisible(bool visible) => Record("SetPanelVisible", visible);

    public void SetZoomSize(int positionX, int positionY) => Record("SetZoomSize", positionX, positionY);

    public void LoadPictureToDrawingPage(string name) => Record("LoadPictureToDrawingPage", Quote(name));
}
