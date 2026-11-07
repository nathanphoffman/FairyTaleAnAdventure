using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class MagicSwapTests
{
    [Fact]
    public void ABirdTotemBecomesAnotherStoneInRemastered()
    {
        FaeryTale.Game.Data.Game game = new();

        game.Experience = Experience.Remastered;

        int[] others = [9, 10, 11, 12, 14, 15];

        for (int tries = 0; tries < 50; tries++)
        {
            Assert.Contains(MagicSwap.Resolve(game, MagicSwap.BIRD_TOTEM), others);
        }
    }

    [Fact]
    public void EverythingElseAndEverythingInTheOriginalStaysAsItIs()
    {
        FaeryTale.Game.Data.Game remastered = new();
        FaeryTale.Game.Data.Game original = new();

        remastered.Experience = Experience.Remastered;
        original.Experience = Experience.Original;

        Assert.Equal(11, MagicSwap.Resolve(remastered, 11));
        Assert.Equal(MagicSwap.BIRD_TOTEM, MagicSwap.Resolve(original, MagicSwap.BIRD_TOTEM));
    }

    [Fact]
    public void InReimaginedTheTotemSlotHoldsTheInfernoSkullAndShowsInTheInventoryAndBar()
    {
        FaeryTale.Game.Data.Game game = new() { Experience = Experience.Reimagined };

        game.Hero.Stuff[MagicSwap.BIRD_TOTEM] = 1;

        Assert.Equal(MagicSwap.BIRD_TOTEM, MagicSwap.Resolve(game, MagicSwap.BIRD_TOTEM));
        Assert.Contains(InventoryLayout.CELLS, cell => cell.Item.Slot == MagicSwap.BIRD_TOTEM);
        Assert.Contains(MagicSwap.BIRD_TOTEM, Hotbar.Spells(game));
    }
}
