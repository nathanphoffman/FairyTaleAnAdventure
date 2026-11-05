using FaeryTale.Game.Data;

namespace FaeryTale.Tests;

public class ObjectTablesTests
{
    private static readonly int[] CAPACITY = [13, 11, 15, 22, 13, 15, 11, 11, 78, 9];

    [Fact]
    public void EveryRegionHasRoomForItsUsedObjects()
    {
        ObjectLists lists = new();

        for (int region = 0; region < CAPACITY.Length; region++)
        {
            Assert.Equal(CAPACITY[region], lists.Region(region).Length);
            Assert.True(lists.Used[region] <= CAPACITY[region], $"region {region}");
        }
    }

    [Fact]
    public void RegionsWithRandomTreasureHaveTenSpareEntries()
    {
        ObjectLists lists = new();

        for (int region = 0; region < 8; region++)
        {
            Assert.True(lists.Used[region] + 10 <= CAPACITY[region], $"region {region}");
        }
    }

    [Fact]
    public void TheGameWorksOnItsOwnCopyOfTheObjects()
    {
        ObjectLists lists = new();

        lists.Region(1)[0].ObStat = 2;

        Assert.Equal(1, ObjectTables.InitialObList1[0].ObStat);
    }

    [Fact]
    public void ItemTranslationTableEndsWithAZeroPair()
    {
        byte[] table = ObjectTables.Itrans;

        Assert.Equal(new byte[] { 0, 0 }, table[^2..]);
        Assert.Equal(0, table.Length % 2);
    }

    [Fact]
    public void TableSizesAreRight()
    {
        Assert.Equal(64 * 4, ObjectTables.Witchpoints.Length);
        Assert.Equal(5 * 8, ObjectTables.TreasureProbs.Length);
        Assert.Equal(8 * 4, ObjectTables.WeaponProbs.Length);
        Assert.Equal(16, ObjectTables.RandTreasure.Length);
    }

    [Fact]
    public void ANewGameStartsWithItsOwnMenusAndColours()
    {
        FaeryTale.Game.Data.Game game = new();

        game.Menus[0].Enabled[0] = 99;
        game.PageColors[0] = 0x123;

        Assert.Equal(3, Tables.InitialMenus[0].Enabled[0]);
        Assert.Equal(0, ObjectTables.InitialPageColors[0]);
        Assert.Equal(3, game.World.Region);
    }
}
