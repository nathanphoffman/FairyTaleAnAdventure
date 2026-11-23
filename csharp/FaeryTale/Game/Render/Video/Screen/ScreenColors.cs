using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Screen: view ports, colors and the scene.
public sealed partial class Screen
{
    public ViewPortState GetViewPort(ViewPortId identifier) => identifier == ViewPortId.Page ? VpPage : VpText;

    /// Loads the first colours of a view port's table from colors (12 bit colours, 0x0RGB).
    public void LoadColors(ViewPortId identifier, ReadOnlySpan<ushort> colors)
    {
        ushort[] table = GetViewPort(identifier).Colors;

        colors[..Math.Min(table.Length, colors.Length)].CopyTo(table);
    }

    /// Sets one colour of a view port from its red, green and blue (4 bits each).
    public void SetColor(ViewPortId identifier, int n, (uint R, uint G, uint B) color)
    {
        ushort[] table = GetViewPort(identifier).Colors;

        if (n >= 0 && n < table.Length)
        {
            // Packs the three components into one 12 bit colour: each masked to its width, then moved to its place.
            table[n] = (ushort)(((color.R & COMPONENT_MASK) << RED_SHIFT) | ((color.G & COMPONENT_MASK) << GREEN_SHIFT) | (color.B & COMPONENT_MASK));
        }
    }

    /// What is on screen now: the text panel on top of the play field, unless hidden.
    public Scene ToScene()
    {
        List<ViewPort> viewPorts = [];

        foreach (ViewPortState state in new[] { VpText, VpPage })
        {
            if (state.Ras is { } ras && (state.Modes & VP_HIDE) == 0 && state.Colors.Length > 0)
            {
                viewPorts.Add(MakeViewPort(state, ras));
            }
        }

        return new Scene { ViewPorts = viewPorts, Pointer = PointerImage is null ? null : new Pointer(PointerImage, PointerPosition) };
    }
}
