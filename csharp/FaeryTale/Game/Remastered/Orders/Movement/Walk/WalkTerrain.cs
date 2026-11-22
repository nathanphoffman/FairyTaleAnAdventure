using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Input;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// What the terrain costs a click-to-move walk: blocked, open, or wading (worth avoiding).
internal static class WalkTerrain
{
    // Terrain the planner avoids: deep water, and the wading types cost extra so a dry way wins.
    private const int DEEP_WATER = 5;

    private const int WADING_FIRST = 3;

    private const int WADING_LAST = 4;

    private const int WADING_COST = 60;

    /// Probes for terrain are this far below the hero's position (as in the collision check).
    private const int FEET_OFFSET = 2;

    /// What entering the world position costs the planner: blocked (-1), open (0), or open but
    /// worth avoiding (wading). Uses the same blocking check as the hero's own steps.
    internal static int CellCost(Data.Game game, int x, int y)
    {
        if (Blocking.BlockedAt(game, (short)x, (short)y, 0) != 0)
        {
            return -1;
        }

        int terrain = MapLookup.TerrainAt(game, x, y + FEET_OFFSET);

        if (terrain == DEEP_WATER)
        {
            return game.Walk.Route.Swimming ? WADING_COST : -1;
        }

        return terrain is >= WADING_FIRST and <= WADING_LAST ? WADING_COST : 0;
    }

    // Whether a world position is water (wading or deep).
    internal static bool IsWater(Data.Game game, (int X, int Y) place) =>
        MapLookup.TerrainAt(game, place.X, place.Y + FEET_OFFSET) is >= WADING_FIRST and <= DEEP_WATER;

    // Divides rounding to the nearest whole number (halves away from zero).
    internal static int RoundDivide(int value, int by) => (int)Math.Round((double)value / by, MidpointRounding.AwayFromZero);
}
