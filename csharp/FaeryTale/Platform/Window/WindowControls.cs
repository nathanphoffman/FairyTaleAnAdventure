namespace FaeryTale.Platform;

/// Settings the game asks of the window. The real window reads them; a headless run ignores them.
public static class WindowControls
{
    /// Remastered and Reimagined draw the world at the window's own size with the new world drawer
    /// (and let it be zoomed); Original draws the old 640 x 400 picture.
    public static bool NativeRendering { get; set; }

    /// How many pictures a second the new world drawer makes: MONITOR_SYNC (0) makes one for every
    /// refresh of the monitor and waits for it (the smoothest, and the default); a number makes that
    /// many. The game itself moves 50 times a second; the pictures between its steps draw the camera
    /// and the characters part of the way along their last move.
    public static int FrameRate { get; set; } = MONITOR_SYNC;

    public const int MONITOR_SYNC = 0;
    public const int TICKS_PER_SECOND = 50;

    private static readonly int[] FRAME_RATES = [MONITOR_SYNC, 50, 100, 150, 200, 250, 300];

    /// The text of the frame rate for the menu.
    public static string FrameRateText => FrameRate == MONITOR_SYNC ? "Monitor" : $"{FrameRate} fps";

    /// The next rate in the list, going round from the highest to the first (the monitor's own).
    public static void CycleFrameRate() =>
        FrameRate = FRAME_RATES[(Array.IndexOf(FRAME_RATES, FrameRate) + 1) % FRAME_RATES.Length];

    /// Shows or hides the ordinary mouse arrow (the game draws its own while playing).
    public static bool SystemCursorShown { get; set; }
}
