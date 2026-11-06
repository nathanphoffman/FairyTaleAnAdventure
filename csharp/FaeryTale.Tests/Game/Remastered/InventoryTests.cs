using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class InventoryTests
{
    private static FaeryTale.Game.Data.Game GameCarrying(int slot, byte count)
    {
        FaeryTale.Game.Data.Game game = new();

        game.Hero.Stuff[slot] = count;

        return game;
    }

    [Fact]
    public void EveryItemIsInOneSquareAndNoTwoSquaresOverlap()
    {
        List<int> slots = InventoryLayout.CELLS.Select(cell => cell.Item.Slot).ToList();

        Assert.Equal(slots.Count, slots.Distinct().Count());

        foreach (InventoryCell cell in InventoryLayout.CELLS)
        {
            Assert.Equal(cell.Item.Slot, InventoryLayout.CellAt((cell.X + 1, cell.Y + 1))!.Value.Item.Slot);
            Assert.True(InventoryLayout.PanelContains((cell.X, cell.Y)));
        }
    }

    [Fact]
    public void AClickOnAWeaponTakesItInHand()
    {
        FaeryTale.Game.Data.Game game = GameCarrying(2, 1);
        InventoryItem sword = new(2, ItemUse.Equip);

        InventoryActions.Use(game, new FaeryTale.Game.Output.Recorder(), sword);

        Assert.True(InventoryActions.IsEquipped(game, 2));
        Assert.Equal(3, game.Actors.List[0].Weapon);
    }

    [Fact]
    public void AWeaponTheHeroDoesNotHaveCannotBeEquipped()
    {
        FaeryTale.Game.Data.Game game = new();

        InventoryActions.Use(game, new FaeryTale.Game.Output.Recorder(), new InventoryItem(2, ItemUse.Equip));

        Assert.False(InventoryActions.IsEquipped(game, 2));
    }

    [Fact]
    public void TheScreenOnlyUsesUpClicksWhileOpenAndOnThePanel()
    {
        FaeryTale.Game.Data.Game game = new();
        FaeryTale.Game.Output.Recorder output = new();
        (int X, int Y) onPanel = (InventoryLayout.PANEL_LEFT + 5, InventoryLayout.PANEL_TOP + 5);

        Inventory.Close();
        Assert.False(Inventory.TryClick(game, output, onPanel));

        Inventory.Toggle();
        Assert.True(Inventory.TryClick(game, output, onPanel));
        Assert.False(Inventory.TryClick(game, output, (0, 0)));

        Inventory.Close();
    }

    [Fact]
    public void TheCloseButtonClosesIt()
    {
        FaeryTale.Game.Data.Game game = new();

        Inventory.Toggle();
        Inventory.TryClick(game, new FaeryTale.Game.Output.Recorder(), (InventoryLayout.CloseCorner.X + 2, InventoryLayout.CloseCorner.Y + 2));

        Assert.False(Inventory.IsOpen);
    }
}
