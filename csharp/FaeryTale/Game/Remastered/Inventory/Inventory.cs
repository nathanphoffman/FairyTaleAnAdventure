using FaeryTale.Game.Output;

namespace FaeryTale.Game.Remastered;

/// The inventory screen (Remastered and Reimagined): opened and closed with I, or closed with the X.
/// It is only drawn over the game, which keeps running, so the hero can be walking or fighting with it
/// open. A click on it is used up (it equips or uses an item, or closes the screen); a click anywhere
/// else is for the world as usual.
public static class Inventory
{
    private static volatile bool open;

    /// Whether the inventory is on the screen.
    public static bool IsOpen => open;

    /// Opens it if closed and closes it if open (the I key).
    public static void Toggle() => open = !open;

    /// Closes it.
    public static void Close() => open = false;

    /// Does what a click at a place on the overlay grid means. Returns true when the click was on the
    /// inventory (it is used up) and false when it was not, or the inventory is closed.
    public static bool TryClick(Data.Game game, IOutput output, (int X, int Y) place)
    {
        if (!open || !InventoryLayout.PanelContains(place))
        {
            return false;
        }

        if (InventoryLayout.CloseContains(place))
        {
            Close();
        }
        else if (InventoryLayout.CellAt(place) is { } cell)
        {
            InventoryActions.Use(game, output, cell.Item);
        }

        return true;
    }
}
