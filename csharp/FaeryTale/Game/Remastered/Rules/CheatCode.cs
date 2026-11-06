using FaeryTale.Game.Output;
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
        (9, 15, 5),
        (16, 21, 3),
        (23, 23, 1),
        (24, 24, 3),
        (25, 30, 1),
    ];

    private const int GOLD_GIVEN = 200;

    private static readonly object padlock = new();
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

        game.Hero.Wealth += GOLD_GIVEN;
        Menus.UpdateMenuAvailability(game);
        output.Print("All items unlocked.");
    }
}
