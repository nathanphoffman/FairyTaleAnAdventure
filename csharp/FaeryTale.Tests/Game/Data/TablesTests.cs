using FaeryTale.Game.Data;

namespace FaeryTale.Tests;

public class TablesTests
{
    [Fact]
    public void DoorsAreSortedByXForTheBinarySearch()
    {
        Door[] doors = Tables.DoorList;

        for (int index = 1; index < doors.Length; index++)
        {
            Assert.True(doors[index - 1].Xc1 <= doors[index].Xc1, $"door at x {doors[index].Xc1} is out of order");
        }
    }

    // The "around village" extent (20) has its top below its bottom in the original data, so no
    // point is ever inside it. The port keeps that as it was.
    private const int INVERTED_EXTENT = 20;

    [Fact]
    public void ExtentsAreProperRectangles()
    {
        for (int number = 0; number < Tables.InitialExtents.Length; number++)
        {
            Extent extent = Tables.InitialExtents[number];

            if (number == INVERTED_EXTENT)
            {
                Assert.True(extent.Y1 > extent.Y2, "the quirk of extent 20 is gone; update this test");
            }
            else
            {
                Assert.True(extent.X1 <= extent.X2 && extent.Y1 <= extent.Y2, $"extent {number}");
            }
        }
    }

    [Fact]
    public void MenuLabelsCoverAllTheEntries()
    {
        // Entries 0-4 are the menu names; every entry after that has a 5 character label.
        for (int number = 0; number < Tables.InitialMenus.Length; number++)
        {
            Menu menu = Tables.InitialMenus[number];

            Assert.True(menu.LabelList.Length >= Math.Max(menu.Num - 5, 0) * 5, $"menu {number}");
            Assert.True(menu.Num <= menu.Enabled.Length, $"menu {number}");
        }
    }

    [Fact]
    public void EveryShortcutKeyPicksARealEntry()
    {
        HashSet<(sbyte, sbyte)> seen = new();

        foreach (Letter shortcut in Tables.MenuShortcuts)
        {
            Assert.True(shortcut.Choice < Tables.InitialMenus[shortcut.Menu].Num, $"key {shortcut.Key}");
            Assert.True(seen.Add((shortcut.Key, shortcut.Menu)), $"key {shortcut.Key} twice");
        }
    }

    [Fact]
    public void SwingTransitionsLeadToSwingStates()
    {
        foreach (sbyte[] next in Tables.SwingTransitions)
        {
            Assert.All(next, to => Assert.InRange(to, 0, 8));
        }
    }

    [Fact]
    public void EveryItemHasANameAndTheTablesHaveTheirDeclaredSizes()
    {
        Assert.Equal(36, Tables.ItemList.Length);
        Assert.All(Tables.ItemList, item => Assert.NotEmpty(item.Name));
        Assert.Equal("Dirk", Tables.ItemList[0].Name);
        Assert.Equal(87, Tables.AnimationFrames.Length);
        Assert.Equal(86, Tables.DoorList.Length);
        Assert.Equal(37, Tables.MenuShortcuts.Length);
    }

    [Fact]
    public void MonstersUseTheEnemySpriteFiles()
    {
        Assert.All(Tables.EncounterChart, monster => Assert.InRange(monster.FileId, 6, 9));
    }

    [Fact]
    public void ABadTableIndexGivesTheFirstEntry()
    {
        Assert.Same(Tables.DoorList[0], Tables.Entry(Tables.DoorList, 999));
    }
}
