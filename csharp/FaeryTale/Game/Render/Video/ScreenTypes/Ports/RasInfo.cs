using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Which part of which bitmap a view port shows.
public struct RasInfo
{
    public BitmapId Bitmap;
    public short X;
    public short Y;
}
