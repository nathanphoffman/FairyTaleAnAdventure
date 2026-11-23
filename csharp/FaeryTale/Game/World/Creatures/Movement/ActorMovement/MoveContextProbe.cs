using FaeryTale.Game.Output;

namespace FaeryTale.Game.Creatures.Movement;

/// Gives the tests access to the pure helpers of the movement code without exposing its internals.
internal static class MoveContextProbe
{
    internal static bool SwanCarries(short nearness, short carrierSlot, bool hasLasso) => MoveContext.SwanCarries(nearness, carrierSlot, hasLasso);

    internal static bool TurtleCarries(short nearness, short carrierSlot) => MoveContext.TurtleCarries(nearness, carrierSlot);

    internal static short TurtleHeading(short direction, Func<short, bool> waterAhead) => MoveContext.TurtleHeading(direction, waterAhead);

    internal static short? FrustratedFrame(sbyte frustration, short cycle) => MoveContext.FrustratedFrame(frustration, cycle);

    internal static short StepLength(bool heroOnTurtle, sbyte environ) => MoveContext.StepLength(heroOnTurtle, environ);

    internal static bool ClimbsOutOfWater(sbyte k, short j) => MoveContext.ClimbsOutOfWater(k, j);

    internal static (sbyte, string) WadeStep(sbyte depth, short j, bool deadOrDying)
    {
        (sbyte newDepth, MoveContext.Wading outcome) = MoveContext.WadeStep(depth, j, deadOrDying);

        return (newDepth, outcome.ToString());
    }
}
