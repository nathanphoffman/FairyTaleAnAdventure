using FaeryTale.Game.Creatures.Interaction;

namespace FaeryTale.Game.Render;

/// Finishing the scrolled map edge each frame, and the witch's beam and her damage.
public static class MapEdge
{
    // The map strip to redraw after a scroll: the right edge (36) or the left (0), and the bottom
    // row (10) or the top (0).
    private const int STRIP_RIGHT = 36;
    private const int STRIP_LEFT = 0;
    private const int ROW_BOTTOM = 10;
    private const int ROW_TOP = 0;

    /// The witch's beam hurts the hero within this distance.
    private const int WITCH_REACH = 100;

    // The witch's position is kept relative to the scrolled map: its x snaps to the 16 pixel tile
    // grid and its y (15 pixels above her feet) to the 32 pixel one.
    private const ushort TILE_X_MASK = 0xfff0;
    private const ushort TILE_Y_MASK = 0xffe0;
    private const int WITCH_ANCHOR_Y = 15;
    private const int WITCH_SLOT = 2;

    /// Finishes the scrolled map edge (difX, difY = scroll step) and runs the witch's beam and her damage.
    public static void RepairEdges(Data.Game game, Video video, short difX, short difY)
    {
        if (difX == 1)
        {
            MapTiles.DrawMapColumn(game, video, STRIP_RIGHT);
        }
        else if (difX == -1)
        {
            MapTiles.DrawMapColumn(game, video, STRIP_LEFT);
        }

        if (difY == 1)
        {
            MapTiles.DrawMapRow(game, video, ROW_BOTTOM);
        }
        else if (difY == -1)
        {
            MapTiles.DrawMapRow(game, video, ROW_TOP);
        }

        Data.Shape witch = game.Actors.List[WITCH_SLOT];
        FPage page = video.Screen.DrawingPage;

        page.WitchX = unchecked((short)(witch.AbsX - (game.World.MapX & TILE_X_MASK)));
        page.WitchY = unchecked((short)(witch.AbsY - WITCH_ANCHOR_Y - (game.World.MapY & TILE_Y_MASK)));

        if (game.Rng.Rand4() == 0)
        {
            game.Witch.Direction = (sbyte)(game.Witch.Side1 > 0 ? 1 : -1);
        }

        game.Witch.Angle = unchecked((byte)(game.Witch.Angle + (byte)game.Witch.Direction));
        page.WitchDir = game.Witch.Angle;
        page.WFlag = game.Witch.OnScreen;

        if (game.Witch.OnScreen == 0)
        {
            return;
        }

        WitchBeam.WitchFx(game.Witch, game.World, game.Hero, video, video.Screen.Drawing);

        if (game.Witch.Side1 > 0 && game.Witch.Side2 < 0 && Blocking.ActorDistance(game, WITCH_SLOT, 0) < WITCH_REACH)
        {
            Hits.HitActor(game, video, -1, 0, witch.Facing, (short)(game.Rng.Rand2() + 1));
        }
    }
}
