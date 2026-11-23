namespace FaeryTale.Game.Data;

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
