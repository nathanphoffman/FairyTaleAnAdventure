using FaeryTale.Game.Data;

namespace FaeryTale.Game.Output;

/// Recorder: menu entries, compass, item list and overview map.
public sealed partial class Recorder
{
    public void DrawMenuEntry(int slot, ReadOnlySpan<byte> label, bool selected, byte boxPen) =>
        Record("DrawMenuEntry", slot, Quote(TextEncoding.FromBytes(label)), selected, boxPen);

    public void BlankMenuEntry(int slot) => Record("BlankMenuEntry", slot);

    public void DrawCompass(short dir) => Record("DrawCompass", dir);

    public void DrawItemList(Hero hero) => Record("DrawItemList");

    public void DrawOverviewMap(MapData map, World world, int mx, int my) => Record("DrawOverviewMap", mx, my);

    public void MarkOverviewMap((int X, int Y)? at) =>
        Record("MarkOverviewMap", at is { } place ? $"({place.X}, {place.Y})" : "none");
}
