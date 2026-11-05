using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Creatures.Interaction;

/// Where encounters and carriers appear: picking a spot, summoning the turtle and centring an extent.
public static class Extents
{
    private const int TURTLE_TRIES = 25;
    private const int WATER = 5;
    private const short TURTLE_EXTENT = 1;
    private const short TURTLE_CARRIER = 5;

    // An extent covers 500 by 400 pixels around its centre.
    private const int HALF_WIDTH = 250;
    private const int HALF_HEIGHT = 200;

    // An encounter appears this far away: 150 plus up to 63.
    private const int ENCOUNTER_MIN_DISTANCE = 150;

    /// Places the turtle: tries up to 25 random spots around the hero for one on water (terrain
    /// 5), then moves the turtle extent there and loads the turtle as the carrier. Called when
    /// turtle eggs are in view.
    public static void SummonTurtle(Data.Game game, IOutput output)
    {
        int triesUsed = 0;

        while (triesUsed < TURTLE_TRIES)
        {
            PickEncounterSpot(game);

            if (MapLookup.TerrainAt(game, game.Encounter.X, game.Encounter.Y) == WATER)
            {
                break;
            }

            triesUsed++;
        }

        if (triesUsed == TURTLE_TRIES)
        {
            return;
        }

        CenterExtentOn(game, TURTLE_EXTENT, (short)game.Encounter.X, (short)game.Encounter.Y);
        Encounters.LoadCarrier(game, output, TURTLE_CARRIER);
    }

    /// Centres extent e on a point (it covers 500 by 400 pixels around it).
    public static void CenterExtentOn(Data.Game game, short e, short x, short y)
    {
        game.Extents[e].X1 = (ushort)(x - HALF_WIDTH);
        game.Extents[e].Y1 = (ushort)(y - HALF_HEIGHT);
        game.Extents[e].X2 = (ushort)(x + HALF_WIDTH);
        game.Extents[e].Y2 = (ushort)(y + HALF_HEIGHT);
    }

    /// Picks where a new encounter appears: a random direction and distance from the hero (150 to
    /// 213 pixels).
    public static void PickEncounterSpot(Data.Game game)
    {
        int direction = game.Rng.Rand8();
        int distance = ENCOUNTER_MIN_DISTANCE + game.Rng.Rand64();

        game.Encounter.X = (ushort)MapSteps.StepX(game.Hero.X, direction, distance);
        game.Encounter.Y = (ushort)MapSteps.StepY(game.Hero.Y, direction, distance);
    }
}
