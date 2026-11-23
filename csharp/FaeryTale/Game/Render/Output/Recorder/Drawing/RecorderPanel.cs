using FaeryTale.Game.Data;

namespace FaeryTale.Game.Output;

/// Recorder: menu entries, compass, item list and overview map.
public sealed partial class Recorder
{
    public void DrawMenuEntry(int slot, ReadOnlySpan<byte> label, bool selected, byte boxPen) =>
        Record("DrawMenuEntry", slot, Quote(TextEncoding.FromBytes(label)), selected, boxPen);

    public void BlankMenuEntry(int slot) => Record("BlankMenuEntry", slot);

    public void DrawCompass(short direction) => Record("DrawCompass", direction);

    public void DrawItemList(Hero hero) => Record("DrawItemList");

    public void DrawOverviewMap(MapData map, World world, int mx, int my) => Record("DrawOverviewMap", mx, my);

    public void MarkOverviewMap((int X, int Y)? offset) =>
        Record("MarkOverviewMap", offset is { } place ? $"({place.X}, {place.Y})" : "none");
}
