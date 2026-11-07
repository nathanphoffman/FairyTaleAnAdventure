using System.Text;
using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class ItemWordsTests
{
    private const int ADVANCE = 10;
    private const int TEXT_LEFT = 16;
    private const int SWORD_SLOT = 2;

    private static FaeryTale.Game.Data.Game ReimaginedGame() => new() { Experience = Experience.Reimagined };

    private static List<ItemWord> FindIn(string line) => ItemWords.Find(ReimaginedGame(), Encoding.Latin1.GetBytes(line));

    [Fact]
    public void FindsWholeWordNamesInOrder()
    {
        List<ItemWord> found = FindIn("Julian found a Sword and a Gold Key.");

        Assert.Equal([SWORD_SLOT, 16], found.Select(word => word.Slot));
        Assert.Equal([15, 27], found.Select(word => word.Start));
        Assert.Equal([5, 8], found.Select(word => word.Length));
    }

    [Fact]
    public void IgnoresNamesInsideLongerWordsAndOtherSpelling()
    {
        Assert.Empty(FindIn("A swordsman with a sword and a Swordfish."));
    }

    [Fact]
    public void KnowsTheInfernoSkullByItsNewNameInReimagined()
    {
        Assert.Single(FindIn("You hold an Inferno Skull."));
        Assert.Empty(ItemWords.Find(new FaeryTale.Game.Data.Game { Experience = Experience.Remastered }, "You hold an Inferno Skull."u8));
    }

    [Fact]
    public void TipFindsTheItemUnderThePointerAndNothingBesideIt()
    {
        FaeryTale.Game.Data.Game game = ReimaginedGame();
        MessageHistory history = new();

        history.StartLine("Julian found a Sword."u8);

        // "Julian found a " is 15 characters, then the 5 of "Sword"; the line is the newest, its baseline row 42.
        int swordLeft = TEXT_LEFT + 15 * ADVANCE;
        int row = 278 + (42 - 3) * 2;

        Assert.Equal(SWORD_SLOT, MessageItemTip.SlotAt(game, history, _ => ADVANCE, (swordLeft + 1, row))?.Slot);
        Assert.Null(MessageItemTip.SlotAt(game, history, _ => ADVANCE, (swordLeft - 1, row)));
        Assert.Null(MessageItemTip.SlotAt(game, history, _ => ADVANCE, (swordLeft + 5 * ADVANCE, row)));
    }
}
