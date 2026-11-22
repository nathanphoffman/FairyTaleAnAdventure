using FaeryTale.Gfx;

namespace FaeryTale.Game.Output;

/// Recorder: text position, fonts and the placard.
public sealed partial class Recorder
{
    public void SetTextPosition(int x, int y) => Record("SetTextPosition", x, y);

    public void UseStoryFont() => Record("UseStoryFont");

    public void UsePanelFont() => Record("UsePanelFont");

    public void SelectMessageArea() => Record("SelectMessageArea");

    public void ClearPlacard() => Record("ClearPlacard");

    public void DrawPlacardBorder() => Record("DrawPlacardBorder");
}
