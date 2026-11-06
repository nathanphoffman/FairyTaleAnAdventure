using FaeryTale.Game.Input;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Remastered;

/// What is on the hotbar and what a click on it does. The bar fills itself: the weapons the hero carries
/// on the left (the one in hand lit), the magic stones on the right, in the order of the pack, each
/// appearing as it is picked up. A click on a weapon takes it in hand and a click on a stone casts it
/// (the F1 to F7 keys cast too, as in the classic game).
public static class Hotbar
{
    private const int FIRST_WEAPON = 0;
    private const int WEAPON_COUNT = 5;
    private const int FIRST_SPELL = 9;

    /// The slots of the weapons the hero carries.
    public static List<int> Weapons(Data.Game game) => Carried(game, FIRST_WEAPON, WEAPON_COUNT);

    /// The slots of the magic stones the hero carries.
    public static List<int> Spells(Data.Game game) => Carried(game, FIRST_SPELL, HotbarLayout.SPELL_SLOTS);

    /// The key that casts a spell (F1 for the first stone of the pack, F2 for the next, and so on).
    public static string KeyOf(int slot) => $"F{slot - FIRST_SPELL + 1}";

    /// Does what a click on the bar means. Returns true when the click was on a square (it is used up).
    public static bool TryClick(Data.Game game, IOutput output, InputHandler.Click click, int windowWidth, int windowHeight)
    {
        if (!click.Native || game.Experience == Data.Experience.Original || HotbarLayout.For(windowWidth, windowHeight) is not { } geometry)
        {
            return false;
        }

        List<HotbarSlot> slots = HotbarLayout.Slots(geometry, Weapons(game), Spells(game));

        if (HotbarLayout.SlotAt(geometry, slots, click.X, click.Y) is not { } slot)
        {
            return false;
        }

        if (slot.IsSpell)
        {
            InventoryActions.Cast(game, output, slot.Item);
        }
        else
        {
            InventoryActions.Equip(game, slot.Item);
        }

        return true;
    }

    // The slots of a run of the pack that the hero has at least one of.
    private static List<int> Carried(Data.Game game, int first, int count) =>
        Enumerable.Range(first, count).Where(slot => InventoryActions.CountOf(game, slot) > 0).ToList();
}
