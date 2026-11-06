using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Platform;
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
    // The menu is laid out from the heading down, and the whole block is centred on the screen.
    private const int FIRST_ENTRY_BELOW_HEADING = 66;
    private const int ENTRY_SPACING = 44;
    private const int BOARD_BELOW_BASELINE = 16;
    private const int DESCRIPTION_BELOW_BOARDS = 24;
    private const int DESCRIPTION_DESCENT = 2;

    /// How dark the game looks behind the menu (0-255).
    private const byte BACKGROUND_DARKNESS = 170;

    private const int FULL_HEIGHT = 400;
    private const int HEADING_SEED = 41;

    /// "300 fps", the longest the frame rate entry gets.
    private const int MAX_RATE_TEXT_LENGTH = 7;
    private const int ENTRY_SEED = 51;

    // The entries, top to bottom, and what each does. The difficulty entry (DIFFICULTY_ENTRY) shows
    // the level and changes it when clicked; it does not close the menu.
    private static readonly (string Label, GameMenuChoice Choice)[] ENTRIES =
    [
        ("Back To Game", GameMenuChoice.Resume),
        ("Difficulty", GameMenuChoice.Resume),
        ("VSync", GameMenuChoice.Resume),
        ("Frame Rate", GameMenuChoice.Resume),
        ("Back To Game Selection", GameMenuChoice.BackToGameSelection),
        ("Exit Game", GameMenuChoice.ExitGame),
    ];

    private const int DIFFICULTY_ENTRY = 1;
    private const int VSYNC_ENTRY = 2;
    private const int FRAME_RATE_ENTRY = 3;
    private const string VSYNC_LABEL = "VSync: ";
    private const string FRAME_RATE_LABEL = "Frame Rate: ";
    private const string DIFFICULTY_LABEL = "Difficulty: ";

    /// The line under the entries that says what the difficulty does.
    private const int DESCRIPTION_SCALE = 1;
    private const uint DESCRIPTION_COLOR = 0xffee_dcb8;

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
            (int pointerX, int pointerY) = InputHandler.GridPointer;
            int? hovered = EntryAt(areas, pointerX, pointerY);

            video.SetOverlay(Draw(game, heading, boards, hovered));

            if (hovered is { } entry && InputHandler.LeftButtonDown)
            {
                if (entry is DIFFICULTY_ENTRY or VSYNC_ENTRY or FRAME_RATE_ENTRY)
                {
                    ChangeSetting(game, video, entry);

                    continue;
                }

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

    // A click on the difficulty entry makes it the next level (after Extreme comes Explorer), a click
    // on the vsync entry turns it on or off, one on the frame rate entry gives the next fixed rate. They are kept for
    // the rest of the session. The click is waited out so it counts once.
    private static void ChangeSetting(Data.Game game, Video video, int entry)
    {
        if (entry == DIFFICULTY_ENTRY)
        {
            game.Difficulty = Difficulty.Next(game.Difficulty);
            Difficulty.Chosen = game.Difficulty;
        }
        else if (entry == VSYNC_ENTRY)
        {
            WindowControls.ToggleVSync();
        }
        else
        {
            WindowControls.CycleFrameRate();
        }

        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }
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
        int longestLevel = Enum.GetValues<DifficultyLevel>().Max(level => Difficulty.NameOf(level).Length);
        int widest = Math.Max(ENTRIES.Max(entry => entry.Label.Length), DIFFICULTY_LABEL.Length + longestLevel);

        widest = Math.Max(widest, FRAME_RATE_LABEL.Length + MAX_RATE_TEXT_LENGTH);
        int width = widest * Overlay.GLYPH_SIZE * TITLE_SCALE + PANEL_PADDING * 2;

        return ENTRIES.Select((_, index) => ((Overlay.WIDTH - width) / 2, EntryBaseline(index) - TITLE_ASCENT - PANEL_PADDING, width, TITLE_ASCENT + PANEL_PADDING * 2 + Overlay.GLYPH_SIZE)).ToArray();
    }

    // Where the heading, the entries and the description go, so that all of them together (the top
    // of the heading's board to the bottom of the description) are in the middle of the screen.
    private static int HeadingY => (FULL_HEIGHT - (DescriptionBelowHeading + DESCRIPTION_DESCENT + HeadingBoardAbove)) / 2 + HeadingBoardAbove;

    private static int HeadingBoardAbove => TITLE_ASCENT + PANEL_PADDING;

    private static int DescriptionBelowHeading => FIRST_ENTRY_BELOW_HEADING + (ENTRIES.Length - 1) * ENTRY_SPACING + BOARD_BELOW_BASELINE + DESCRIPTION_BELOW_BOARDS;

    private static int DescriptionY => HeadingY + DescriptionBelowHeading;

    // The baseline of an entry's label.
    private static int EntryBaseline(int index) => HeadingY + FIRST_ENTRY_BELOW_HEADING + index * ENTRY_SPACING;

    private static (int Left, int Top, int Width, int Height) HeadingArea()
    {
        int width = Overlay.TextWidth(HEADING, TITLE_SCALE) + PANEL_PADDING * 2;

        return ((Overlay.WIDTH - width) / 2, HeadingY - TITLE_ASCENT - PANEL_PADDING, width, TITLE_ASCENT + PANEL_PADDING * 2);
    }

    // The darkened game, the heading and the entries (the one under the pointer lit).
    private static Overlay Draw(Data.Game game, Image heading, Image[][] boards, int? hovered)
    {
        Overlay overlay = new() { Shades = { new Shade(0, 0, Overlay.WIDTH, FULL_HEIGHT, BACKGROUND_DARKNESS) }, PointerOnTop = true, CenterOnScreen = true };

        overlay.Images.Add(heading);
        AddCentered(overlay, HEADING, HeadingY, HEADING_COLOR, TITLE_SCALE);

        for (int index = 0; index < ENTRIES.Length; index++)
        {
            bool lit = hovered == index;

            overlay.Images.Add(boards[index][lit ? 1 : 0]);
            AddCentered(overlay, LabelOf(game, index), EntryBaseline(index), lit ? HOVER_COLOR : TITLE_COLOR, TITLE_SCALE);
        }

        AddCentered(overlay, DescriptionOf(game.Difficulty), DescriptionY, DESCRIPTION_COLOR, DESCRIPTION_SCALE);

        return overlay;
    }

    // The text of an entry: the difficulty entry names the level, the frame rate entry the rate.
    private static string LabelOf(Data.Game game, int index) => index switch
    {
        DIFFICULTY_ENTRY => DIFFICULTY_LABEL + Difficulty.NameOf(game.Difficulty),
        VSYNC_ENTRY => VSYNC_LABEL + WindowControls.VSyncText,
        FRAME_RATE_ENTRY => FRAME_RATE_LABEL + WindowControls.FrameRateText,
        _ => ENTRIES[index].Label,
    };

    // What a level does, as percentages of the original game's numbers.
    private static string DescriptionOf(DifficultyLevel level)
    {
        Difficulty.Factors f = Difficulty.FactorsOf(level);

        return $"Enemies: {Percent(f.EnemyHealth)} health, {Percent(f.EnemyDamage)} damage    You: {Percent(f.PlayerHealth)} health, {Percent(f.PlayerDamage)} damage";
    }

    private static string Percent(double factor) => $"{Math.Round(factor * 100)}%";

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
