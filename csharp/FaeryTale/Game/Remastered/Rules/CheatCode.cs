using FaeryTale.Game.Output;
using FaeryTale.Game.Reimagined;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// A cheat for testing (Remastered and Reimagined): typing GRAPHIX while playing gives the hero one or
/// more of every item, and some gold. The letters still do what they normally do as they are typed, so
/// the inventory may be open after it (which is handy). The talisman is left out, as having it ends the
/// game at the next thing picked up.
public static class CheatCode
{
    private const string WORD = "GRAPHIX";

    // How many of each run of the pack's slots the cheat gives (see the item list for the slots).
    private static readonly (int First, int Last, byte Count)[] GIVEN =
    [
        (0, 7, 1),
        (8, 8, 50),
        (9, 12, 5),
        (14, 15, 5),
        (16, 21, 3),
        (23, 23, 1),
        (24, 24, 3),
        (25, 30, 1),
    ];

    private const int GOLD_GIVEN = 200;
    private const byte SKULLS_GIVEN = 5;

    // Names of keys that are only held down with another (they do not end a word being typed).
    private static readonly string[] MODIFIER_KEYS = ["Left Shift", "Right Shift", "Left Ctrl", "Right Ctrl", "Left Alt", "Right Alt", "Left GUI", "Right GUI", "CapsLock"];

    private static readonly object padlock = new();
    private static bool inventoryHeld;
    private static string typed = "";
    private static bool completed;

    /// Notes a letter that was typed.
    public static void Type(char letter)
    {
        lock (padlock)
        {
            typed = (typed + char.ToUpperInvariant(letter))[^Math.Min(WORD.Length, typed.Length + 1)..];

            if (typed == WORD)
            {
                completed = true;
                typed = "";
            }
        }
    }

    /// Whether a press of the I key should wait to see if it is the I of the word: it is held back when the
    /// letters typed so far are the word up to and including its I, and the inventory opens (or closes)
    /// only if the word is not finished. A second I (as in GRAPHII) is then the one that opens it.
    public static bool HoldInventoryKey()
    {
        lock (padlock)
        {
            inventoryHeld = typed.EndsWith(WORD[..(WORD.IndexOf('I') + 1)], StringComparison.Ordinal);

            return inventoryHeld;
        }
    }

    /// Whether a key that came after a held I means the I was a real press of the key (so the inventory
    /// should now be opened or closed): any key but another I or the last letter of the word.
    public static bool ShouldReleaseHeldInventoryKey(string key)
    {
        lock (padlock)
        {
            if (!inventoryHeld || key.Equals("I", StringComparison.OrdinalIgnoreCase) || MODIFIER_KEYS.Contains(key))
            {
                return false;
            }

            inventoryHeld = false;

            return !completed;
        }
    }

    /// Whether the word was typed since the last call.
    public static bool TakeCompleted()
    {
        lock (padlock)
        {
            bool done = completed;

            completed = false;

            return done;
        }
    }

    /// Gives the hero the items.
    public static void GiveEverything(Data.Game game, IOutput output)
    {
        foreach ((int first, int last, byte count) in GIVEN)
        {
            for (int slot = first; slot <= last; slot++)
            {
                game.Hero.Stuff[slot] = count;
            }
        }

        if (game.Experience == Data.Experience.Reimagined)
        {
            game.Hero.Stuff[InfernoSkull.SKULL_SLOT] = SKULLS_GIVEN;
        }

        game.Hero.Wealth += GOLD_GIVEN;
        Menus.UpdateMenuAvailability(game);
        output.Print("All Items Unlocked:");
        output.Print("You're like a kid in an Amiga store in 1988");
    }
}
