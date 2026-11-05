namespace FaeryTale.Platform;

/// Settings the game asks of the window. The real window reads them; a headless run ignores them.
public static class WindowControls
{
    /// Shows or hides the ordinary mouse arrow (the game draws its own while playing).
    public static bool SystemCursorShown { get; set; }
}
