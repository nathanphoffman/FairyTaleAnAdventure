using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using static FaeryTale.Gfx.MenuStyle;

namespace FaeryTale.Game.Remastered;

/// What the player chose in the game menu.
public enum GameMenuChoice
{
    /// Close the menu and carry on playing.
    Resume,

    /// Leave this game and go back to the choice of Original, Remastered or Reimagined.
    BackToGameSelection,

    /// Leave the program.
    ExitGame,
}

/// The menu Escape opens in Remastered and Reimagined: wooden boards in the style of the launch
/// menu over the game, which is darkened and stands still behind it. Escape again closes it.
public static class GameMenu
{
    private const string HEADING = "Game Menu";
    private const int HEADING_Y = 120;
    private const int FIRST_ENTRY_Y = 190;
    private const int ENTRY_SPACING = 56;

    /// How dark the game looks behind the menu (0-255).
    private const byte BACKGROUND_DARKNESS = 170;

    private const int FULL_HEIGHT = 400;
    private const int HEADING_SEED = 41;
    private const int ENTRY_SEED = 51;

    // The entries, top to bottom, and what each does.
    private static readonly (string Label, GameMenuChoice Choice)[] ENTRIES =
    [
        ("Back To Game", GameMenuChoice.Resume),
        ("Back To Game Selection", GameMenuChoice.BackToGameSelection),
        ("Exit Game", GameMenuChoice.ExitGame),
    ];

    /// Shows the menu until the player picks something. The game must not run meanwhile.
    public static GameMenuChoice Show(Data.Game game, Video video)
    {
        (int Left, int Top, int Width, int Height)[] areas = EntryAreas();
        Image[][] boards = areas.Select((area, index) => Board(area, ENTRY_SEED + (uint)index)).ToArray();
        Image heading = Board(HeadingArea(), HEADING_SEED)[0];
        GameMenuChoice choice;

        ClickToMove.Cancel();

        while (true)
        {
            (int pointerX, int pointerY) = InputHandler.RawPointer;
            int? hovered = EntryAt(areas, pointerX * 2, pointerY * 2);

            video.SetOverlay(Draw(heading, boards, hovered));

            if (hovered is { } entry && InputHandler.LeftButtonDown)
            {
                choice = ENTRIES[entry].Choice;

                break;
            }

            if (InputHandler.NextKey() == InputHandler.ESCAPE_KEY)
            {
                choice = GameMenuChoice.Resume;

                break;
            }

            Display.Delay(game, video, 1);
        }

        Leave(game, video);

        return choice;
    }

    /// The entry whose board the point (overlay pixels) is on, if any.
    internal static int? EntryAt((int Left, int Top, int Width, int Height)[] areas, int x, int y)
    {
        for (int index = 0; index < areas.Length; index++)
        {
            (int left, int top, int width, int height) = areas[index];

            if (x >= left && x < left + width && y >= top && y < top + height)
            {
                return index;
            }
        }

        return null;
    }

    /// The board of each entry: all as wide as the longest label, centred.
    internal static (int Left, int Top, int Width, int Height)[] EntryAreas()
    {
        int width = ENTRIES.Max(entry => Overlay.TextWidth(entry.Label, TITLE_SCALE)) + PANEL_PADDING * 2;

        return ENTRIES.Select((_, index) => ((Overlay.WIDTH - width) / 2, EntryBaseline(index) - TITLE_ASCENT - PANEL_PADDING, width, TITLE_ASCENT + PANEL_PADDING * 2 + Overlay.GLYPH_SIZE)).ToArray();
    }

    // The baseline of an entry's label.
    private static int EntryBaseline(int index) => FIRST_ENTRY_Y + index * ENTRY_SPACING;

    private static (int Left, int Top, int Width, int Height) HeadingArea()
    {
        int width = Overlay.TextWidth(HEADING, TITLE_SCALE) + PANEL_PADDING * 2;

        return ((Overlay.WIDTH - width) / 2, HEADING_Y - TITLE_ASCENT - PANEL_PADDING, width, TITLE_ASCENT + PANEL_PADDING * 2);
    }

    // The darkened game, the heading and the entries (the one under the pointer lit).
    private static Overlay Draw(Image heading, Image[][] boards, int? hovered)
    {
        Overlay overlay = new() { Shades = { new Shade(0, 0, Overlay.WIDTH, FULL_HEIGHT, BACKGROUND_DARKNESS) }, PointerOnTop = true };

        overlay.Images.Add(heading);
        AddCentered(overlay, HEADING, HEADING_Y, HEADING_COLOR, TITLE_SCALE);

        for (int index = 0; index < ENTRIES.Length; index++)
        {
            bool lit = hovered == index;

            overlay.Images.Add(boards[index][lit ? 1 : 0]);
            AddCentered(overlay, ENTRIES[index].Label, EntryBaseline(index), lit ? HOVER_COLOR : TITLE_COLOR, TITLE_SCALE);
        }

        return overlay;
    }

    // Takes the menu off the screen and makes sure the click or key that closed it does not carry
    // on into the game.
    private static void Leave(Data.Game game, Video video)
    {
        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }

        video.SetOverlay(new Overlay());
        InputHandler.TakeClick();
        InputHandler.ResetKeyBuffer();
        InputHandler.ForgetLastMenu();
    }
}
