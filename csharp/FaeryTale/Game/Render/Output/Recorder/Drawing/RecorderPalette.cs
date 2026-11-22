using FaeryTale.Gfx;

namespace FaeryTale.Game.Output;

/// Recorder: loading colors.
public sealed partial class Recorder
{
    public void LoadPageColors(ReadOnlySpan<ushort> colors) => Record("LoadPageColors", Hex(colors));

    public void LoadPanelColors(ReadOnlySpan<ushort> colors) => Record("LoadPanelColors", Hex(colors));
}
