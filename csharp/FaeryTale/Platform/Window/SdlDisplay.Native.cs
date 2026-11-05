using System.Diagnostics;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using static SDL2.SDL;

namespace FaeryTale.Platform;

/// The part of the window that draws Remastered and Reimagined: pictures of the window's own size
/// from the new world drawer, several for every game tick (the ones between ticks show the movement
/// part of the way along), borderless fullscreen, and the zoom.
public sealed partial class SdlDisplay
{
    private const double TICK_MS = 1000.0 / WindowControls.TICKS_PER_SECOND;

    /// Pacing restarts if the game falls this many ticks behind.
    private const double MAX_LAG_MS = TICK_MS * MAX_LAG_TICKS;

    /// The last milliseconds of a wait are spent looking at the clock, because sleeping is not exact.
    private const double SPIN_MS = 1.0;

    private readonly NativePresenter nativePresenter = new();
    private readonly Stopwatch clock = Stopwatch.StartNew();
    private bool nativeApplied;
    private int lastCaptureTick = -1;
    private double slotStartMs;
    private bool vsyncActive;
    private double nextPictureMs;
    private double lastFlipMs;

    /// What a monitor is taken to be when its refresh rate cannot be asked.
    private const double DEFAULT_REFRESH_HZ = 60;
    private const double MIN_PICTURE_SPACING_MS = 4.0;
    private readonly bool logFrameRate = Environment.GetEnvironmentVariable("FTA_FPS_LOG") is not null;
    private int framesShown;
    private int ticksCounted;
    private double lastLogMs;

    // Switches the window between the old 640 x 400 picture and the native size when the game asks:
    // the native size goes borderless fullscreen with the fullscreen zoom.
    private void ApplyNativeMode()
    {
        if (WindowControls.NativeRendering == nativeApplied)
        {
            return;
        }

        nativeApplied = WindowControls.NativeRendering;

        if (nativeApplied)
        {
            SDL_RenderSetLogicalSize(renderer, 0, 0);
            SetFullscreen(true);
            WorldView.Current.SetZoom(WorldView.FULLSCREEN_ZOOM);
            slotStartMs = clock.Elapsed.TotalMilliseconds;
        }
        else
        {
            SDL_RenderSetLogicalSize(renderer, Scene.FRAME_WIDTH, Scene.FRAME_HEIGHT * ROW_SCALE);
        }
    }

    // Borderless fullscreen at the monitor's own size, or back to the window.
    private void SetFullscreen(bool on)
    {
        if (SDL_SetWindowFullscreen(window, on ? (uint)SDL_WindowFlags.SDL_WINDOW_FULLSCREEN_DESKTOP : 0) != 0)
        {
            Console.Error.WriteLine($"Could not change fullscreen: {SDL_GetError()}");
        }
    }

    // Makes the SDL renderer; with vsync every picture waits for the monitor's next refresh (without
    // a graphics card or a real video driver the software renderer still works).
    private IntPtr CreateRenderer(bool vsync)
    {
        SDL_RendererFlags flags = SDL_RendererFlags.SDL_RENDERER_ACCELERATED | (vsync ? SDL_RendererFlags.SDL_RENDERER_PRESENTVSYNC : 0);
        IntPtr made = SDL_CreateRenderer(window, -1, flags);

        return made != IntPtr.Zero ? made : SDL_CreateRenderer(window, -1, SDL_RendererFlags.SDL_RENDERER_SOFTWARE);
    }

    // The renderer's vsync can only be chosen when it is made, so a change of the frame rate setting
    // between the monitor's own rate and a fixed one makes a new renderer.
    private void EnsureVSync(bool wanted)
    {
        if (wanted == vsyncActive)
        {
            return;
        }

        vsyncActive = wanted;
        nativePresenter.Release();
        SDL_DestroyRenderer(renderer);
        renderer = CreateRenderer(wanted);
        SDL_RenderSetLogicalSize(renderer, 0, 0);
    }

    // One game tick's time, shown as many pictures as the monitor (or the chosen rate) allows. Each
    // picture draws the camera and the characters where they will be at the moment it is shown: part
    // of the way from where the last tick had them to where this tick has them, worked out from the
    // clock. That keeps the movement even whatever the picture rate is, and when the machine cannot
    // keep up there are simply fewer pictures, never a slower game.
    private void WaitFrameNative(Scene scene)
    {
        bool monitorSync = WindowControls.FrameRate == WindowControls.MONITOR_SYNC;

        ticksCounted++;

        EnsureVSync(monitorSync);

        (int width, int height) = NativePresenter.OutputSize(renderer);
        Video? video = Source;
        bool world = video?.Capture is { Enabled: true, WorldVisible: true };
        bool fresh = world && video!.Capture.Tick != lastCaptureTick;
        double tickEnd = slotStartMs + TICK_MS;
        double interval = monitorSync ? RefreshIntervalMs() : 1000.0 / WindowControls.FrameRate;

        lastCaptureTick = video?.Capture.Tick ?? lastCaptureTick;

        if (!world)
        {
            ShowPicture(scene, video, width, height, 1.0);
            WaitUntil(tickEnd);
            AdvanceSlot();

            return;
        }

        // Pictures keep to their own rhythm from one game step to the next.
        double scheduled = Math.Max(nextPictureMs, clock.Elapsed.TotalMilliseconds);

        do
        {
            // When this picture will be on the screen: the next refresh of the monitor, or the moment
            // it is scheduled for.
            double shownAt = monitorSync ? lastFlipMs + interval : scheduled;
            double alpha = fresh ? Math.Clamp((shownAt - slotStartMs) / TICK_MS, 0.0, 1.0) : 1.0;

            if (!monitorSync)
            {
                WaitUntil(scheduled);
            }

            ShowPicture(scene, video, width, height, alpha);
            lastFlipMs = clock.Elapsed.TotalMilliseconds;

            // Without a real vsync (a software renderer) pictures would otherwise be made flat out.
            if (monitorSync)
            {
                WaitUntil(lastFlipMs + MIN_PICTURE_SPACING_MS);
            }

            scheduled = Math.Max(scheduled + interval, clock.Elapsed.TotalMilliseconds);
            HandleEvents();
        }
        while ((monitorSync ? lastFlipMs + interval : scheduled) < tickEnd);

        // The next game step starts at the end of this one's time, not earlier: its movement is drawn
        // from that moment on.
        nextPictureMs = scheduled;
        WaitUntil(tickEnd);
        AdvanceSlot();
    }

