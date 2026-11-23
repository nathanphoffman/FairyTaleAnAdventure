using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Video as an output: the classic drawing of what the rules ask for.
public sealed partial class Video : IOutput
{
    // The scrolling message area of the text panel.
    private const int TEXT_X_MIN = 16;
    private const int TEXT_X_MAX = 400;
    private const int TEXT_Y_MIN = 5;
    private const int TEXT_Y_MAX = 44;

    // Pixels the panel scrolls up for each new line, and the baseline of that new line.
    private const int LINE_HEIGHT = 10;
    private const int NEW_LINE_Y = 42;

    // Item names in the message log: the dark brown pen of the text panel's colours, and the underline
    // two lines below the baseline (the letters reach one line below it).
    private const byte ITEM_NAME_PEN = 18;
    private const int UNDERLINE_OFFSET = 2;

    /// The colour the hero's "+" is drawn in on the overview map.
    private const byte MAP_MARK_PEN = 31;

    // The intro zoom is centred on a 320 x 200 screen, and the text panel gets what is left of 95 lines.
    private const int SCREEN_CENTER_X = 160;
    private const int SCREEN_CENTER_Y = 100;
    private const int ZOOM_TEXT_HEIGHT = 95;

    // A story page's text area (top left and bottom right corners), and its colours.
    private static readonly (int X, int Y) PLACARD_TOP_LEFT = (13, 13);
    private static readonly (int X, int Y) PLACARD_BOTTOM_RIGHT = (271, 107);
    private const byte PEN_BLACK = 0;
    private const byte PLACARD_TEXT_PEN = 24;

    // Where the menu entries are drawn in the text panel: two columns, a row every 9 pixels.
    private const int MENU_LEFT_COLUMN_X = 430;
    private const int MENU_RIGHT_COLUMN_X = 482;
    private const int MENU_FIRST_ROW_Y = 8;
    private const int MENU_ROW_HEIGHT = 9;

    /// The entry's text is inset from the edge of its coloured box by this much.
    private const int MENU_TEXT_INSET = 4;

    /// Each label is 5 characters; the box behind it is 6 wide.
    private static readonly byte[] MENU_BOX = "      "u8.ToArray();

    /// Characters in a formatted number (the last one is the trailing space).
    private const int NUMBER_FIELD = 10;

    public uint Ticks => Device.Ticks;

    /// The lines the message window has shown, for scrolling back (kept in Remastered and Reimagined).
    public MessageHistory History { get; } = new();

    // The message window is showing lines this far back (0 is the live window).
    private int shownScroll;

    public void LoadPageColors(ReadOnlySpan<ushort> colors) => Screen.LoadColors(ViewPortId.Page, colors);

    public void SwapPages()
    {
        int finished = Screen.Drawing;

        (Screen.Drawing, Screen.Viewing) = (Screen.Viewing, Screen.Drawing);
        Screen.VpPage.Ras = Screen.Pages[finished].Ras;
    }

    public void Present()
    {
        Screen.PointerPosition = Platform.InputBridge.Pointer;
        Device.Present(Screen.ToScene());
    }

    public void WaitFrame()
    {
        Screen.PointerPosition = Platform.InputBridge.Pointer;
        Device.WaitFrame(Screen.ToScene());
    }

    public void SetOverlay(Overlay overlay) => Device.SetOverlay(overlay);

    public double AnimationFrameMs => Device.AnimationFrameMs;

    public void ShowAnimationFrame()
    {
        Screen.PointerPosition = Platform.InputBridge.Pointer;
        Device.ShowAnimationFrame(Screen.ToScene());
    }
}
