using FaeryTale.Game.Files;
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

    public void SetZoomSize(int positionX, int positionY)
    {
        short left = (short)(SCREEN_CENTER_X - positionX);
        short topEdge = (short)(SCREEN_CENTER_Y - positionY);

        for (int ras = 0; ras < 2; ras++)
        {
            Screen.Ras[ras].X = left;
            Screen.Ras[ras].Y = topEdge;
        }

        Screen.VpPage.X = left;
        Screen.VpPage.Width = (short)(positionX + positionX);
        Screen.VpPage.Y = topEdge;
        Screen.VpPage.Height = (short)(positionY + positionY);

        Screen.VpText.X = 0;
        Screen.VpText.Y = 0;
        Screen.VpText.Height = (short)(ZOOM_TEXT_HEIGHT - positionY);
    }

    public void LoadPictureToDrawingPage(string name)
    {
        BitmapId identifier = Screen.DrawingBitmapId;

        Screen.DrawTarget = identifier;
        IffPicture.LoadPicture(name, Screen.GetBitmap(identifier), 0, 0);
    }

    public void LoadPanelColors(ReadOnlySpan<ushort> colors) => Screen.LoadColors(ViewPortId.Text, colors);
}
