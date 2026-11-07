using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Platform;
using static FaeryTale.Gfx.MenuStyle;

namespace FaeryTale.Game.Remastered;

/// The Controls screen of the game menu: what each action's key is, and a click on an action asks for a
/// new key for it (Escape keeps the old one). The keys are saved in controls.cfg, so they stay for the
/// next run. The keys the game came with go on working too. The game waits meanwhile.
public static class ControlsMenu
{
    private const string HEADING = "Controls";
    private const byte BACKGROUND_DARKNESS = 200;
    private const int FULL_HEIGHT = 400;
    private const int COLUMNS = 2;
    private const int COLUMN_GAP = 10;
    private const int ENTRY_SCALE = 1;
    private const int ENTRY_ASCENT = 7;
    private const int ROW_SPACING = 34;
    private const int HEADING_GAP = 40;
    private const int HELP_GAP = 24;
    private const int ENTRY_SEED = 81;
    private const int HEADING_SEED = 91;
    private const int LABEL_CHARACTERS = 24;
    private const string ESCAPE_KEY_NAME = "Escape";

    // The actions: what they are called, the name of the key the game sees for them, and the keys it
    // comes with (for the screen to say).
    private static readonly (string Label, string GameKey, string Default)[] ACTIONS =
    [
        ("Move Up", "Up", "W / Up"),
        ("Move Down", "Down", "S / Down"),
        ("Move Left", "Left", "A / Left"),
        ("Move Right", "Right", "D / Right"),
        ("Fight", "Left Ctrl", "Ctrl"),
        ("Inventory", "I", "I"),
        ("Big Map", "M", "M"),
        ("Journal", "Tab", "Tab"),
    ];

    private const string RESET_LABEL = "Reset To Defaults";
    private const string BACK_LABEL = "Back";
    private static readonly int RESET_ENTRY = ACTIONS.Length;
    private static readonly int BACK_ENTRY = ACTIONS.Length + 1;

    /// Shows the screen until Back is clicked or Escape is pressed.
    public static void Show(Data.Game game, Video video)
    {
        (int Left, int Top, int Width, int Height)[] areas = Areas();
        Image[][] boards = areas.Select((area, index) => Board(area, ENTRY_SEED + (uint)index)).ToArray();
        Image heading = Board(HeadingArea(), HEADING_SEED)[0];
        string help = "Click an action, then press the key you want for it.";

        WaitForRelease(game, video);

        while (true)
        {
            int? hovered = GameMenu.EntryAt(areas, InputHandler.GridPointer.X, InputHandler.GridPointer.Y);

            video.SetOverlay(Draw(areas, boards, heading, hovered, help));

            if (hovered is { } entry && InputHandler.LeftButtonDown)
            {
                if (entry == BACK_ENTRY)
                {
                    break;
                }

                help = Choose(game, video, entry);
                WaitForRelease(game, video);
            }

            if (InputHandler.NextKey() == InputHandler.ESCAPE_KEY)
            {
                break;
            }

            Display.Delay(game, video, 1);
        }

        Leave(game, video);
    }

    // Does what a click on an entry says and returns the line of help to show after it.
    private static string Choose(Data.Game game, Video video, int entry)
    {
        if (entry == RESET_ENTRY)
        {
            KeyRemap.ResetAll();

            return "The keys are back to the ones the game came with.";
        }

        (string label, string gameKey, _) = ACTIONS[entry];

        return AskForKey(game, video, label, gameKey);
    }

    // Waits for a key and gives it to the action (Escape gives up).
    private static string AskForKey(Data.Game game, Video video, string label, string gameKey)
    {
        KeyRemap.StartCapture();

        string? key;

        while ((key = KeyRemap.TakeCaptured()) is null)
        {
            video.SetOverlay(WaitingOverlay(label));
            Display.Delay(game, video, 1);
        }

        InputHandler.ResetKeyBuffer();

        if (key == ESCAPE_KEY_NAME)
        {
            return "The key was not changed.";
        }

        KeyRemap.Bind(key, gameKey);

        return $"{label} is now {key}.";
    }

