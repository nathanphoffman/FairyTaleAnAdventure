using FaeryTale.Game.Output;

namespace FaeryTale.Game.Creatures.Movement;

/// Moving and animating one actor per frame.
public static class ActorMovement
{
    /// Moves and animates actor i; cycle is the frame counter. lastTerrain is the main loop's j,
    /// which carries the last terrain value from one actor to the next (a dead actor reads it
    /// without setting it), so it is passed in and written back.
    public static void MoveActor(Data.Game game, IOutput output, int i, short cycle, ref short lastTerrain) =>
        MoveContext.MoveActor(game, output, i, cycle, ref lastTerrain);
}

/// Gives the tests access to the pure helpers of the movement code without exposing its internals.
internal static class MoveContextProbe
{
    internal static bool SwanCarries(short nearness, short carrierSlot, bool hasLasso) => MoveContext.SwanCarries(nearness, carrierSlot, hasLasso);

    internal static bool TurtleCarries(short nearness, short carrierSlot) => MoveContext.TurtleCarries(nearness, carrierSlot);

    internal static short TurtleHeading(short dir, Func<short, bool> waterAhead) => MoveContext.TurtleHeading(dir, waterAhead);

    internal static short? FrustratedFrame(sbyte frustration, short cycle) => MoveContext.FrustratedFrame(frustration, cycle);

    internal static short StepLength(bool heroOnTurtle, sbyte environ) => MoveContext.StepLength(heroOnTurtle, environ);

    internal static bool ClimbsOutOfWater(sbyte k, short j) => MoveContext.ClimbsOutOfWater(k, j);

    internal static (sbyte, string) WadeStep(sbyte depth, short j, bool deadOrDying)
    {
        (sbyte newDepth, MoveContext.Wading outcome) = MoveContext.WadeStep(depth, j, deadOrDying);

        return (newDepth, outcome.ToString());
    }
}
