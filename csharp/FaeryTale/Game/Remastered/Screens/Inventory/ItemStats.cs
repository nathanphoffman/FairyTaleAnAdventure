namespace FaeryTale.Game.Remastered;

/// What each item does, in a few words, for the inventory screen. The numbers are the game's own (the
/// weapon strengths, how long each spell lasts, how much the vial heals) before the difficulty level
/// changes the damage. Items whose use is only part of the story say so.
public static class ItemStats
{
    // How many game ticks make a second (the game moves 50 times a second).
    private const int TICKS_PER_SECOND = 50;

    // The spell times, in ticks (as the Magic menu has them).
    private const int LIGHT_TICKS = 760;
    private const int SECRET_TICKS = 360;
    private const int FREEZE_TICKS = 100;

    /// The words for an item of the pack (its slot), or "" when there is nothing to say.
    public static string For(int slot) => slot switch
    {
        0 => "Damage 1 to 3 a hit. The fastest, shortest blade.",
        1 => "Damage 2 to 4 a hit.",
        2 => "Damage 3 to 5 a hit. The best blade.",
        3 => "Shoots arrows: 4 to 11 damage each. Uses arrows.",
        4 => "Shoots fireballs: 4 to 11 damage each.",
        5 => "A rope for taming the golden swan.",
        6 => "Calls the turtle to carry you over water.",
        7 => "Makes the witch vulnerable when she is near.",
        8 => "Ammunition for the bow.",
        9 => "Takes you to the next stone circle, if you stand in one.",
        10 => $"Light: lights the dark for about {Seconds(LIGHT_TICKS)} seconds.",
        11 => "Heals you 4 to 11 health, up to a limit that grows with bravery.",
        12 => $"Shows what is hidden for about {Seconds(SECRET_TICKS)} seconds.",
        13 => "Shows the map of the land.",
        14 => $"Freezes time, so enemies stand still, for about {Seconds(FREEZE_TICKS)} seconds.",
        15 => "Frightens and kills the weaker enemies in view.",
        >= 16 and <= 21 => "Opens locked doors and chests of its colour.",
        22 => "The talisman. Bring it home to win the game.",
        24 => "Food. Eaten when you are hungry.",
        25 => "One of five. Find them all to reach the hidden city.",
        28 => "The king's writ. Show it to the priest.",
        29 => "Bones to give the spectre.",
        _ => "",
    };

    private static int Seconds(int ticks) => ticks / TICKS_PER_SECOND;
}
