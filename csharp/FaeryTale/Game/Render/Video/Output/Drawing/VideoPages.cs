using FaeryTale.Game.Files;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Video: the drawing page, its scroll and colors.
public sealed partial class Video
{
    public void ClearDrawingPage()
    {
        Capture.WorldVisible = false;
        Screen.GetPort(PortId.Map).Target = Screen.DrawingBitmapId;
        Screen.Clear(PortId.Map, PEN_BLACK);
        Screen.GetPort(PortId.Map).Target = Screen.ViewingBitmapId;
    }

    public void ResetPageScroll()
    {
        int ras = Screen.DrawingPage.Ras;

        Screen.Ras[ras].X = 0;
        Screen.Ras[ras].Y = 0;
    }

    public void PrepareMessagePage()
    {
        Capture.WorldVisible = false;
        Screen.CurrentPort = PortId.Map;
        Screen.GetPort(PortId.Map).Target = Screen.DrawingBitmapId;
        Screen.SetDrawMode(PortId.Map, Drawing.JAM1);
        Screen.SetAPen(PortId.Map, PLACARD_TEXT_PEN);
        Screen.Clear(PortId.Map, 0);
    }

    public void SetZoomSize(int x, int y)
    {
        short left = (short)(SCREEN_CENTER_X - x);
        short top = (short)(SCREEN_CENTER_Y - y);

        for (int ras = 0; ras < 2; ras++)
        {
            Screen.Ras[ras].X = left;
            Screen.Ras[ras].Y = top;
        }

        Screen.VpPage.X = left;
        Screen.VpPage.Width = (short)(x + x);
        Screen.VpPage.Y = top;
        Screen.VpPage.Height = (short)(y + y);

        Screen.VpText.X = 0;
        Screen.VpText.Y = 0;
        Screen.VpText.Height = (short)(ZOOM_TEXT_HEIGHT - y);
    }

    public void LoadPictureToDrawingPage(string name)
    {
        BitmapId id = Screen.DrawingBitmapId;

        Screen.DrawTarget = id;
        IffPicture.LoadPicture(name, Screen.GetBitmap(id), 0, 0);
    }

    public void LoadPanelColors(ReadOnlySpan<ushort> colors) => Screen.LoadColors(ViewPortId.Text, colors);
}
