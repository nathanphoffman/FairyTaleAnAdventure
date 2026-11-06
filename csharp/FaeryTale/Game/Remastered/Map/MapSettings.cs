namespace FaeryTale.Game.Remastered;

/// Whether the minimap and the big map (M) are on. They are on until the player turns them off in
/// the game menu (for those who want the old game's way of finding the way); kept for the session.
public static class MapSettings
{
    public static bool Enabled { get; set; } = true;

    /// The menu entry's words: what clicking it does.
    public static string MenuText => Enabled ? "Disable Map" : "Enable Map";

    /// Turns the maps off if they are on, and on if they are off.
    public static void Toggle() => Enabled = !Enabled;
}
