namespace FaeryTale.Gfx;

/// The look shared by the menus drawn on wooden boards (the launch menu and the game menu):
/// sizes, colours, and putting text and boards on an overlay.
internal static class MenuStyle
{
    /// Titles are drawn at the size of the classic game text, descriptions at half that.
    public const int TITLE_SCALE = 2;

    /// A title is this far above its baseline.
    public const int TITLE_ASCENT = 14 * TITLE_SCALE / 2;

    /// Space between the text and the edge of its board.
    public const int PANEL_PADDING = 8;

    // Colours of the text on the wood: cream, gold for the heading, yellow under the pointer, with
    // a dark shadow behind each letter so it can be read on the grain.
    public const uint TITLE_COLOR = 0xffff_f4dc;
    public const uint HEADING_COLOR = 0xffff_e08a;
    public const uint HOVER_COLOR = 0xffff_e566;
    public const uint DESCRIPTION_COLOR = 0xffee_dcb8;
    public const uint LIT_DESCRIPTION_COLOR = Overlay.WHITE;
    private const uint TEXT_SHADOW_COLOR = 0xff1e_1006;

    /// A plain and a lit board for an area (left, top, width, height).
    public static Image[] Board((int Left, int Top, int Width, int Height) area, uint seed)
    {
        Image Make(uint[] pixels) => new(area.Left, area.Top, area.Width + Wood.SHADOW, area.Height + Wood.SHADOW, pixels);

        return [Make(Wood.Plank(area.Width, area.Height, seed)), Make(Wood.LitPlank(area.Width, area.Height, seed))];
    }

    /// Adds a line of text in the middle of the screen with its baseline at y.
    public static void AddCentered(Overlay overlay, string text, int y, uint color, int scale) =>
        AddText(overlay, text, ((Overlay.WIDTH - Overlay.TextWidth(text, scale)) / 2, y), color, scale);

    /// Adds a line of text with a dark shadow behind it, down and to the right.
    public static void AddText(Overlay overlay, string text, (int X, int Y) at, uint color, int scale)
    {
        overlay.Texts.Add(new Text(text, at.X + scale, at.Y + scale, TEXT_SHADOW_COLOR, scale));
        overlay.Texts.Add(new Text(text, at.X, at.Y, color, scale));
    }
}
