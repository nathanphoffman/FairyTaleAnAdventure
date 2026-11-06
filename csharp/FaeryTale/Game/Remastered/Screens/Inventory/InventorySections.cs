namespace FaeryTale.Game.Remastered;

/// What kind of thing an item is, and so what a click on it does.
public enum ItemUse
{
    /// Nothing: it is carried (treasure and quest items, supplies).
    None,

    /// A weapon: clicking it takes it in hand.
    Equip,

    /// A magic stone: clicking it casts it.
    Cast,

    /// A key: clicking it tries it on the doors and chests round the hero.
    Unlock,

    /// A special item the Use menu knows (the sea shell, the sun stone).
    UseSpecial,
}

/// One item of the inventory: its slot in the hero's pack, and what a click on it does.
public readonly record struct InventoryItem(int Slot, ItemUse Use);

/// A titled group of items on the inventory screen.
public sealed record InventorySection(string Title, InventoryItem[] Items);

/// How the inventory screen groups the hero's items. The slots are those of the hero's pack (the
/// same numbers the item list and the menus use).
public static class InventorySections
{
    // Slots of the pack (see the item list table).
    private const int FIRST_WEAPON = 0;
    private const int LASSO = 5;
    private const int SEA_SHELL = 6;
    private const int SUN_STONE = 7;
    private const int ARROWS = 8;
    private const int FIRST_MAGIC = 9;
    private const int FIRST_KEY = 16;
    private const int TALISMAN = 22;
    private const int ROSE = 23;
    private const int FRUIT = 24;
    private const int GOLD_STATUE = 25;
    private const int BOOK = 26;
    private const int HERB = 27;
    private const int WRIT = 28;
    private const int BONE = 29;
    private const int SHARD = 30;

    private const int WEAPON_COUNT = 5;
    private const int MAGIC_COUNT = 7;
    private const int KEY_COUNT = 6;

    /// Weapons you hold in hand (one is equipped at a time).
    public static readonly InventorySection WEAPONS = new("Weapons", Range(FIRST_WEAPON, WEAPON_COUNT, ItemUse.Equip));

    /// The magic stones, each used up when cast.
    public static readonly InventorySection MAGIC = new("Magic", Range(FIRST_MAGIC, MAGIC_COUNT, ItemUse.Cast));

    /// Things that are eaten, shot or otherwise run out.
    public static readonly InventorySection SUPPLIES = new("Supplies",
    [
        new(ARROWS, ItemUse.None),
        new(FRUIT, ItemUse.None),
        new(HERB, ItemUse.None),
    ]);

    public static readonly InventorySection KEYS = new("Keys", Range(FIRST_KEY, KEY_COUNT, ItemUse.Unlock));

    /// Treasures and things the story asks for.
    public static readonly InventorySection TREASURES = new("Treasures",
    [
        new(GOLD_STATUE, ItemUse.None),
        new(TALISMAN, ItemUse.None),
        new(ROSE, ItemUse.None),
        new(BOOK, ItemUse.None),
        new(WRIT, ItemUse.None),
        new(BONE, ItemUse.None),
        new(SHARD, ItemUse.None),
        new(SEA_SHELL, ItemUse.UseSpecial),
        new(SUN_STONE, ItemUse.UseSpecial),
        new(LASSO, ItemUse.None),
    ]);

    /// The sections of the left column and of the right column, top to bottom.
    public static readonly InventorySection[] LEFT = [WEAPONS, MAGIC, SUPPLIES];

    public static readonly InventorySection[] RIGHT = [KEYS, TREASURES];

    // A run of slots one after another, all used the same way.
    private static InventoryItem[] Range(int first, int count, ItemUse use) =>
        Enumerable.Range(first, count).Select(slot => new InventoryItem(slot, use)).ToArray();
}