    private static Overlay WaitingOverlay(string label)
    {
        Overlay overlay = new() { Shades = { new Shade(0, 0, Overlay.WIDTH, FULL_HEIGHT, BACKGROUND_DARKNESS) }, PointerOnTop = true, CenterOnScreen = true };

        AddCentered(overlay, $"Press a key for {label}", FULL_HEIGHT / 2, TITLE_COLOR, TITLE_SCALE);
        AddCentered(overlay, "Escape keeps the old key", FULL_HEIGHT / 2 + HELP_GAP, DESCRIPTION_COLOR, ENTRY_SCALE);

        return overlay;
    }

    private static void WaitForRelease(Data.Game game, Video video)
    {
        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }
    }

    // The boards: two columns of the same width, a row at a time, under the heading.
    private static (int Left, int Top, int Width, int Height)[] Areas()
    {
        int width = LABEL_CHARACTERS * Overlay.GLYPH_SIZE * ENTRY_SCALE + PANEL_PADDING * 2;
        int left = (Overlay.WIDTH - COLUMNS * width - (COLUMNS - 1) * COLUMN_GAP) / 2;
        int rows = (ACTIONS.Length + 2 + COLUMNS - 1) / COLUMNS;
        int top = (FULL_HEIGHT - (rows * ROW_SPACING + HEADING_GAP + HELP_GAP)) / 2 + HEADING_GAP;

        return Enumerable.Range(0, ACTIONS.Length + 2)
            .Select(index => (left + index % COLUMNS * (width + COLUMN_GAP), top + index / COLUMNS * ROW_SPACING, width, ENTRY_ASCENT + PANEL_PADDING * 2 + Overlay.GLYPH_SIZE))
            .ToArray();
    }

    private static (int Left, int Top, int Width, int Height) HeadingArea()
    {
        int width = Overlay.TextWidth(HEADING, TITLE_SCALE) + PANEL_PADDING * 2;
        int top = Areas()[0].Top - HEADING_GAP;

        return ((Overlay.WIDTH - width) / 2, top, width, TITLE_ASCENT + PANEL_PADDING * 2);
    }

    // The text of an entry: an action and its key, or one of the two buttons.
    private static string LabelOf(int index)
    {
        if (index == RESET_ENTRY)
        {
            return RESET_LABEL;
        }

        if (index == BACK_ENTRY)
        {
            return BACK_LABEL;
        }

        (string label, string gameKey, string defaults) = ACTIONS[index];

        return $"{label}: {KeyRemap.KeyFor(gameKey) ?? defaults}";
    }

    private static Overlay Draw((int Left, int Top, int Width, int Height)[] areas, Image[][] boards, Image heading, int? hovered, string help)
    {
        Overlay overlay = new() { Shades = { new Shade(0, 0, Overlay.WIDTH, FULL_HEIGHT, BACKGROUND_DARKNESS) }, PointerOnTop = true, CenterOnScreen = true };

        overlay.Images.Add(heading);
        AddCentered(overlay, HEADING, HeadingArea().Top + TITLE_ASCENT + PANEL_PADDING / 2, HEADING_COLOR, TITLE_SCALE);

        for (int index = 0; index < areas.Length; index++)
        {
            string label = LabelOf(index);
            bool lit = hovered == index;

            overlay.Images.Add(boards[index][lit ? 1 : 0]);
            AddText(overlay, label, (areas[index].Left + (areas[index].Width - Overlay.TextWidth(label, ENTRY_SCALE)) / 2, areas[index].Top + PANEL_PADDING + ENTRY_ASCENT), lit ? HOVER_COLOR : TITLE_COLOR, ENTRY_SCALE);
        }

        AddCentered(overlay, help, areas[^1].Top + areas[^1].Height + HELP_GAP, DESCRIPTION_COLOR, ENTRY_SCALE);

        return overlay;
    }

    private static void Leave(Data.Game game, Video video)
    {
        WaitForRelease(game, video);
        video.SetOverlay(new Overlay());
        InputHandler.TakeClick();
        InputHandler.ResetKeyBuffer();
    }
}
