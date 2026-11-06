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

    /// A choice for the size of the window: a size, and whether it fills the monitor with no border
    /// (the monitor's own size) or is a window.
    public readonly record struct Resolution(int Width, int Height, bool Borderless)
    {
        /// The words in the menu, such as "1920x1080 (borderless)" or "800x600 (windowed)".
        public string Text => $"{Width}x{Height} ({(Borderless ? "borderless" : "windowed")})";
    }

    // The sizes offered besides the monitor's own, largest first; the smallest is the lowest choice.
    private static readonly (int Width, int Height)[] WINDOW_SIZES =
    [
        (3840, 2160), (2560, 1440), (1920, 1080), (1600, 900), (1366, 768), (1280, 720), (1024, 768), (800, 600), (640, 480),
    ];

    private static readonly object resolutionPadlock = new();
    private static Resolution? requested;

    /// The monitor's own size (the real window reads it; until then a common one, for tests).
    public static (int Width, int Height) Monitor { get; set; } = (1920, 1080);

    /// What the window is now: its size and whether it is the borderless full screen.
    public static (int Width, int Height, bool Fullscreen) WindowState { get; set; } = (1920, 1080, true);

    /// The choices for the window, from the monitor's own size (borderless) down to 640x480 (windowed),
    /// in that order. Only sizes that fit on the monitor are offered.
    public static List<Resolution> ResolutionChoices()
    {
        List<Resolution> choices = [new Resolution(Monitor.Width, Monitor.Height, true)];

        foreach ((int width, int height) in WINDOW_SIZES)
        {
            bool fits = width <= Monitor.Width && height <= Monitor.Height;
            bool isMonitorSize = width == Monitor.Width && height == Monitor.Height;

            if (fits && !isMonitorSize)
            {
                choices.Add(new Resolution(width, height, false));
            }
        }

        return choices;
    }

    /// The words for the window as it is now, for the game menu.
    public static string ResolutionText =>
        WindowState.Fullscreen ? new Resolution(Monitor.Width, Monitor.Height, true).Text : new Resolution(WindowState.Width, WindowState.Height, false).Text;

    /// Asks the window to take a size (the window carries it out on its own thread).
    public static void RequestResolution(Resolution choice)
    {
        lock (resolutionPadlock)
        {
            requested = choice;
        }
    }

    /// The size asked for since the last call, if any.
    public static Resolution? TakeResolutionRequest()
    {
        lock (resolutionPadlock)
        {
            Resolution? asked = requested;

            requested = null;

            return asked;
        }
    }

    /// Shows or hides the ordinary mouse arrow (the game draws its own while playing).
    public static bool SystemCursorShown { get; set; }
}
