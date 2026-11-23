namespace FaeryTale.Game;

/// The kinds of command a test script can queue (FTA_CMD).
public enum CommandKind
{
    /// Move the hero to (First, Second), like going through a door.
    Teleport,

    /// Kill the hero, who has First luck left.
    KillHero,

    /// Set inventory slot First to a count of Second.
    Give,

    /// Set a stat First (0 hunger, 1 fatigue, anything else wealth) to Second.
    SetStat,

    /// Kill every enemy that is out.
    SlayEnemies,
}
