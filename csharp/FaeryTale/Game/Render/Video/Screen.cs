using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The bitmaps the game keeps.
public enum BitmapId
{
    /// The two display pages.
    Page1,
    Page2,

    /// The text panel, all 4 planes.
    Text,

    /// The text panel, but only its first plane (the message area is drawn in one colour).
    TextPlane0,

    /// The two pages of the story book, which are also work areas for the picture loader.
    PageA,
    PageB,

    /// The pictures of the things the hero carries, one long strip, for the item list.
    Items,
}

/// The three places text and graphics are drawn from.
public enum PortId
{
    /// The play field.
    Map,

    /// The message area of the text panel.
    Text,

    /// The menu area of the text panel.
    Text2,
}

public enum FontKind
{
    /// The small font of the panel.
    Panel,

    /// The story font.
    Story,
}

public enum ViewPortId
{
    /// The play field.
    Page,

    /// The text panel.
    Text,
}

/// A pen, the bitmap it draws on and the font it writes in.
public sealed class Port
{
    public Pen Pen = new();
    public BitmapId? Target;
    public FontKind Font = FontKind.Panel;
}

/// Which part of which bitmap a view port shows.
public struct RasInfo
{
    public BitmapId Bitmap;
    public short X;
    public short Y;
}

/// A rectangle of the screen that shows a bitmap with its own colours.
public sealed class ViewPortState
{
    /// The index into Screen.Ras of what it shows, once it shows anything.
    public int? Ras;
    public short Width;
    public short Height;
    public short X;
    public short Y;
    public ushort Modes;
    public ushort[] Colors = [];
}

/// One of the two display pages being drawn and shown (struct fpage). The game draws on one while
/// the other is on screen, then swaps them. It also remembers what is drawn there so it can be
/// erased later, and the witch's beam position.
public sealed class FPage
{
    /// Which of the screen's Ras entries (the bitmap and its scroll offset) this page shows.
    public int Ras;
    public int IsvX;
    public int IsvY;
    public short ObCount;

    /// How each sprite drawn on this page was drawn, to undo them.
    public List<SShape> ShapeQueue = [];

    /// Where this page's saved backgrounds start.
    public Loc BackSave = Loc.NONE;
    public int SaveUsed;
    public short WitchX;
    public short WitchY;
    public short WitchDir;
    public short WFlag;
}

/// The screen: the bitmaps the game draws on, the pens that draw on them, the two view ports that
/// make up the picture, and the two display pages that are drawn and shown in turn.
///
/// The play field is a lores picture of 5 bit planes (32 colours) on top; the text panel is a
/// hires picture of 4 bit planes below it. The game draws on one page while the other is on
/// screen, then swaps them.
public sealed class Screen
{
    /// View port mode bits.
    public const ushort HIRES = 0x8000;
    public const ushort SPRITES = 0x4000;
    public const ushort VP_HIDE = 0x2000;

    // Sizes of the bitmaps. The play field is 320 wide with room to scroll, 5 planes deep; the
    // text panel is hires, 4 planes deep.
    private const int PAGE_WIDTH = 320;
    private const int PAGE_ROWS = 200;
    private const int PAGE_DEPTH = 5;
    private const int TEXT_WIDTH = 640;
    private const int TEXT_ROWS = 57;
    private const int TEXT_DEPTH = 4;
    private const int COMPASS_WIDTH = 64;
    private const int COMPASS_ROWS = 24;

    // Where the two view ports sit on the screen.
    private const short LOWRES_WIDTH = 288;
    private const short HIRES_WIDTH = 640;
    private const short PAGE_VIEW_HEIGHT = 140;
    private const short PAGE_VIEW_X = 16;
    private const short PAGE_HEIGHT = 143;
    private const int PAGE_COLORS = 32;
    private const int TEXT_COLORS = 20;

    /// Bytes of work memory for the sprite blitter's queue (and, the same again, its mask).
    private const int BLITTER_REGION = 6880 + 256;

    // A colour is 12 bits: 4 bits each of red, green and blue.
    private const uint COMPONENT_MASK = 15;
    private const int RED_SHIFT = 8;
    private const int GREEN_SHIFT = 4;

    private readonly Bitmap[] bitmaps = Enumerable.Range(0, 6).Select(_ => Bitmap.Empty()).ToArray();
    private readonly Port[] ports = [new(), new(), new()];
    private readonly Font[] fonts = [Font.Builtin, Font.Builtin];

    /// The compass picture: lit up, and not.
    public Bitmap[] Compass = [Bitmap.Empty(), Bitmap.Empty()];

    public RasInfo[] Ras = new RasInfo[3];
    public FPage[] Pages = [new(), new()];

    /// Which of the two pages is being drawn on and which is on screen.
    public int Drawing;
    public int Viewing = 1;

    public PortId CurrentPort = PortId.Text;
    public ViewPortState VpPage = new();
    public ViewPortState VpText = new();

    /// The bitmap the map and the sprites are being drawn on.
    public BitmapId DrawTarget = BitmapId.Page1;

