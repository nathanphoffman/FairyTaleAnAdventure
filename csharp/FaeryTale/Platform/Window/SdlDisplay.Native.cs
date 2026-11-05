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
    private readonly bool logFrameRate = Environment.GetEnvironmentVariable("FTA_FPS_LOG") is not null;
    private int framesShown;
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

    // One game tick's time: FramesPerTick pictures, the last one the present state of the game. The
    // ones before it are drawn with the movement of the last tick only part way done. Waiting for the
    // moment of each picture keeps them evenly spaced.
    private void WaitFrameNative(Scene scene)
    {
        (int width, int height) = NativePresenter.OutputSize(renderer);
        Video? video = Source;
        bool world = video?.Capture is { Enabled: true, WorldVisible: true };
        bool fresh = world && video!.Capture.Tick != lastCaptureTick;
        int frames = world ? WindowControls.FramesPerTick : 1;

        lastCaptureTick = video?.Capture.Tick ?? lastCaptureTick;

        for (int frame = 1; frame <= frames; frame++)
        {
            double alpha = fresh ? frame / (double)frames : 1.0;
            NativeComposer composer = nativePresenter.Composer;

            nativePresenter.Show(renderer, composer.Compose(width, height, scene, overlay, video, alpha, composer.Pointer), width, height);
            LogFrameRate(width, height);
            WaitUntil(slotStartMs + frame * TICK_MS / frames);

            if (frame < frames)
            {
                HandleEvents();
            }
        }

        AdvanceSlot();
    }

    // FTA_FPS_LOG=1 prints how many pictures a second were shown (to see if the machine keeps up).
    private void LogFrameRate(int width, int height)
    {
        framesShown++;

        double now = clock.Elapsed.TotalMilliseconds;

        if (logFrameRate && now - lastLogMs >= 1000)
        {
            Console.Error.WriteLine($"{framesShown} pictures a second at {width}x{height}, zoom {WorldView.Current.Zoom:F2}");
            framesShown = 0;
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

    // FTA_FPS=100 starts with a picture rate of 100 a second (rounded to a whole number of pictures per tick).
    private static void ReadFrameRateSetting()
    {
        if (int.TryParse(Environment.GetEnvironmentVariable("FTA_FPS"), out int rate))
        {
            WindowControls.FramesPerTick = Math.Clamp((int)Math.Round(rate / (double)WindowControls.TICKS_PER_SECOND), WindowControls.MIN_FRAMES_PER_TICK, WindowControls.MAX_FRAMES_PER_TICK);
        }
    }
}
