using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// A "< Back" button above the classic buttons of the text panel (Remastered and Reimagined) while a
/// submenu is open, so it is clear how to get back to the first menu (the original way is to click
/// the lit name at the top). It is a wooden board in the style of the other new buttons, on the
/// 640 x 400 overlay grid; a click on it is not a click on the world.
public static class PanelBackButton
{
    // Just above the two columns of classic buttons, as wide as both.
    private const int LEFT = 434;
    private const int TOP = 266;
    private const int WIDTH = 100;
    private const int HEIGHT = 18;

    private const string LABEL = "< Back";
    private const uint BOARD_SEED = 0x2b7e;
    private const int TEXT_SCALE = 1;

    // The glyph's baseline is this far below the top of its box.
    private const int GLYPH_ASCENT = 7;

    // The first menu, where the Items buttons are (the one every other menu goes back to).
    private const int ITEMS_MENU = 0;
    private const sbyte VIEW_NORMAL = 0;

    private static Image[]? boards;

    /// Whether the button is shown: a submenu is open and the world is in view.
    public static bool IsShown(Data.Game game) =>
        game.Experience != Data.Experience.Original && game.Ui.CurrentMenu != ITEMS_MENU && game.Ui.ViewStatus == VIEW_NORMAL;

    /// Whether a place on the overlay grid is on the button.
    public static bool Contains((int X, int Y) place) =>
        place.X >= LEFT && place.X < LEFT + WIDTH && place.Y >= TOP && place.Y < TOP + HEIGHT;

    /// Presses the button if it is shown and the place is on it: back to the first menu. Returns
    /// whether it was pressed (the click is then used up).
    public static bool TryPress(Data.Game game, IOutput output, (int X, int Y) place)
    {
        if (!IsShown(game) || !Contains(place))
        {
            return false;
        }

        Menus.GoToMenu(game, output, ITEMS_MENU);

        return true;
    }

    /// The button as an overlay, lit when the pointer is over it.
    public static Overlay Of(bool hovered)
    {
        Overlay overlay = new();

        boards ??= MenuStyle.Board((LEFT, TOP, WIDTH, HEIGHT), BOARD_SEED);
        overlay.Images.Add(boards[hovered ? 1 : 0]);

        int textLeft = LEFT + (WIDTH - Overlay.TextWidth(LABEL, TEXT_SCALE)) / 2;
        int baseline = TOP + (HEIGHT - Overlay.GLYPH_SIZE) / 2 + GLYPH_ASCENT;

        MenuStyle.AddText(overlay, LABEL, (textLeft, baseline), hovered ? MenuStyle.HOVER_COLOR : MenuStyle.TITLE_COLOR, TEXT_SCALE);

        return overlay;
    }
}