    /// The mouse pointer's picture, once it is set up, and where the pointer is (lores pixels).
    public uint[]? PointerImage;
    public (int X, int Y) PointerPosition;

    /// The sprite blitter's work memory: saved backgrounds, and the scenery mask.
    public byte[] QueueRegion = [];
    public byte[] MaskRegion = [];
    public short PlaneSize;
    public sbyte Flasher;

    /// How many planes drawing on this bitmap touches.
    public static int PlanesOf(BitmapId id) => id == BitmapId.TextPlane0 ? 1 : int.MaxValue;

    /// Makes the bitmaps, pens and view ports, and the memory the sprite blitter works in.
    public void SetUp(byte[][] compass)
    {
        MakeBitmaps(compass);
        MakeRastersWithPorts();
        MakeViewPorts();

        QueueRegion = new byte[BLITTER_REGION];
        MaskRegion = new byte[BLITTER_REGION];
    }

    public Bitmap GetBitmap(BitmapId id) => bitmaps[IndexOf(id)];

    public void SetBitmap(BitmapId id, Bitmap bitmap) => bitmaps[IndexOf(id)] = bitmap;

    /// What the sprite blitter works on: the bitmap being drawn on and its two work regions.
    public (Bitmap Bitmap, byte[] Queue, byte[] Mask) BlitterMemory() => (bitmaps[IndexOf(DrawTarget)], QueueRegion, MaskRegion);

    public Bitmap DrawBitmap => GetBitmap(DrawTarget);

    /// The bitmap of display page n (0 or 1) as it is wired up now.
    public BitmapId PageBitmap(int n) => Ras[Pages[n].Ras].Bitmap;

    /// The page being drawn on.
    public FPage DrawingPage => Pages[Drawing];

    /// The bitmap of the page being drawn on / being shown.
    public BitmapId DrawingBitmapId => PageBitmap(Drawing);

    public BitmapId ViewingBitmapId => PageBitmap(Viewing);

    public Port GetPort(PortId id) => ports[(int)id];

    public void SetFontKind(PortId id, FontKind kind) => GetPort(id).Font = kind;

    public void SetFont(FontKind kind, Font font) => fonts[(int)kind] = font;

    public void SetAPen(PortId id, byte pen) => GetPort(id).Pen.Fg = pen;

    public void SetBPen(PortId id, byte pen) => GetPort(id).Pen.Bg = pen;

    public void SetDrawMode(PortId id, byte mode) => GetPort(id).Pen.Mode = mode;

    public void MoveTo(PortId id, int x, int y)
    {
        Port port = GetPort(id);

        port.Pen.X = x;
        port.Pen.Y = y;
    }

    /// Writes the bytes at the pen of the port.
    public void Text(PortId id, ReadOnlySpan<byte> bytes)
    {
        if (TargetOf(id) is { } target)
        {
            Port port = GetPort(id);

            WriteText(target.Bitmap, port, bytes, target.Planes);
        }
    }

    /// How far the pen of a port moves across after a character, in the port's font.
    public int AdvanceOf(PortId id, byte character) => fonts[(int)GetPort(id).Font].AdvanceOf(character);

    /// Draws a line from the port's pen to (x, y).
    public void LineTo(PortId id, int x, int y)
    {
        if (TargetOf(id) is { } target)
        {
            Gfx.Drawing.LineTo(target.Bitmap, ref GetPort(id).Pen, (x, y), target.Planes);
        }
    }

    public void RectFill(PortId id, (int X, int Y) from, (int X, int Y) to)
    {
        if (TargetOf(id) is { } target)
        {
            Gfx.Drawing.RectFill(target.Bitmap, GetPort(id).Pen, from, to, target.Planes);
        }
    }

    public void FillPolygon(PortId id, IReadOnlyList<(int X, int Y)> points, Rect clip)
    {
        if (TargetOf(id) is { } target)
        {
            Gfx.Drawing.FillPolygon(target.Bitmap, GetPort(id).Pen, points, clip, target.Planes);
        }
    }

    /// Fills the whole bitmap of the port with colour pen.
    public void Clear(PortId id, byte pen)
    {
        if (TargetOf(id) is { } target)
        {
            target.Bitmap.Clear(pen, target.Planes);
        }
    }

    /// Scrolls an area of the port's bitmap; what scrolls in gets the port's background colour.
    public void Scroll(PortId id, Rect area, (int X, int Y) by)
    {
        if (TargetOf(id) is { } target)
        {
            BitmapOps.Scroll(target.Bitmap, area, by, GetPort(id).Pen.Bg, target.Planes);
        }
    }

    /// Copies a rectangle of one bitmap to another (not the same one), for the planes in mask.
    public void Copy(BitmapId from, Rect rect, BitmapId to, (int X, int Y) at, uint mask)
    {
        (Bitmap source, Bitmap destination) = TwoBitmaps(from, to);

        BitmapOps.CopyRect(source, destination, rect, at, mask);
    }

