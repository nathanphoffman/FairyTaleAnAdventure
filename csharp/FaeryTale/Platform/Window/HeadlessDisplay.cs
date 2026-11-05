using FaeryTale.Game;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Testing;

namespace FaeryTale.Platform;

/// A display without a window, for the regression tests. It counts ticks, runs the scripts the
/// test environment variables describe (FTA_INJECT, FTA_CLICK, FTA_CMD) and saves a screenshot
/// (FTA_SHOT after FTA_SHOT_AFTER ticks, then exits). FTA_FAST=1 skips the 50 Hz pacing; the
/// game logic only counts ticks, not time.
public sealed class HeadlessDisplay : IDisplayDevice
{
    private const int TICK_MILLISECONDS = 20; // 50 Hz

    /// Used when FTA_SHOT_AFTER is not set.
    private const uint DEFAULT_AFTER_TICKS = 1;

    /// A scripted click releases the button this many ticks after pressing it.
    private const uint CLICK_HOLD_TICKS = 3;

    /// Lores pixels are doubled horizontally in the 640 wide frame.
    private const int LORES_TO_FRAME = 2;

    /// Scripted Space press that skips the intro: (press tick, release tick).
    private const uint SKIP_INTRO_PRESS = 5;
    private const uint SKIP_INTRO_RELEASE = 9;

    private readonly uint[] frame = new uint[Scene.FRAME_WIDTH * Scene.FRAME_HEIGHT];
    private readonly List<KeyEvent> keys;
    private readonly List<Click> clicks = TestScript.ClicksFromEnvironment();
    private readonly List<TimedCommand> commands = TestScript.CommandsFromEnvironment();
    private readonly bool fast = Environment.GetEnvironmentVariable("FTA_FAST") is not null;
    private readonly DateTime started = DateTime.UtcNow;
    private Overlay overlay = new();
    private (uint[] Pixels, int Height) shown;
    private DateTime nextTick = DateTime.UtcNow.AddMilliseconds(TICK_MILLISECONDS);

    public HeadlessDisplay(bool launchSkipped)
    {
        keys = TestScript.KeysFromEnvironment();

        if (launchSkipped)
        {
            keys.Add(new KeyEvent(SKIP_INTRO_PRESS, "Space", true));
            keys.Add(new KeyEvent(SKIP_INTRO_RELEASE, "Space", false));
        }
    }

    public uint Ticks { get; private set; }

    public void Present(Scene scene)
    {
        Draw(scene);
        TestHooks();
    }

    public void WaitFrame(Scene scene)
    {
        TestHooks();
        Ticks++;
        RunScript();
        Pace();
        Draw(scene);
    }

    public void SetOverlay(Overlay overlay) => this.overlay = overlay;

    // Turns the scene into the pixels on screen, with the overlay on top.
    private void Draw(Scene scene) => shown = FrameComposer.Build(scene, overlay, frame);

    // Waits for the next 50 Hz tick (the original's vertical blank).
    private void Pace()
    {
        DateTime now = DateTime.UtcNow;
        TimeSpan wait = nextTick > now ? nextTick - now : TimeSpan.Zero;

        nextTick = now > nextTick.AddMilliseconds(TICK_MILLISECONDS * 4) ? now.AddMilliseconds(TICK_MILLISECONDS) : nextTick.AddMilliseconds(TICK_MILLISECONDS);

        if (!fast && wait > TimeSpan.Zero)
        {
            Thread.Sleep(wait);
        }
    }

    // Runs everything the test environment variables scheduled for the current tick: commands,
    // then clicks, then keys.
    private void RunScript()
    {
        foreach (TimedCommand timed in commands.Where(timed => timed.Tick == Ticks))
        {
            CommandQueue.Queue(timed.Command);
        }

        foreach (Click click in clicks)
        {
            if (click.Tick == Ticks)
            {
                InputBridge.MouseMove(click.X * LORES_TO_FRAME, click.Y);
                InputBridge.MouseButton(true, true);
            }
            else if (click.Tick + CLICK_HOLD_TICKS == Ticks)
            {
                InputBridge.MouseButton(true, false);
            }
        }

        foreach (KeyEvent key in keys.Where(key => key.Tick == Ticks))
        {
            InputBridge.KeyEvent(key.Name, key.Down);
        }
    }

    // FTA_SHOT=file.ppm saves the screen after FTA_SHOT_AFTER ticks and quits; FTA_SECONDS=n
    // quits after n seconds.
    private void TestHooks()
    {
        string? path = Environment.GetEnvironmentVariable("FTA_SHOT");

        if (path is not null && Ticks >= TicksFromEnvironment("FTA_SHOT_AFTER"))
        {
            (uint[] pixels, int height) = shown;

            try
            {
                File.WriteAllBytes(path, PpmWriter.FrameAsPpm(pixels, Scene.FRAME_WIDTH, height));
            }
            catch (IOException error)
            {
                Console.Error.WriteLine($"Could not save the screenshot: {error.Message}");
            }

            Environment.Exit(0);
        }

        if (uint.TryParse(Environment.GetEnvironmentVariable("FTA_SECONDS"), out uint seconds) && (DateTime.UtcNow - started).TotalSeconds >= seconds)
        {
            Environment.Exit(0);
        }
    }

    private static uint TicksFromEnvironment(string name) =>
        uint.TryParse(Environment.GetEnvironmentVariable(name), out uint ticks) ? ticks : DEFAULT_AFTER_TICKS;
}
