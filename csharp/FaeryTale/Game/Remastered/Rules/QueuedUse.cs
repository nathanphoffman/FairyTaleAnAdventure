using FaeryTale.Game.Output;

namespace FaeryTale.Game.Remastered;

/// Hotbar squares clicked while the game is paused (Remastered and Reimagined): the weapon to take in
/// hand and the magic stone to cast as soon as the game goes on. A second click on a queued square
/// takes it out of the queue again, and a newer weapon (or stone) replaces the older one.
public static class QueuedUse
{
    private static int? weapon;
    private static int? spell;

    /// Whether the square of this item is waiting for the game to go on (it gets a dotted border).
    public static bool Contains(int slot) => slot == weapon || slot == spell;

    /// Queues the item, or takes it out of the queue if it was queued.
    public static void Toggle(int slot, bool isSpell)
    {
        if (isSpell)
        {
            spell = spell == slot ? null : slot;
        }
        else
        {
            weapon = weapon == slot ? null : slot;
        }
    }

    /// Does what was queued (the weapon first, so a spell is cast with it in hand) and empties the queue.
    public static void Run(Data.Game game, IOutput output)
    {
        if (weapon is { } weaponSlot)
        {
            InventoryActions.Equip(game, weaponSlot);
        }

        if (spell is { } spellSlot)
        {
            InventoryActions.Cast(game, output, spellSlot);
        }

        Reset();
    }

    /// Forgets everything queued (a new game starts).
    public static void Reset()
    {
        weapon = null;
        spell = null;
    }
}
