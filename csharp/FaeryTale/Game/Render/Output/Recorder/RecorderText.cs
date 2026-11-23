
namespace FaeryTale.Game.Output;

/// Recorder: text position, fonts and the placard.
public sealed partial class Recorder
{
    public void SetTextPosition(int positionX, int positionY) => Record("SetTextPosition", positionX, positionY);

    public void UseStoryFont() => Record("UseStoryFont");

    public void UsePanelFont() => Record("UsePanelFont");

    public void SelectMessageArea() => Record("SelectMessageArea");

    public void ClearPlacard() => Record("ClearPlacard");

    public void DrawPlacardBorder() => Record("DrawPlacardBorder");
}
