namespace FaeryTale.Game.Data;

/// Names one actor of Actors.List by its number, so code can hold on to "that actor" while it
/// calls others that change the list.
public readonly record struct ActorRef(int Slot);

/// The same for the missiles in flight.
public readonly record struct MissileRef(int Slot);

/// The actors on the map, the missiles in flight, and who is near the hero.
public sealed class Actors
{
    public const int ACTOR_SLOTS = 20;
    public const int MISSILE_SLOTS = 6;

    public Shape[] List = NewArray(ACTOR_SLOTS, () => new Shape());
    public short Active;
    public short Drawn;
    public Missile[] Missiles = NewArray(MISSILE_SLOTS, () => new Missile());
    public short NextMissile;
    public short NearestObject;
    public short NearestPerson;
    public short LastPerson;
    public short Leader;
    public byte[] DrawOrder = new byte[ACTOR_SLOTS];
    public sbyte Loading;
    public sbyte OnScreen;
    public short ActiveCarrier;
    public short CarrierSlot;
    public short RaftNearness;
    public short TurtleNearness;
    public short NearestPersonDistance;
    public short LoadedActorFile;
    public short LoadedSetfigFile;
    public sbyte BattleFlag;
    public sbyte Frustration;
    public sbyte FieryDeath;

    /// Where the actor being moved would be after this step.
    public ushort TestX;
    public ushort TestY;

    /// The first free actor slot while the object lists are being turned into actors.
    public short FreeSlot;

    /// The slot of the turtle's eggs on the actor list (0 if there are none).
    public sbyte TurtleEggs;

    /// The actor r names.
    public Shape Actor(ActorRef r) => List[r.Slot];

    /// The missile r names.
    public Missile Missile(MissileRef r) => Missiles[r.Slot];

    private static T[] NewArray<T>(int length, Func<T> make)
    {
        T[] items = new T[length];

        for (int index = 0; index < length; index++)
        {
            items[index] = make();
        }

        return items;
    }
}
