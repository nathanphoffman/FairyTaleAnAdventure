using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The screen: the bitmaps the game draws on, the pens that draw on them, the two view ports that
/// make up the picture, and the two display pages that are drawn and shown in turn.
///
/// The play field is a lores picture of 5 bit planes (32 colours) on top; the text panel is a
/// hires picture of 4 bit planes below it. The game draws on one page while the other is on
/// screen, then swaps them.
public sealed partial class Screen
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

    // The bitmaps by position: the page A, page B and the last one (the rest of the ids) come after
    // the first three (pages 1 and 2 and the text bitmap).
    private const int BITMAP_COUNT = 6;
    private const int PAGE_A_INDEX = 3;
    private const int PAGE_B_INDEX = 4;
    private const int LAST_BITMAP_INDEX = 5;

    /// How many raster infos the screen keeps.
    private const int RAS_INFO_COUNT = 3;

    private readonly Bitmap[] bitmaps = Enumerable.Range(0, BITMAP_COUNT).Select(_ => Bitmap.Empty()).ToArray();
    private readonly Port[] ports = [new(), new(), new()];
    private readonly Font[] fonts = [Font.Builtin, Font.Builtin];

    /// The compass picture: lit up, and not.
    public Bitmap[] Compass = [Bitmap.Empty(), Bitmap.Empty()];

    public RasInfo[] Ras = new RasInfo[RAS_INFO_COUNT];
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
    public static int PlanesOf(BitmapId identifier) => identifier == BitmapId.TextPlane0 ? 1 : int.MaxValue;

    /// Makes the bitmaps, pens and view ports, and the memory the sprite blitter works in.
    public void SetUp(byte[][] compass)
    {
        MakeBitmaps(compass);
        MakeRastersWithPorts();
        MakeViewPorts();

        QueueRegion = new byte[BLITTER_REGION];
        MaskRegion = new byte[BLITTER_REGION];
    }

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
