namespace FaeryTale.Platform;

/// Settings the game asks of the window. The real window reads them; a headless run ignores them.
public static class WindowControls
{
    /// While this is on, Escape reaches the game as a key (to skip the intro); otherwise it quits.
    public static bool EscapeSkips { get; set; }

    /// Shows or hides the ordinary mouse arrow (the game draws its own while playing).
    public static bool SystemCursorShown { get; set; }
}