    /// The compass picture which (0 normal, 1 lit) copied into plane plane of the text panel.
    public void CopyCompass(int which, Rect rect, (int X, int Y) at, int plane) =>
        BitmapOps.CopyPlaneRect(Compass[which], 0, GetBitmap(BitmapId.Text), plane, rect, at);

    /// Two different bitmaps, the first to read and the second to write.
    public (Bitmap From, Bitmap To) TwoBitmaps(BitmapId from, BitmapId to)
    {
        if (IndexOf(from) == IndexOf(to))
        {
            throw new InvalidOperationException("a copy needs two different bitmaps");
        }

        return (GetBitmap(from), GetBitmap(to));
    }

    public ViewPortState GetViewPort(ViewPortId id) => id == ViewPortId.Page ? VpPage : VpText;

    /// Loads the first colours of a view port's table from colors (12 bit colours, 0x0RGB).
    public void LoadColors(ViewPortId id, ReadOnlySpan<ushort> colors)
    {
        ushort[] table = GetViewPort(id).Colors;

        colors[..Math.Min(table.Length, colors.Length)].CopyTo(table);
    }

    /// Sets one colour of a view port from its red, green and blue (4 bits each).
    public void SetColor(ViewPortId id, int n, (uint R, uint G, uint B) color)
    {
        ushort[] table = GetViewPort(id).Colors;

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

    // Writes text with the port's pen and font on the bitmap.
    private void WriteText(Bitmap bitmap, Port port, ReadOnlySpan<byte> bytes, int planes) =>
        Gfx.Drawing.Text(bitmap, ref port.Pen, fonts[(int)port.Font], bytes, planes);

    // Splits a port from the bitmap it draws on and the planes drawing touches.
    private (Bitmap Bitmap, int Planes)? TargetOf(PortId id) =>
        GetPort(id).Target is { } target ? (bitmaps[IndexOf(target)], PlanesOf(target)) : null;

    private ViewPort MakeViewPort(ViewPortState state, int rasIndex)
    {
        RasInfo ras = Ras[rasIndex];

        return new ViewPort
        {
            Bitmap = GetBitmap(ras.Bitmap),
            Scroll = (ras.X, ras.Y),
            Position = (state.X, state.Y),
            Size = (state.Width, state.Height),
            Hires = (state.Modes & HIRES) != 0,
            Colors = state.Colors,
            ShowsPointer = (state.Modes & SPRITES) != 0,
        };
    }

    // The position of an id in the list of bitmaps.
    private static int IndexOf(BitmapId id) => id switch
    {
        BitmapId.Page1 => 0,
        BitmapId.Page2 => 1,
        BitmapId.Text or BitmapId.TextPlane0 => 2,
        BitmapId.PageA => 3,
        BitmapId.PageB => 4,
        _ => 5,
    };

    // The pages, the text panel and the compass pictures.
    private void MakeBitmaps(byte[][] compass)
    {
        SetBitmap(BitmapId.Page1, new Bitmap(PAGE_WIDTH, PAGE_ROWS, PAGE_DEPTH));
        SetBitmap(BitmapId.Page2, new Bitmap(PAGE_WIDTH, PAGE_ROWS, PAGE_DEPTH));
        SetBitmap(BitmapId.Text, new Bitmap(TEXT_WIDTH, TEXT_ROWS, TEXT_DEPTH));
        SetBitmap(BitmapId.PageA, new Bitmap(PAGE_WIDTH, PAGE_ROWS, PAGE_DEPTH));
        SetBitmap(BitmapId.PageB, new Bitmap(PAGE_WIDTH, PAGE_ROWS, PAGE_DEPTH));
        Compass = compass.Select(data => Gfx.Bitmap.FromPlane(COMPASS_WIDTH, COMPASS_ROWS, data)).ToArray();
    }

    // Which bitmap each raster shows, which page is drawn and shown, and where each port draws.
    private void MakeRastersWithPorts()
    {
        Ras = [new RasInfo { Bitmap = BitmapId.Page1 }, new RasInfo { Bitmap = BitmapId.Page2 }, new RasInfo { Bitmap = BitmapId.Text }];

        // Both pages start on the first bitmap; the intro gives the second page its own.
        Pages[0].Ras = 0;
        Pages[1].Ras = 0;
        Drawing = 0;
        Viewing = 1;

        GetPort(PortId.Map).Target = null;
        GetPort(PortId.Text).Target = BitmapId.Text;
        GetPort(PortId.Text2).Target = BitmapId.Text;
        CurrentPort = PortId.Text;
    }

    // The play field on top and the text panel below it.
    private void MakeViewPorts()
    {
        VpPage = new ViewPortState
        {
            Width = LOWRES_WIDTH,
            Height = PAGE_VIEW_HEIGHT,
            X = PAGE_VIEW_X,
            Colors = new ushort[PAGE_COLORS],
        };

        VpText = new ViewPortState
        {
            Ras = 2,
            Width = HIRES_WIDTH,
            Height = TEXT_ROWS,
            Y = PAGE_HEIGHT,
            Modes = HIRES | SPRITES | VP_HIDE,
            Colors = new ushort[TEXT_COLORS],
        };
    }
}
