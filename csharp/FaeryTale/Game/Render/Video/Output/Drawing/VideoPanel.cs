using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Video: the compass, item list, overview map and panel.
public sealed partial class Video
{
    public void DrawCompass(short dir) => ScreenEffects.DrawCompass(Screen, dir);

    public void DrawItemList(Hero hero)
    {
        Capture.WorldVisible = false;
        ScreenEffects.DrawItemList(this, hero);
    }

    public void DrawOverviewMap(MapData map, World world, int mx, int my)
    {
        Capture.WorldVisible = false;
        Screen.DrawTarget = Screen.DrawingBitmapId;
        OverviewMap.DrawOverviewMap(this, map, world, mx, my);
    }

    public void MarkOverviewMap((int X, int Y)? at)
    {
        Screen.GetPort(PortId.Map).Target = Screen.DrawTarget;
        Screen.SetDrawMode(PortId.Map, Drawing.JAM1);
        Screen.SetAPen(PortId.Map, MAP_MARK_PEN);

        if (at is { } place)
        {
            Screen.MoveTo(PortId.Map, place.X, place.Y);
            Screen.Text(PortId.Map, "+"u8);
        }
    }

    public void SetPanelVisible(bool visible) =>
        Screen.VpText.Modes = visible ? (ushort)(Screen.HIRES | Screen.SPRITES) : (ushort)(Screen.HIRES | Screen.SPRITES | Screen.VP_HIDE);

    // The top left corner of menu entry slot: even slots on the left, odd on the right.
    private static (int X, int Y) MenuSlotPosition(int slot) =>
        ((slot & 1) != 0 ? MENU_RIGHT_COLUMN_X : MENU_LEFT_COLUMN_X, slot / 2 * MENU_ROW_HEIGHT + MENU_FIRST_ROW_Y);
}
