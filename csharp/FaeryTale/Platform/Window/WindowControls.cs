namespace FaeryTale.Platform;

/// Settings the game asks of the window. The real window reads them; a headless run ignores them.
public static class WindowControls
{
    /// Remastered and Reimagined draw the world at the window's own size with the new world drawer
    /// (and let it be zoomed); Original draws the old 640 x 400 picture.
    public static bool NativeRendering { get; set; }

    /// How many pictures a second the new world drawer makes: MONITOR_SYNC (0) makes one for every
    /// refresh of the monitor and waits for it (vsync, the smoothest but it can hold the game back);
    /// a number makes that many (the default is 150, so vsync starts off). The game itself moves 50
    /// times a second; the pictures between its steps draw the camera and the characters part of the
    /// way along their last move.
    public static int FrameRate { get; set; } = DEFAULT_FIXED_RATE;

    public const int MONITOR_SYNC = 0;
    public const int TICKS_PER_SECOND = 50;

    private const int DEFAULT_FIXED_RATE = 150;

    private static readonly int[] FIXED_RATES = [50, 100, 150, 200, 250, 300];

    // The fixed rate to go back to when vsync is turned off again.
    private static int lastFixedRate = DEFAULT_FIXED_RATE;

    /// Whether pictures wait for the monitor's refresh (vsync).
    public static bool VSync => FrameRate == MONITOR_SYNC;

    /// "On" or "Off" for the menu.
    public static string VSyncText => VSync ? "On" : "Off";

    /// The text of the frame rate for the menu.
    public static string FrameRateText => VSync ? "Monitor" : $"{FrameRate} fps";

    /// Turns vsync on, or off (back to the last fixed rate).
    public static void ToggleVSync()
    {
        if (VSync)
        {
            FrameRate = lastFixedRate;
        }
        else
        {
            lastFixedRate = FrameRate;
            FrameRate = MONITOR_SYNC;
        }
    }

    /// The next fixed rate, going round from the highest to the lowest. With vsync on the monitor
    /// sets the rate, so nothing changes.
    public static void CycleFrameRate()
    {
        if (!VSync)
        {
            FrameRate = FIXED_RATES[(Array.IndexOf(FIXED_RATES, FrameRate) + 1) % FIXED_RATES.Length];
        }
    }

    /// Shows or hides the ordinary mouse arrow (the game draws its own while playing).
    public static bool SystemCursorShown { get; set; }
}
