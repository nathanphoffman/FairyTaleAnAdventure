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

    // The pack slots of the items described below (the slot numbers of the inventory).
    private const int DIRK_SLOT = 0;
    private const int MACE_SLOT = 1;
    private const int SWORD_SLOT = 2;
    private const int BOW_SLOT = 3;
    private const int WAND_SLOT = 4;
    private const int LASSO_SLOT = 5;
    private const int SHELL_SLOT = 6;
    private const int SUN_STONE_SLOT = 7;
    private const int ARROWS_SLOT = 8;
    private const int BLUE_STONE_SLOT = 9;
    private const int GREEN_JEWEL_SLOT = 10;
    private const int GLASS_VIAL_SLOT = 11;
    private const int CRYSTAL_ORB_SLOT = 12;
    private const int BIRD_TOTEM_SLOT = 13;
    private const int GOLD_RING_SLOT = 14;
    private const int JADE_SKULL_SLOT = 15;
    private const int TALISMAN_SLOT = 22;
    private const int FOOD_SLOT = 24;
    private const int GOLD_STATUE_SLOT = 25;
    private const int WRIT_SLOT = 28;
    private const int BONES_SLOT = 29;
    private const int FIRST_KEY_SLOT = 16;
    private const int LAST_KEY_SLOT = 21;

    /// The words for an item of the pack (its slot), or "" when there is nothing to say.
    public static string For(int slot) => slot switch
    {
        DIRK_SLOT => "Damage 1 to 3 a hit. The fastest, shortest blade.",
        MACE_SLOT => "Damage 2 to 4 a hit.",
        SWORD_SLOT => "Damage 3 to 5 a hit. The best blade.",
        BOW_SLOT => "Shoots arrows: 4 to 11 damage each. Uses arrows.",
        WAND_SLOT => "Shoots fireballs: 4 to 11 damage each.",
        LASSO_SLOT => "A rope for taming the golden swan.",
        SHELL_SLOT => "Calls the turtle to carry you over water.",
        SUN_STONE_SLOT => "Makes the witch vulnerable when she is near.",
        ARROWS_SLOT => "Ammunition for the bow.",
        BLUE_STONE_SLOT => "Takes you to the next stone circle, if you stand in one.",
        GREEN_JEWEL_SLOT => $"Light: lights the dark for about {Seconds(LIGHT_TICKS)} seconds.",
        GLASS_VIAL_SLOT => "Heals you 4 to 11 health, up to a limit that grows with bravery.",
        CRYSTAL_ORB_SLOT => $"Shows what is hidden for about {Seconds(SECRET_TICKS)} seconds.",
        BIRD_TOTEM_SLOT => "Hurls a fireball that bursts into flame and burns every enemy near.",
        GOLD_RING_SLOT => $"Freezes time, so enemies stand still, for about {Seconds(FREEZE_TICKS)} seconds.",
        JADE_SKULL_SLOT => "Frightens and kills the weaker enemies in view.",
        >= FIRST_KEY_SLOT and <= LAST_KEY_SLOT => "Opens locked doors and chests of its colour.",
        TALISMAN_SLOT => "The talisman. Bring it home to win the game.",
        FOOD_SLOT => "Food. Eaten when you are hungry.",
        GOLD_STATUE_SLOT => "One of five. Find them all to reach the hidden city.",
        WRIT_SLOT => "The king's writ. Show it to the priest.",
        BONES_SLOT => "Bones to give the spectre.",
        _ => "",
    };

    private static int Seconds(int ticks) => ticks / TICKS_PER_SECOND;
}
