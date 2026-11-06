using FaeryTale.Game;
using FaeryTale.Game.Remastered;
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
    private const int DEFAULT_NATIVE_WIDTH = 1280;
    private const int DEFAULT_NATIVE_HEIGHT = 720;
    private const uint SKIP_INTRO_PRESS = 5;
    private const uint SKIP_INTRO_RELEASE = 9;

    private readonly uint[] frame = new uint[Scene.FRAME_WIDTH * Scene.FRAME_HEIGHT];
    private readonly List<KeyEvent> keys;
    private readonly List<Click> clicks = TestScript.ClicksFromEnvironment();
    private readonly List<TimedCommand> commands = TestScript.CommandsFromEnvironment();
    private readonly bool fast = Environment.GetEnvironmentVariable("FTA_FAST") is not null;
    private readonly DateTime started = DateTime.UtcNow;
    private Overlay overlay = new();
    private (uint[] Pixels, int Width, int Height) shown;
    private readonly NativeComposer composer = new();
    private readonly (int Width, int Height) nativeSize = NativeSizeFromEnvironment();
    private bool zoomApplied;
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

    public Video? Source { get; set; }

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

    public double AnimationFrameMs => TICK_MILLISECONDS;

    public void ShowAnimationFrame(Scene scene) => WaitFrame(scene);

    // Turns the scene into the pixels on screen, with the overlay on top.
    private void Draw(Scene scene)
    {
        if (WindowControls.NativeRendering)
        {
            // Remastered and Reimagined: the picture of a window of FTA_NATIVE size (default 1280x720),
            // drawn by the new world drawer, with the zoom of FTA_ZOOM.
            ApplyZoomFromEnvironment();
            shown = (composer.Compose(nativeSize.Width, nativeSize.Height, scene, overlay, Source, 1.0, composer.Pointer), nativeSize.Width, nativeSize.Height);

            return;
        }

        (uint[] pixels, int rows) = FrameComposer.Build(scene, overlay, frame);

        shown = (pixels, Scene.FRAME_WIDTH, rows);
    }

    // FTA_NATIVE=1920x1080 sets the size of the native picture.
    private static (int Width, int Height) NativeSizeFromEnvironment()
    {
        string[] parts = (Environment.GetEnvironmentVariable("FTA_NATIVE") ?? "").Split('x');

        return parts.Length == 2 && int.TryParse(parts[0], out int width) && int.TryParse(parts[1], out int height) ? (width, height) : (DEFAULT_NATIVE_WIDTH, DEFAULT_NATIVE_HEIGHT);
    }

    // FTA_ZOOM=0.5 sets the zoom (once).
    private void ApplyZoomFromEnvironment()
    {
        if (!zoomApplied && double.TryParse(Environment.GetEnvironmentVariable("FTA_ZOOM"), System.Globalization.CultureInfo.InvariantCulture, out double zoom))
        {
            WorldView.Current.SetZoom(zoom);
        }

        zoomApplied = true;
    }

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
                MoveMouseTo(click);
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

    // A scripted click is at a place of the old 320 x 200 picture: in the native picture that is the
    // same place on the 640 x 400 grid, scaled.
    private void MoveMouseTo(Click click)
    {
        if (!WindowControls.NativeRendering)
        {
            InputBridge.MouseMove(click.X * LORES_TO_FRAME, click.Y);

            return;
        }

        composer.RouteMouse(composer.GridLeft + click.X * LORES_TO_FRAME * composer.Scale, composer.GridTop + click.Y * 2 * composer.Scale);
    }

    // FTA_SHOT=file.ppm saves the screen after FTA_SHOT_AFTER ticks and quits; FTA_SECONDS=n
    // quits after n seconds.
    private void TestHooks()
    {
        string? path = Environment.GetEnvironmentVariable("FTA_SHOT");

        if (path is not null && Ticks >= TicksFromEnvironment("FTA_SHOT_AFTER"))
        {
            (uint[] pixels, int width, int height) = shown;

            try
            {
                File.WriteAllBytes(path, PpmWriter.FrameAsPpm(pixels, width, height));
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
