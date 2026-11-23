using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Land;

/// Objects: where treasure goes.
public static partial class Objects
{
    // Whether value is strictly between the two limits.
    private static bool Inside(short value, short low, short high) => value > low && value < high;

    // Scatters random treasure over a region the first time it is visited: 10 items at random
    // spots, retrying until the spot is not blocked (terrain type 0 means clear), with the item
    // picked from RandTreasure. Distributed remembers that it has been done.
    private static void DistributeTreasure(Data.Game game)
    {
        int region = game.World.Region;

        for (int count = 0; count < RANDOM_TREASURES; count++)
        {
            short x;
            short y;

            do
            {
                x = (short)(game.Rng.BitRand(REGION_WIDTH - 1) + (game.World.Region & 1) * REGION_WIDTH);
                y = (short)(game.Rng.BitRand(REGION_HEIGHT - 1) + (game.World.Region & 6) * REGION_ROW_STEP);
            }

            while (MapLookup.TerrainAt(game, x, y) != 0);

            int slot = game.Objects.Used[region];
            game.Objects.Used[region]++;

            sbyte kind = (sbyte)ObjectTables.RandTreasure[game.Rng.BitRand(TREASURE_KINDS - 1)];
            WorldObject[] list = game.Objects.Region(region);

            list[slot].Xc = (ushort)x;
            list[slot].Yc = (ushort)y;
            list[slot].ObId = kind;
            list[slot].ObStat = STAT_SHOWN;
        }

        game.Objects.Distributed[region] = 1; // mark this region as distributed
    }
}
