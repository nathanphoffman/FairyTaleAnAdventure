using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Startup;

/// Pictures of the page turn at any point of the turn, 640 x 400 pixels from two 320 x 200 story
/// pages. The right page of the old picture lifts off from the book's spine, narrows as it swings
/// up (its far edge dips in at the top and bottom, as in the original's curl) and, once it is past
/// upright, lies down on the left as the left page of the new picture.
internal sealed partial class PageTurn
{
    public const int PAGE_WIDTH = 320;
    public const int PAGE_HEIGHT = 200;
    public const int OUTPUT_WIDTH = PAGE_WIDTH * 2;
    public const int OUTPUT_HEIGHT = PAGE_HEIGHT * 2;

    /// The book's spine and how far the turning page reaches from it (picture pixels).
    private const double SPINE = 160;
    private const int PAGE_REACH = 135;

    /// The book's outer edges: the pages of the original turn did not reach past them.
    public const int BOOK_LEFT_EDGE = 24;
    public const int BOOK_RIGHT_EDGE = 296;

    /// A page seen edge on is not drawn.
    private const double MIN_COSINE = 0.03;

    // The turning page darkens towards its far edge, and throws a soft shadow on what is below.
    private const double SHADE_AT_EDGE = 0.25;
    private const double SHADOW_REACH = 14;
    private const double SHADOW_STRENGTH = 0.4;

    private const double PIXEL_SIZE = 0.5; // an output pixel is half a picture pixel

    // Sampling at the middle of a pixel, not its corner, is half a pixel in.
    private const double PIXEL_MIDDLE = 0.5;
    private const int CHANNEL_MASK = 0xff;
    private const uint OPAQUE = 0xff00_0000;

    // Where each colour byte sits in a 0xAARRGGBB pixel.
    private const int RED_SHIFT = 16;
    private const int GREEN_SHIFT = 8;

    private readonly uint[] oldPage;
    private readonly uint[] newPage;

    public PageTurn(uint[] oldPage, uint[] newPage)
    {
        this.oldPage = oldPage;
        this.newPage = newPage;
    }

    // What the turning page is like over one output column: the picture columns that squeeze into
    // it, how far along the page it is, and the darkening of the page and of the shadow it throws.
    private sealed record Flap(uint[] Source, int FirstColumn, int LastColumn, double Inset, double Shade, double Shadow, bool Covers);

    /// The picture at progress 0 (the old page) to 1 (the new page).
    public uint[] Render(double progress)
    {
        uint[] output = new uint[OUTPUT_WIDTH * OUTPUT_HEIGHT];
        double angle = progress * Math.PI;

        for (int column = 0; column < OUTPUT_WIDTH; column++)
        {
            double fromSpine = (column + PIXEL_MIDDLE) * PIXEL_SIZE - SPINE;
            uint[] below = fromSpine >= 0 && InsideBook(column / 2) ? newPage : oldPage;
            Flap? flap = FlapAt(fromSpine, Math.Cos(angle), Math.Sin(angle));

            for (int rowIndex = 0; rowIndex < OUTPUT_HEIGHT; rowIndex++)
            {
                output[rowIndex * OUTPUT_WIDTH + column] = PixelAt(below, flap, column / 2, rowIndex);
            }
        }

        return output;
    }

    // The turning page over a column at fromSpine pixels from the spine, if it covers or shades it.
    private Flap? FlapAt(double fromSpine, double cosine, double sine)
    {
        double narrowing = Math.Abs(cosine);

        // On the right of the spine while it rises, on the left once it lies down.
        if (narrowing < MIN_COSINE || (cosine > 0) != (fromSpine >= 0))
        {
            return null;
        }

        double beyondEdge = Math.Abs(fromSpine) - PAGE_REACH * narrowing;

        if (beyondEdge >= SHADOW_REACH)
        {
            return null;
        }

        // How far along the page the column is, and how many page pixels fit into it.
        double distance = Math.Abs(fromSpine) / narrowing;
        double halfSpan = PIXEL_SIZE / narrowing / 2;
        double near = Math.Clamp(distance - halfSpan, 0, PAGE_REACH);
        double far = Math.Clamp(distance + halfSpan, 0, PAGE_REACH);
        double shadow = beyondEdge > 0 ? SHADOW_STRENGTH * sine * (1 - beyondEdge / SHADOW_REACH) : 0;
        double shade = 1 - SHADE_AT_EDGE * sine * Math.Min(distance / PAGE_REACH, 1);
        double inset = MapCamera.PageTurnInset((int)near) * sine;
        int sign = cosine > 0 ? 1 : -1;
        int first = Math.Clamp((int)(SPINE + sign * Math.Min(near, far)), 0, PAGE_WIDTH - 1);
        int last = Math.Clamp((int)(SPINE + sign * Math.Max(near, far)), 0, PAGE_WIDTH - 1);

        return new Flap(cosine > 0 ? oldPage : newPage, Math.Min(first, last), Math.Max(first, last), inset, shade, shadow, beyondEdge <= 0);
    }

    // The book's pages run between its edges; outside them the picture stays as it was.
    private static bool InsideBook(int pictureColumn) => pictureColumn >= BOOK_LEFT_EDGE && pictureColumn < BOOK_RIGHT_EDGE;

    // The colour of one output pixel: the turning page (averaged over the picture columns that
    // squeeze into it) where it covers the pixel, else the page below with the shadow on it.
    private static uint PixelAt(uint[] below, Flap? flap, int pictureColumn, int rowIndex)
    {
        int pictureRow = rowIndex / 2;
        double positionY = (rowIndex + PIXEL_MIDDLE) * PIXEL_SIZE;

        if (flap is null)
        {
            return below[pictureRow * PAGE_WIDTH + pictureColumn];
        }

        if (flap.Covers && positionY >= flap.Inset && positionY < PAGE_HEIGHT - flap.Inset)
        {
            return Scale(Average(flap, pictureRow), flap.Shade);
        }

        return Scale(below[pictureRow * PAGE_WIDTH + pictureColumn], 1 - flap.Shadow);
    }
}
