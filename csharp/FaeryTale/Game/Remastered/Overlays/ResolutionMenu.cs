using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Platform;
using static FaeryTale.Gfx.MenuStyle;

namespace FaeryTale.Game.Remastered;

/// The list of window sizes that the game menu's Resolution entry opens (a dropdown): the monitor's own
/// size as a borderless full screen at the top, then smaller windows down to 640x480 at the bottom. A
/// click on one asks the window for it; Escape leaves the list as it was. The game waits meanwhile.
public static class ResolutionMenu
{
    private const string HEADING = "Resolution";
    private const byte BACKGROUND_DARKNESS = 200;
    private const int FULL_HEIGHT = 400;
    private const int ENTRY_HEIGHT = 34;
    private const int ENTRY_SPACING = 38;
    private const int HEADING_GAP = 40;
    private const int ENTRY_SEED = 71;
    private const int HEADING_SEED = 61;
    private const string CURRENT_MARK = "> ";

    /// Shows the list until a size is chosen or Escape is pressed.
    public static void Show(Data.Game game, Video video)
    {
        List<WindowControls.Resolution> choices = WindowControls.ResolutionChoices();
        (int Left, int Top, int Width, int Height)[] areas = Areas(choices);
        Image[][] boards = areas.Select((area, index) => Board(area, ENTRY_SEED + (uint)index)).ToArray();

        WaitForRelease(game, video);

        while (true)
        {
            (int pointerX, int pointerY) = InputHandler.GridPointer;
            int? hovered = GameMenu.EntryAt(areas, pointerX, pointerY);

            video.SetOverlay(Draw(choices, areas, boards, hovered));

            if (hovered is { } entry && InputHandler.LeftButtonDown)
            {
                WindowControls.RequestResolution(choices[entry]);

                break;
            }

            if (InputHandler.NextKey() == InputHandler.ESCAPE_KEY)
            {
                break;
            }

            Display.Delay(game, video, 1);
        }

        Leave(game, video);
    }

    // The click that opened the list is waited out, so it does not also choose a size.
    private static void WaitForRelease(Data.Game game, Video video)
    {
        while (InputHandler.LeftButtonDown)
        {
            Display.Delay(game, video, 1);
        }
    }

    // The board of each choice: all as wide as the longest words, in a column in the middle of the
    // screen with room for the heading above.
    private static (int Left, int Top, int Width, int Height)[] Areas(List<WindowControls.Resolution> choices)
    {
        int widest = choices.Max(choice => choice.Text.Length) + CURRENT_MARK.Length;
        int width = widest * Overlay.GLYPH_SIZE * TITLE_SCALE + PANEL_PADDING * 2;
        int firstTop = (FULL_HEIGHT - (choices.Count * ENTRY_SPACING - (ENTRY_SPACING - ENTRY_HEIGHT) + HEADING_GAP)) / 2 + HEADING_GAP;

        return choices.Select((_, index) => ((Overlay.WIDTH - width) / 2, firstTop + index * ENTRY_SPACING, width, ENTRY_HEIGHT)).ToArray();
    }

    // The darkened game, the heading and the choices (the one under the pointer lit, the one in use marked).
    private static Overlay Draw(List<WindowControls.Resolution> choices, (int Left, int Top, int Width, int Height)[] areas, Image[][] boards, int? hovered)
    {
        Overlay overlay = new() { Shades = { new Shade(0, 0, Overlay.WIDTH, FULL_HEIGHT, BACKGROUND_DARKNESS) }, PointerOnTop = true, CenterOnScreen = true };
        Image heading = Board(HeadingArea(areas[0]), HEADING_SEED)[0];

        overlay.Images.Add(heading);
        AddCentered(overlay, HEADING, areas[0].Top - HEADING_GAP / 2 + TITLE_ASCENT / 2, HEADING_COLOR, TITLE_SCALE);

        for (int index = 0; index < choices.Count; index++)
        {
            bool lit = hovered == index;
            string text = (IsCurrent(choices[index]) ? CURRENT_MARK : "") + choices[index].Text;

            overlay.Images.Add(boards[index][lit ? 1 : 0]);
            AddCentered(overlay, text, areas[index].Top + TITLE_ASCENT + PANEL_PADDING - TITLE_SCALE, lit ? HOVER_COLOR : TITLE_COLOR, TITLE_SCALE);
        }

        return overlay;
    }

    // The little board behind the heading, above the first choice.
    private static (int Left, int Top, int Width, int Height) HeadingArea((int Left, int Top, int Width, int Height) first)
    {
        int width = Overlay.TextWidth(HEADING, TITLE_SCALE) + PANEL_PADDING * 2;

        return ((Overlay.WIDTH - width) / 2, first.Top - HEADING_GAP, width, TITLE_ASCENT + PANEL_PADDING * 2);
    }

    // Whether a choice is what the window is now.
    private static bool IsCurrent(WindowControls.Resolution choice) =>
        choice.Text == WindowControls.ResolutionText;

    // Takes the list off the screen, and keeps the click or key that closed it out of the game.
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
