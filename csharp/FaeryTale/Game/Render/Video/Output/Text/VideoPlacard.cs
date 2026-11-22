using FaeryTale.Game.Files;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Video: fonts, the placard and the menu entries.
public sealed partial class Video
{
    public void UseStoryFont() => Screen.SetFontKind(Screen.CurrentPort, FontKind.Story);

    public void UsePanelFont() => Screen.SetFontKind(Screen.CurrentPort, FontKind.Panel);

    public void ClearPlacard()
    {
        PortId port = Screen.CurrentPort;

        Screen.SetAPen(port, PEN_BLACK);
        Screen.RectFill(port, PLACARD_TOP_LEFT, PLACARD_BOTTOM_RIGHT);
        Screen.SetAPen(port, PLACARD_TEXT_PEN);
    }

    public void DrawPlacardBorder() => PlacardBorder.DrawPlacardBorder(Screen);

    public void DrawMenuEntry(int slot, ReadOnlySpan<byte> label, bool selected, byte boxPen)
    {
        (int x, int y) = MenuSlotPosition(slot);

        Screen.SetAPen(PortId.Text2, selected ? (byte)1 : (byte)0);
        Screen.SetBPen(PortId.Text2, boxPen);
        Screen.MoveTo(PortId.Text2, x, y);
        Screen.Text(PortId.Text2, MENU_BOX);
        Screen.MoveTo(PortId.Text2, x + MENU_TEXT_INSET, y);
        Screen.Text(PortId.Text2, label);
    }

    public void BlankMenuEntry(int slot)
    {
        (int x, int y) = MenuSlotPosition(slot);

        Screen.SetBPen(PortId.Text2, PEN_BLACK);
        Screen.MoveTo(PortId.Text2, x, y);
        Screen.Text(PortId.Text2, MENU_BOX);
    }
}
