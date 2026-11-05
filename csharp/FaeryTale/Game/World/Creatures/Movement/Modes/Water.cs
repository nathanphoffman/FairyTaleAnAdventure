using FaeryTale.Game.Data;
using FaeryTale.Game.Land;

namespace FaeryTale.Game.Creatures.Movement;

// Wading into water: sinking, drowning, climbing back out.
internal sealed partial class MoveContext
{
    /// What wading does to an actor this frame.
    internal enum Wading
    {
        Nothing,

        /// Fully under: the actor stops moving (or falls through the pit).
        Drowned,

        /// Deep enough to show the sinking frame.
        Sinking,
    }

    // Depth of water that starts to matter, and how deep each wadeable terrain may be before the
    // actor climbs out: terrain 3 up to depth 5, terrain 4 up to depth 10.
    private const sbyte WATER_DEPTH_MIN = 2;
    private const sbyte WADE_LIMIT_SHALLOW = 5;
    private const sbyte WADE_LIMIT_DEEP = 10;

    // The hero who drowns in the pit goes to the dungeon region and arrives here.
    private const ushort DUNGEON_REGION = 9;
    private const ushort DUNGEON_ARRIVAL_X = 0x1080;
    private const ushort DUNGEON_ARRIVAL_Y = 34950;

    /// Standing in water (depth k above 2) on ground that is dry (j 0), or wading with a depth
    /// over the limit of the terrain (3: 5, 4: 10): the actor climbs out one step a frame.
    internal static bool ClimbsOutOfWater(sbyte k, short j) =>
        k > WATER_DEPTH_MIN
        && (j == 0 || (j == TERRAIN_WADE && k > WADE_LIMIT_SHALLOW) || (j == TERRAIN_SLOW_SINK && k > WADE_LIMIT_DEEP));

    /// The depth after one frame in water of terrain type j (shallow 4 or deep 5), and what that
    /// does. The depth moves one step towards its limit; the dead are not affected.
    internal static (sbyte Depth, Wading Outcome) WadeStep(sbyte depth, short j, bool deadOrDying)
    {
        sbyte limit = j == TERRAIN_SLOW_SINK ? WADE_LIMIT_DEEP : ENVIRON_DROWNED;

        if (depth > limit)
        {
            return ((sbyte)(depth - 1), Wading.Nothing);
        }

        if (depth >= limit)
        {
            return (depth, Wading.Nothing);
        }

        sbyte deeper = (sbyte)(depth + 1);

        if (deadOrDying)
        {
            return (deeper, Wading.Nothing);
        }

        if (deeper == ENVIRON_DROWNED)
        {
            return (deeper, Wading.Drowned);
        }

        return (deeper, deeper > ENVIRON_SINKING ? Wading.Sinking : Wading.Nothing);
    }

    // In water (terrain 4 is shallow, 5 deep): the depth moves one step towards its limit, and the
    // actor sinks, or at the pit sector falls through to the dungeon.
    private void Wade(short j)
    {
        (sbyte depth, Wading outcome) = WadeStep(environ, j, state == ActorState.DYING || state == ActorState.DEAD);

        environ = depth;

        if (outcome == Wading.Sinking)
        {
            actor.State = ActorState.SINK;
        }
        else if (outcome == Wading.Drowned)
        {
            Drown();
        }
    }

    // Fully under: at the pit sector the hero falls through to the dungeon and others die; the
    // actor stops moving.
    private void Drown()
    {
        if (game.Hero.Sector == PIT_SECTOR)
        {
            if (slot == 0)
            {
                environ = 0;
                game.World.NewRegion = DUNGEON_REGION;
                Places.TransferHero(game, output, DUNGEON_ARRIVAL_X, DUNGEON_ARRIVAL_Y, 0);
                Places.FindPlace(game, output, 1);
            }
            else
            {
                actor.Vitality = 0;
            }
        }

        actor.State = ActorState.STILL;
    }
}
