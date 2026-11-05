namespace FaeryTale.Platform;

/// The keys of Remastered and Reimagined. WASD walks like the arrow keys, so the three menu
/// shortcuts that used those letters move to the nearest free letters.
///
/// Letters are global shortcuts in the game, each one picking one menu entry, so the new letters
/// must be ones nothing else uses.
public static class RemasteredKeys
{
    // What a key press turns into. W, A, S, D become the arrow keys; the old shortcut letters of
    // Say (S), Ask (A) and the sword in the Buy menu (W) are given to H (hello), N (the other
    // way to ask, "N" for "name") and Z.
    private static readonly Dictionary<string, string> TRANSLATION = new()
    {
        ["W"] = "Up",
        ["A"] = "Left",
        ["S"] = "Down",
        ["D"] = "Right",
        ["H"] = "S",
        ["N"] = "A",
        ["Z"] = "W",
    };

    /// The key name the game should see for a key that was pressed (names are SDL's).
    public static string Translate(string name) =>
        TRANSLATION.GetValueOrDefault(name.Length == 1 ? name.ToUpperInvariant() : name, name);
}
