namespace FaeryTale.Game.Remastered;

/// One item of the inventory: its slot in the hero's pack, and what a click on it does.
public readonly record struct InventoryItem(int Slot, ItemUse Use);