    // Makes and shows one picture.
    private void ShowPicture(Scene scene, Video? video, int width, int height, double alpha)
    {
        NativeComposer composer = nativePresenter.Composer;

        nativePresenter.Show(renderer, composer.Compose(width, height, scene, overlay, video, alpha, composer.Pointer), width, height);
        LogFrameRate(width, height);
    }

    // How long a refresh of the monitor the window is on takes (60 Hz if it cannot be asked).
    private double RefreshIntervalMs()
    {
        int display = Math.Max(SDL_GetWindowDisplayIndex(window), 0);

        if (SDL_GetCurrentDisplayMode(display, out SDL_DisplayMode mode) == 0 && mode.refresh_rate > 0)
        {
            return 1000.0 / mode.refresh_rate;
        }

        return 1000.0 / DEFAULT_REFRESH_HZ;
    }

    // FTA_FPS_LOG=1 prints how many pictures a second were shown (to see if the machine keeps up).
    private void LogFrameRate(int width, int height)
    {
        framesShown++;

        double now = clock.Elapsed.TotalMilliseconds;

        if (logFrameRate && now - lastLogMs >= 1000)
        {
            Console.Error.WriteLine($"{framesShown} pictures and {ticksCounted} game steps a second at {width}x{height}, zoom {WorldView.Current.Zoom:F2}");
            framesShown = 0;
            ticksCounted = 0;
            lastLogMs = now;
        }
    }

    private void WaitUntil(double milliseconds)
    {
        if (fast)
        {
            return;
        }

        double left = milliseconds - clock.Elapsed.TotalMilliseconds;

        if (left > SPIN_MS)
        {
            Thread.Sleep((int)(left - SPIN_MS));
        }

        while (clock.Elapsed.TotalMilliseconds < milliseconds)
        {
            Thread.SpinWait(50);
        }
    }

    // The next tick's time starts where this one's ended, unless the game has fallen far behind.
    private void AdvanceSlot()
    {
        slotStartMs += TICK_MS;

        double now = clock.Elapsed.TotalMilliseconds;

        if (now - slotStartMs > MAX_LAG_MS)
        {
            slotStartMs = now;
        }
    }

    // A window point to a pixel of the drawing area (the two differ on a screen with a scaling).
    private (int X, int Y) ToOutputPixels(int x, int y)
    {
        SDL_GetWindowSize(window, out int windowWidth, out int windowHeight);
        (int outputWidth, int outputHeight) = NativePresenter.OutputSize(renderer);

        return (x * outputWidth / Math.Max(windowWidth, 1), y * outputHeight / Math.Max(windowHeight, 1));
    }

    // The mouse wheel zooms the world in (up) and out (down).
    private static void ZoomWithWheel(int steps)
    {
        if (WindowControls.NativeRendering && steps != 0)
        {
            WorldView.Current.ZoomBySteps(Math.Sign(steps));
        }
    }

    // + and - zoom in and out (keypad too). Returns true when the key was used.
    private static bool ZoomWithKey(string name)
    {
        if (!WindowControls.NativeRendering)
        {
            return false;
        }

        switch (name)
        {
            case "=":
            case "+":
            case "Keypad +":
                WorldView.Current.ZoomBySteps(1);

                return true;

            case "-":
            case "Keypad -":
                WorldView.Current.ZoomBySteps(-1);

                return true;

            default:
                return false;
        }
    }

    // FTA_FPS=100 starts with 100 pictures a second; FTA_FPS=monitor (the default) waits for the monitor.
    private static void ReadFrameRateSetting()
    {
        string? setting = Environment.GetEnvironmentVariable("FTA_FPS");

        if (int.TryParse(setting, out int rate) && rate > 0)
        {
            WindowControls.FrameRate = rate;
        }
        else if (setting is not null && setting.Equals("monitor", StringComparison.OrdinalIgnoreCase))
        {
            WindowControls.FrameRate = WindowControls.MONITOR_SYNC;
        }
    }
}
