using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Startup;

/// Pictures of the page turn at any point of the turn, 640 x 400 pixels from two 320 x 200 story
/// pages. The right page of the old picture lifts off from the book's spine, narrows as it swings
/// up (its far edge dips in at the top and bottom, as in the original's curl) and, once it is past
/// upright, lies down on the left as the left page of the new picture.
internal sealed class PageTurn
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
    private const int CHANNEL_MASK = 0xff;
    private const uint OPAQUE = 0xff00_0000;

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
            double fromSpine = (column + 0.5) * PIXEL_SIZE - SPINE;
            uint[] below = fromSpine >= 0 && InsideBook(column / 2) ? newPage : oldPage;
            Flap? flap = FlapAt(fromSpine, Math.Cos(angle), Math.Sin(angle));

            for (int row = 0; row < OUTPUT_HEIGHT; row++)
            {
                output[row * OUTPUT_WIDTH + column] = PixelAt(below, flap, column / 2, row);
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
    private static uint PixelAt(uint[] below, Flap? flap, int pictureColumn, int row)
    {
        int pictureRow = row / 2;
        double y = (row + 0.5) * PIXEL_SIZE;

        if (flap is null)
        {
            return below[pictureRow * PAGE_WIDTH + pictureColumn];
        }

        if (flap.Covers && y >= flap.Inset && y < PAGE_HEIGHT - flap.Inset)
        {
            return Scale(Average(flap, pictureRow), flap.Shade);
        }

        return Scale(below[pictureRow * PAGE_WIDTH + pictureColumn], 1 - flap.Shadow);
    }

    // The mean colour of a row of a page from one column to another.
    private static uint Average(Flap flap, int row)
    {
        uint red = 0;
        uint green = 0;
        uint blue = 0;

        for (int column = flap.FirstColumn; column <= flap.LastColumn; column++)
        {
            uint color = flap.Source[row * PAGE_WIDTH + column];

            red += (color >> 16) & CHANNEL_MASK;
            green += (color >> 8) & CHANNEL_MASK;
            blue += color & CHANNEL_MASK;
        }

        uint count = (uint)(flap.LastColumn - flap.FirstColumn + 1);

        return OPAQUE | (red / count << 16) | (green / count << 8) | (blue / count);
    }

    // A colour with each channel multiplied by a factor from 0 to 1.
    private static uint Scale(uint color, double factor)
    {
        uint Part(int shift) => (uint)(((color >> shift) & CHANNEL_MASK) * factor) << shift;

        return OPAQUE | Part(16) | Part(8) | Part(0);
    }
}
