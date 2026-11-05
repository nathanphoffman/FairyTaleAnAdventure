namespace FaeryTale.Platform;

/// Settings the game asks of the window. The real window reads them; a headless run ignores them.
public static class WindowControls
{
    /// Remastered and Reimagined draw the world at the window's own size with the new world drawer
    /// (and let it be zoomed); Original draws the old 640 x 400 picture.
    public static bool NativeRendering { get; set; }

    /// How many pictures are drawn for each game tick (the game moves 50 times a second) when the
    /// new world drawer is used: 1 is 50 pictures a second, 3 is 150. The ones between ticks show the
    /// camera and the characters part of the way along, which makes the movement smooth.
    public static int FramesPerTick { get; set; } = DEFAULT_FRAMES_PER_TICK;

    public const int MIN_FRAMES_PER_TICK = 1;
    public const int MAX_FRAMES_PER_TICK = 6;
    private const int DEFAULT_FRAMES_PER_TICK = 3;
    public const int TICKS_PER_SECOND = 50;

    /// The picture rate that goes with FramesPerTick.
    public static int FramesPerSecond => FramesPerTick * TICKS_PER_SECOND;

    /// The next picture rate up, going round from the highest to the lowest.
    public static void CycleFrameRate() =>
        FramesPerTick = FramesPerTick >= MAX_FRAMES_PER_TICK ? MIN_FRAMES_PER_TICK : FramesPerTick + 1;

    /// Shows or hides the ordinary mouse arrow (the game draws its own while playing).
    public static bool SystemCursorShown { get; set; }
}
