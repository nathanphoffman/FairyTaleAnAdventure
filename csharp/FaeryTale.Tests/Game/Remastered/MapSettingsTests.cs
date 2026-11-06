using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

public class MapSettingsTests
{
    [Fact]
    public void TheMapsAreOnUntilTurnedOffAndTheMenuSaysWhatClickingDoes()
    {
        Assert.True(MapSettings.Enabled);
        Assert.Equal("Disable Map", MapSettings.MenuText);

        MapSettings.Toggle();

        Assert.False(MapSettings.Enabled);
        Assert.Equal("Enable Map", MapSettings.MenuText);

        MapSettings.Toggle();

        Assert.True(MapSettings.Enabled);
    }
}
