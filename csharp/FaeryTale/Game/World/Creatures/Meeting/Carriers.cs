using FaeryTale.Game.Data;

namespace FaeryTale.Game.Creatures;

/// How near the hero is to the raft, turtle or swan that can carry them.
public static class Carriers
{
    private const short BIRD = 11;
    private const sbyte TURTLE_ENVIRON = -2;

    // The hero counts as near the raft or turtle within this many pixels (very near within the
    // smaller one).
    private const short NEAR = 16;
    private const short VERY_NEAR = 9;

    // The carrier is the raft (slot 1), or slot 3 when a turtle or bird is out.
    private const short RAFT_SLOT = 1;
    private const short CARRIER_SLOT = 3;

    /// The hero's centre is a little off the carrier's.
    private const int CARRIER_OFFSET = 4;

    /// How close the hero is to the raft or turtle (RaftNearness 1 = near, 2 = very near). Riding
    /// the bird sets the hero's environ.
    public static void RaftProximity(Data.Game game)
    {
        game.Actors.RaftNearness = 0;
        game.Actors.TurtleNearness = 0;
        game.Actors.CarrierSlot = game.Actors.ActiveCarrier != 0 ? CARRIER_SLOT : RAFT_SLOT;

        Shape carrier = game.Actors.List[game.Actors.CarrierSlot];
        Shape hero = game.Actors.List[0];
        short dx = (short)(hero.AbsX - carrier.AbsX - CARRIER_OFFSET);
        short dy = (short)(hero.AbsY - carrier.AbsY - CARRIER_OFFSET);

        game.Actors.RaftNearness = Nearness(dx, dy);

        if (game.Hero.Riding == BIRD)
        {
            hero.Environ = TURTLE_ENVIRON;
        }
    }

    /// 0 when the hero is not near, 1 when near and 2 when very near the carrier (offsets in pixels).
    internal static short Nearness(short dx, short dy)
    {
        if (dx < VERY_NEAR && dx > -VERY_NEAR && dy < VERY_NEAR && dy > -VERY_NEAR)
        {
            return 2;
        }

        return dx < NEAR && dx > -NEAR && dy < NEAR && dy > -NEAR ? (short)1 : (short)0;
    }
}
