namespace FaeryTale.Game.Remastered;

/// An item's square on the inventory screen (a place on the 640 x 400 overlay grid).
public readonly record struct InventoryCell(InventoryItem Item, int X, int Y);
