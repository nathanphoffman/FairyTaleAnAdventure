namespace FaeryTale.Game.Remastered;

/// A titled group of items on the inventory screen.
public sealed record InventorySection(string Title, InventoryItem[] Items);
