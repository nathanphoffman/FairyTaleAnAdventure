using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;
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

    /// A line of text in the message area; in Remastered and Reimagined a line that is too wide wraps
    /// onto more lines by itself.
    public void PrintBytes(ReadOnlySpan<byte> text)
    {
        if (!Capture.Enabled)
        {
            PrintLine(text);

            return;
        }

        foreach (byte[] line in MessageWrapper.Wrap(text, TEXT_X_MAX - TEXT_X_MIN + 1, character => Screen.AdvanceOf(Screen.CurrentPort, character)))
        {
            PrintLine(line);
        }
    }

    // The old lines scroll up by one text line (10 pixels) inside the message area.
    private void PrintLine(ReadOnlySpan<byte> text)
    {
        ShowNewestMessages();

        Rect area = new(TEXT_X_MIN, TEXT_Y_MIN, TEXT_X_MAX - TEXT_X_MIN + 1, TEXT_Y_MAX - TEXT_Y_MIN + 1);

        Screen.Scroll(Screen.CurrentPort, area, (0, LINE_HEIGHT));
        TextRoutines.MoveTo(this, TEXT_X_MIN, NEW_LINE_Y);
        DrawMessageText(Screen.CurrentPort, text);

        if (Capture.Enabled)
        {
            History.StartLine(text);
        }
    }

    public void PrintContBytes(ReadOnlySpan<byte> text)
    {
        ShowNewestMessages();

        if (History.OnMessageLine)
        {
            DrawMessageText(Screen.CurrentPort, text);
        }
        else
        {
            Screen.Text(Screen.CurrentPort, text);
        }

        if (Capture.Enabled)
        {
            History.Append(text);
        }
    }

    public void PrintNumber(int value, int length)
    {
        byte[] text = TextRoutines.FormatNumber((uint)value);
        int from = Math.Max(NUMBER_FIELD - length, 0);

        ShowNewestMessages();
        Screen.Text(Screen.CurrentPort, text.AsSpan(from));

        if (Capture.Enabled)
        {
            History.Append(text.AsSpan(from));
        }
    }

    /// Scrolls the message window the way the player asked (mouse wheel, Page Up and Page Down).
    /// Called every frame; the classic game never asks.
    public void ShowAskedMessageScroll()
    {
        if (!Capture.Enabled)
        {
            return;
        }

        History.ScrollBy(MessageScrollRequests.Take());

        if (MessageScrollBar.DragTarget(History, InputHandler.GridPointer, InputHandler.LeftButtonDown) is { } dragged)
        {
            History.ScrollBy(dragged - History.Scrolled);
        }

        if (shownScroll != History.Scrolled)
        {
            DrawMessageWindow();
        }
    }

    // Before anything new is printed the window goes back to the newest lines.
    private void ShowNewestMessages()
    {
        if (!Capture.Enabled)
        {
            return;
        }

        History.ScrollToNewest();

        if (shownScroll != 0)
        {
            DrawMessageWindow();
        }
    }

    // Draws the lines of the history the window shows now, the newest at the bottom, and leaves the
    // text position at the end of the bottom line.
    private void DrawMessageWindow()
    {
        Rect area = new(TEXT_X_MIN, TEXT_Y_MIN, TEXT_X_MAX - TEXT_X_MIN + 1, TEXT_Y_MAX - TEXT_Y_MIN + 1);
        IReadOnlyList<byte[]> window = History.Window();

        shownScroll = History.Scrolled;
        Screen.Scroll(PortId.Text, area, (0, area.Height));

        for (int index = 0; index < window.Count; index++)
        {
            int rowsFromBottom = window.Count - 1 - index;

            Screen.MoveTo(PortId.Text, TEXT_X_MIN, NEW_LINE_Y - rowsFromBottom * LINE_HEIGHT);
            DrawMessageText(PortId.Text, window[index]);
        }
    }

    /// How far across a character of the message font reaches.
    public int MessageAdvance(byte character) => Screen.AdvanceOf(PortId.Text, character);

    // Writes message text at the pen. In Remastered and Reimagined the names of items are written
    // in dark brown and underlined, so they stand out as things the pointer can ask about.
    private void DrawMessageText(PortId port, ReadOnlySpan<byte> text)
    {
        if (!Capture.Enabled || Capture.Game is not { } game)
        {
            Screen.Text(port, text);

            return;
        }

        int done = 0;

        foreach (ItemWord word in ItemWords.Find(game, text))
        {
            Screen.Text(port, text[done..word.Start]);
            DrawItemName(port, text.Slice(word.Start, word.Length));
            done = word.Start + word.Length;
        }

        Screen.Text(port, text[done..]);
    }

    // One item name: dark brown with a line under it, then the pen goes back as it was.
    private void DrawItemName(PortId port, ReadOnlySpan<byte> name)
    {
        Port current = Screen.GetPort(port);
        (byte color, int left, int baseline) = (current.Pen.Fg, current.Pen.X, current.Pen.Y);

        Screen.SetAPen(port, ITEM_NAME_PEN);
        Screen.Text(port, name);
        Screen.RectFill(port, (left, baseline + UNDERLINE_OFFSET), (current.Pen.X - 1, baseline + UNDERLINE_OFFSET));
        Screen.SetAPen(port, color);
    }

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

    public void ForgetSprites() => Graphics.NextShape = 0;

    public void RewindSpritesTo(byte slot) => Graphics.NextShape = Graphics.SeqList[slot].Location;

    public void StartSpriteSheet(byte slot) => Graphics.SeqList[slot].Location = Graphics.NextShape;

    public void LoadSpriteFile(int file) => SpriteSheets.LoadSpriteFile(this, file);

    public void FinishSpriteLoad(short slot) => SpriteSheets.FinishSpriteLoad(this, slot);

    public void LoadImageBank(int bank, short firstBlock) => Graphics.LoadImageBank(bank, firstBlock);

    public void SetTextPosition(int x, int y)
    {
        History.OnMessageLine = y == NEW_LINE_Y;
        Screen.MoveTo(Screen.CurrentPort, x, y);
    }

    public void UseStoryFont() => Screen.SetFontKind(Screen.CurrentPort, FontKind.Story);

    public void UsePanelFont() => Screen.SetFontKind(Screen.CurrentPort, FontKind.Panel);

    public void SelectMessageArea() => Screen.CurrentPort = PortId.Text;

    public void ClearPlacard()
    {
        PortId port = Screen.CurrentPort;

        Screen.SetAPen(port, PEN_BLACK);
        Screen.RectFill(port, PLACARD_TOP_LEFT, PLACARD_BOTTOM_RIGHT);
        Screen.SetAPen(port, PLACARD_TEXT_PEN);
    }

    public void DrawPlacardBorder() => PlacardBorder.DrawPlacardBorder(Screen);

    public void DrawMenuEntry(int slot, ReadOnlySpan<byte> label, bool selected, byte boxPen)
    {
        (int x, int y) = MenuSlotPosition(slot);

        Screen.SetAPen(PortId.Text2, selected ? (byte)1 : (byte)0);
        Screen.SetBPen(PortId.Text2, boxPen);
        Screen.MoveTo(PortId.Text2, x, y);
        Screen.Text(PortId.Text2, MENU_BOX);
        Screen.MoveTo(PortId.Text2, x + MENU_TEXT_INSET, y);
        Screen.Text(PortId.Text2, label);
    }

    public void BlankMenuEntry(int slot)
    {
        (int x, int y) = MenuSlotPosition(slot);

        Screen.SetBPen(PortId.Text2, PEN_BLACK);
        Screen.MoveTo(PortId.Text2, x, y);
        Screen.Text(PortId.Text2, MENU_BOX);
    }

    public void DrawCompass(short dir) => ScreenEffects.DrawCompass(Screen, dir);

    public void DrawItemList(Hero hero)
    {
        Capture.WorldVisible = false;
        ScreenEffects.DrawItemList(this, hero);
    }

    public void DrawOverviewMap(MapData map, World world, int mx, int my)
    {
        Capture.WorldVisible = false;
        Screen.DrawTarget = Screen.DrawingBitmapId;
        OverviewMap.DrawOverviewMap(this, map, world, mx, my);
    }

    public void MarkOverviewMap((int X, int Y)? at)
    {
        Screen.GetPort(PortId.Map).Target = Screen.DrawTarget;
        Screen.SetDrawMode(PortId.Map, Drawing.JAM1);
        Screen.SetAPen(PortId.Map, MAP_MARK_PEN);

        if (at is { } place)
        {
            Screen.MoveTo(PortId.Map, place.X, place.Y);
            Screen.Text(PortId.Map, "+"u8);
        }
    }

    public void ClearDrawingPage()
    {
        Capture.WorldVisible = false;
        Screen.GetPort(PortId.Map).Target = Screen.DrawingBitmapId;
        Screen.Clear(PortId.Map, PEN_BLACK);
        Screen.GetPort(PortId.Map).Target = Screen.ViewingBitmapId;
    }

    public void ResetPageScroll()
    {
        int ras = Screen.DrawingPage.Ras;

        Screen.Ras[ras].X = 0;
        Screen.Ras[ras].Y = 0;
    }

    public void PrepareMessagePage()
    {
        Capture.WorldVisible = false;
        Screen.CurrentPort = PortId.Map;
        Screen.GetPort(PortId.Map).Target = Screen.DrawingBitmapId;
        Screen.SetDrawMode(PortId.Map, Drawing.JAM1);
        Screen.SetAPen(PortId.Map, PLACARD_TEXT_PEN);
        Screen.Clear(PortId.Map, 0);
    }

    public void SetPanelVisible(bool visible) =>
        Screen.VpText.Modes = visible ? (ushort)(Screen.HIRES | Screen.SPRITES) : (ushort)(Screen.HIRES | Screen.SPRITES | Screen.VP_HIDE);

    public void SetZoomSize(int x, int y)
    {
        short left = (short)(SCREEN_CENTER_X - x);
        short top = (short)(SCREEN_CENTER_Y - y);

        for (int ras = 0; ras < 2; ras++)
        {
            Screen.Ras[ras].X = left;
            Screen.Ras[ras].Y = top;
        }

        Screen.VpPage.X = left;
        Screen.VpPage.Width = (short)(x + x);
        Screen.VpPage.Y = top;
        Screen.VpPage.Height = (short)(y + y);

        Screen.VpText.X = 0;
        Screen.VpText.Y = 0;
        Screen.VpText.Height = (short)(ZOOM_TEXT_HEIGHT - y);
    }

    public void LoadPictureToDrawingPage(string name)
    {
        BitmapId id = Screen.DrawingBitmapId;

        Screen.DrawTarget = id;
        IffPicture.LoadPicture(name, Screen.GetBitmap(id), 0, 0);
    }

    public void LoadPanelColors(ReadOnlySpan<ushort> colors) => Screen.LoadColors(ViewPortId.Text, colors);

    // The top left corner of menu entry slot: even slots on the left, odd on the right.
    private static (int X, int Y) MenuSlotPosition(int slot) =>
        ((slot & 1) != 0 ? MENU_RIGHT_COLUMN_X : MENU_LEFT_COLUMN_X, slot / 2 * MENU_ROW_HEIGHT + MENU_FIRST_ROW_Y);
}
