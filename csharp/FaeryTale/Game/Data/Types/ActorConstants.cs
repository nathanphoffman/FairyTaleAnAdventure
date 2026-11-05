namespace FaeryTale.Game.Data;

/// What sort of thing an actor is (Shape.Kind; the sequence numbers of the original).
public static class ActorKind
{
    public const sbyte OBJECTS = 1;
    public const sbyte ENEMY = 2;
    public const sbyte RAFT = 3;
    public const sbyte SETFIG = 4;
    public const sbyte CARRIER = 5;
    public const sbyte DRAGON = 6;
}

/// What an actor is doing (Shape.State).
public static class ActorState
{
    public const sbyte FIGHTING = 0;
    public const sbyte WALKING = 12;
    public const sbyte STILL = 13;
    public const sbyte DYING = 14;
    public const sbyte DEAD = 15;
    public const sbyte SINK = 16;
    public const sbyte TALKING = 19;
    public const sbyte FALL = 22;
    public const sbyte SLEEP = 23;
    public const sbyte SHOOT1 = 24;
    public const sbyte SHOOT3 = 25;
}

/// What an actor is trying to do (Shape.Goal).
public static class ActorGoal
{
    public const sbyte ATTACK1 = 1;
    public const sbyte ATTACK2 = 2;
    public const sbyte ARCHER1 = 3;
    public const sbyte ARCHER2 = 4;
    public const sbyte FLEE = 5;
    public const sbyte STAND = 6;
    public const sbyte DEATH = 7;
    public const sbyte WAIT = 8;
    public const sbyte FOLLOWER = 9;
    public const sbyte CONFUSED = 10;
}

/// How an enemy goes about its goal (Shape.Tactic).
public static class ActorTactic
{
    /// Tactic values 0 and 9 mean the enemy is stuck.
    public const int FRUSTRATED = 0;
    public const int SHOOT_FRUSTRATED = 9;
    public const int PURSUE = 1;
    public const int FOLLOW = 2;
    public const int BUMBLE_SEEK = 3;
    public const int RANDOM = 4;
    public const int BACKUP = 5;
    public const int EVADE = 6;
    public const int SHOOT = 8;
    public const int EGG_SEEK = 10;
}

/// Values of Ui.ViewStatus that other code reacts to.
public static class ViewStatus
{
    /// The whole screen is rebuilt.
    public const sbyte REBUILD_ALL = 99;
}

/// Entries of the print queue (what the screen code should redraw).
public static class PrintRequest
{
    public const int VITALITY = 4;
    public const int STATS = 7;
}
