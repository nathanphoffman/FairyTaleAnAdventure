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
