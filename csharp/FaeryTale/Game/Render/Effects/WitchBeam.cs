using FaeryTale.Game.Data;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The witch's spinning beam: a filled quadrilateral drawn in XOR mode in a temporary layer,
/// with the sign of the hero's side of each edge kept for hit tests.
///
/// The beam is the area between two of 64 directions around the witch (Witchpoints lists the near
/// and far end of each). WitchFx draws it with the exclusive-or drawing mode, so drawing it a
/// second time erases it. It also records, in Side1 and Side2, on which side of each edge the hero
/// stands, which the main loop uses to decide whether the beam hits the hero.
public static class WitchBeam
{
    // The beam is clipped to the 19 x 6 tiles of the play field.
    private const int CLIP_WIDTH = 16 * 19;
    private const int CLIP_HEIGHT = 6 * 32;

    // The map position is cut to whole tiles (16 wide, 32 tall).
    private const ushort TILE_X_MASK = 0xfff0;
    private const ushort TILE_Y_MASK = 0xffe0;

    // The beam has 64 directions, 4 bytes (two points) each.
    private const int POINT_BYTES = 4;
    private const int DIRECTIONS_BEFORE = 63;
    private const int DIRECTIONS_AFTER = 1;

    /// An edge is two (x, y) points of one byte each, so 4 bytes.
    private const int EDGE_BYTES = 4;

    /// Which side of the line (p1, p2) the point (xh, yh) is on: the cross product, as the 16 bit
    /// value the original kept.
    internal static short Side((short X, short Y) p1, (short X, short Y) p2, short xh, short yh)
    {
        short dx = unchecked((short)(p2.X - p1.X));
        short dy = unchecked((short)(p2.Y - p1.Y));
        short dx1 = unchecked((short)(xh - p1.X));
        short dy1 = unchecked((short)(yh - p1.Y));

        return unchecked((short)(dx * dy1 - dy * dx1));
    }

    /// Draws the witch's beam on page page and records on which side of its edges the hero is.
    /// The drawing is clipped to the map area.
    public static void WitchFx(Witch witch, World world, Hero hero, Video video, int page)
    {
        FPage fp = video.Screen.Pages[page];
        short xh = unchecked((short)(hero.X - (world.MapX & TILE_X_MASK)));
        short yh = unchecked((short)(hero.Y - (world.MapY & TILE_Y_MASK)));

        ((short, short) p1, (short, short) p2) = Edge(unchecked((short)(fp.WitchDir + DIRECTIONS_BEFORE)), fp);
        ((short, short) p3, (short, short) p4) = Edge(unchecked((short)(fp.WitchDir + DIRECTIONS_AFTER)), fp);

        witch.Side1 = Side(p1, p2, xh, yh);
        witch.Side2 = Side(p3, p4, xh, yh);

        PaintBeam(video, [p1, p2, p4, p3]);
    }

    // The two points of the beam edge for direction dir, relative to the witch.
    private static ((short X, short Y) Near, (short X, short Y) Far) Edge(short dir, FPage fp)
    {
        int start = (byte)(dir * POINT_BYTES);
        sbyte[] points = ObjectTables.Witchpoints;

        return (
            ((short)(points[start] + fp.WitchX), (short)(points[start + 1] + fp.WitchY)),
            ((short)(points[start + 2] + fp.WitchX), (short)(points[start + 3] + fp.WitchY)));
    }

    // Fills the beam's quadrilateral with the corners (in drawing order) in complement mode, so
    // drawing it a second time erases it.
    private static void PaintBeam(Video video, (short X, short Y)[] corners)
    {
        (int, int)[] points = corners.Select(corner => ((int)corner.X, (int)corner.Y)).ToArray();

        video.Screen.SetDrawMode(PortId.Map, Drawing.COMPLEMENT);
        video.Screen.FillPolygon(PortId.Map, points, new Rect(0, 0, CLIP_WIDTH + 1, CLIP_HEIGHT + 1));
    }
}
