using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class HotbarTests
{
    private static FaeryTale.Game.Data.Game Carrying(params (int Slot, byte Count)[] items)
    {
        FaeryTale.Game.Data.Game game = new();

        foreach ((int slot, byte count) in items)
        {
            game.Hero.Stuff[slot] = count;
        }

        return game;
    }

    [Fact]
    public void TheBarShowsOnlyTheWeaponsAndStonesTheHeroCarries()
    {
        FaeryTale.Game.Data.Game game = Carrying((0, 1), (2, 1), (10, 3), (14, 1), (16, 2));

        Assert.Equal([0, 2], Hotbar.Weapons(game));
        Assert.Equal([10, 14], Hotbar.Spells(game));
    }

    [Fact]
    public void AStoneIsCastWithTheFKeyForItsPlaceInThePack()
    {
        Assert.Equal("F1", Hotbar.KeyOf(9));
        Assert.Equal("F7", Hotbar.KeyOf(15));
    }

    [Fact]
    public void ThereIsRoomForTheBarOnALargeWindowButNotASmallOne()
    {
        Assert.NotNull(HotbarLayout.For(1920, 1080));
        Assert.NotNull(HotbarLayout.For(1280, 720));
        Assert.Null(HotbarLayout.For(800, 600));
    }

    [Fact]
    public void EachSquareIsFoundWhereItIsDrawnAndTheyDoNotOverlap()
    {
        HotbarGeometry geometry = HotbarLayout.For(1920, 1080)!;
        List<HotbarSlot> slots = HotbarLayout.Slots(geometry, [0, 1, 2], [9, 10]);

        Assert.Equal(5, slots.Count);

        foreach (HotbarSlot slot in slots)
        {
            Assert.Equal(slot, HotbarLayout.SlotAt(geometry, slots, slot.X + 1, slot.Y + 1));
        }

        Assert.Null(HotbarLayout.SlotAt(geometry, slots, 0, 0));
    }

    [Fact]
    public void TheWeaponsAreOnTheLeftOfThePanelAndTheStonesOnTheRight()
    {
        HotbarGeometry geometry = HotbarLayout.For(1920, 1080)!;
        List<HotbarSlot> slots = HotbarLayout.Slots(geometry, [0], [9]);

        Assert.True(slots.Single(slot => !slot.IsSpell).X + geometry.SlotSize <= geometry.BoardLeft);
        Assert.True(slots.Single(slot => slot.IsSpell).X >= geometry.BoardRight);
    }
}
