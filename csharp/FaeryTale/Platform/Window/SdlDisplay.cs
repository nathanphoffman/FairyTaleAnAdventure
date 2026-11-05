using System.Runtime.InteropServices;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using static SDL2.SDL;

namespace FaeryTale.Platform;

/// The real window, on SDL2: shows the frames, paces the game at 50 Hz and passes keyboard,
/// mouse and gamepad events to the input bridge. Closing the window quits; Escape is passed on to
/// the game (it skips the intro, leaves a game, or opens the game menu). F11 or Alt+Enter switch
/// fullscreen.
public sealed class SdlDisplay : IDisplayDevice, IDisposable
{
    private const string WINDOW_TITLE = "Fairy Tale: An Adventure";
    private const int WINDOW_WIDTH = 1280;
    private const int WINDOW_HEIGHT = 800;
    private const int TICK_MILLISECONDS = 20; // 50 Hz

    /// If the game falls this many ticks behind, pacing restarts instead of catching up.
    private const int MAX_LAG_TICKS = 4;

    /// 640x200 pixels are shown on a 4:3 screen, so each row is drawn two pixels tall.
    private const int ROW_SCALE = FrameComposer.ROW_SCALE;
    private const int BYTES_PER_PIXEL = 4;

    private readonly IntPtr window;
    private readonly IntPtr renderer;
    private readonly uint[] frame = new uint[Scene.FRAME_WIDTH * Scene.FRAME_HEIGHT];
    private readonly bool fast = Environment.GetEnvironmentVariable("FTA_FAST") is not null;
    private readonly PadInput pad = new();
    private Overlay overlay = new();
    private bool cursorShown;
    private DateTime nextTick = DateTime.UtcNow.AddMilliseconds(TICK_MILLISECONDS);

    public SdlDisplay()
    {
        if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMECONTROLLER) != 0)
        {
            throw new InvalidOperationException($"Could not start SDL: {SDL_GetError()}");
        }

        window = SDL_CreateWindow(WINDOW_TITLE, SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, WINDOW_WIDTH, WINDOW_HEIGHT, SDL_WindowFlags.SDL_WINDOW_RESIZABLE);
        renderer = SDL_CreateRenderer(window, -1, SDL_RendererFlags.SDL_RENDERER_ACCELERATED);

        // Without a graphics card (or a real video driver) the software renderer still works.
        if (renderer == IntPtr.Zero)
        {
            renderer = SDL_CreateRenderer(window, -1, SDL_RendererFlags.SDL_RENDERER_SOFTWARE);
        }

        if (window == IntPtr.Zero || renderer == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Could not open the window: {SDL_GetError()}");
        }

        SDL_RenderSetLogicalSize(renderer, Scene.FRAME_WIDTH, Scene.FRAME_HEIGHT * ROW_SCALE);
        SDL_ShowCursor(0); // the game draws its own pointer
        KeyRemap.Load();
    }

    public uint Ticks { get; private set; }

    public void Present(Scene scene)
    {
        Draw(scene);
        HandleEvents();
    }

    public void WaitFrame(Scene scene)
    {
        HandleEvents();
        Ticks++;
        ApplyCursorSetting();
        Pace();
        Draw(scene);
    }

    public void SetOverlay(Overlay overlay) => this.overlay = overlay;

    public void Dispose()
    {
        SDL_DestroyRenderer(renderer);
        SDL_DestroyWindow(window);
    }

    // Composes the scene and the overlay and puts the result on screen.
    private void Draw(Scene scene)
    {
        (uint[] pixels, int height) = FrameComposer.Build(scene, overlay, frame);
        IntPtr texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_ARGB8888, (int)SDL_TextureAccess.SDL_TEXTUREACCESS_STREAMING, Scene.FRAME_WIDTH, height);

        if (texture == IntPtr.Zero)
        {
            Console.Error.WriteLine($"Could not create the screen texture: {SDL_GetError()}");

            return;
        }

        UpdateAndShow(texture, pixels);
        SDL_DestroyTexture(texture);
    }

    private void UpdateAndShow(IntPtr texture, uint[] pixels)
    {
        // The pixels are pinned so SDL can read them where they are.
        GCHandle pinned = GCHandle.Alloc(pixels, GCHandleType.Pinned);

        try
        {
            if (SDL_UpdateTexture(texture, IntPtr.Zero, pinned.AddrOfPinnedObject(), Scene.FRAME_WIDTH * BYTES_PER_PIXEL) != 0)
            {
                Console.Error.WriteLine($"Could not update the screen texture: {SDL_GetError()}");
            }
        }
        finally
        {
            pinned.Free();
        }

        SDL_RenderClear(renderer);
        SDL_RenderCopy(renderer, texture, IntPtr.Zero, IntPtr.Zero);
        SDL_RenderPresent(renderer);
    }

    // Waits for the next 50 Hz tick (the original's vertical blank).
    private void Pace()
    {
        DateTime now = DateTime.UtcNow;
        TimeSpan wait = nextTick > now ? nextTick - now : TimeSpan.Zero;

        nextTick = now > nextTick.AddMilliseconds(TICK_MILLISECONDS * MAX_LAG_TICKS) ? now.AddMilliseconds(TICK_MILLISECONDS) : nextTick.AddMilliseconds(TICK_MILLISECONDS);

        if (!fast && wait > TimeSpan.Zero)
        {
            Thread.Sleep(wait);
        }
    }

    // Shows or hides the ordinary mouse arrow when the game asked for a change.
    private void ApplyCursorSetting()
    {
        if (cursorShown != WindowControls.SystemCursorShown)
        {
            cursorShown = WindowControls.SystemCursorShown;
            SDL_ShowCursor(cursorShown ? 1 : 0);
        }
    }

    private void HandleEvents()
    {
        while (SDL_PollEvent(out SDL_Event windowEvent) != 0)
        {
            HandleEvent(windowEvent);
        }
    }

    private void HandleEvent(SDL_Event windowEvent)
    {
        switch (windowEvent.type)
        {
            case SDL_EventType.SDL_QUIT:
                Environment.Exit(0);
                break;

            case SDL_EventType.SDL_KEYDOWN:
            case SDL_EventType.SDL_KEYUP:
                HandleKey(windowEvent.key);
                break;

            case SDL_EventType.SDL_MOUSEMOTION:
                MouseMovedTo(windowEvent.motion.x, windowEvent.motion.y);
                break;

            case SDL_EventType.SDL_MOUSEBUTTONDOWN:
            case SDL_EventType.SDL_MOUSEBUTTONUP:
                HandleMouseButton(windowEvent.button);
                break;

            default:
                pad.Handle(windowEvent);
                break;
        }
    }

    private void HandleKey(SDL_KeyboardEvent key)
    {
        bool down = key.type == SDL_EventType.SDL_KEYDOWN;

        if (down && IsFullscreenToggle(key.keysym))
        {
            ToggleFullscreen();
        }
        else if (key.repeat == 0)
        {
            InputBridge.KeyEvent(KeyRemap.Apply(SDL_GetKeyName(key.keysym.sym)), down);
        }
    }

    // F11 or Alt+Enter.
    private static bool IsFullscreenToggle(SDL_Keysym key) =>
        key.sym == SDL_Keycode.SDLK_F11 || (key.sym == SDL_Keycode.SDLK_RETURN && (key.mod & SDL_Keymod.KMOD_ALT) != 0);

    private void ToggleFullscreen()
    {
        bool isFullscreen = (SDL_GetWindowFlags(window) & (uint)SDL_WindowFlags.SDL_WINDOW_FULLSCREEN_DESKTOP) != 0;

        if (SDL_SetWindowFullscreen(window, isFullscreen ? 0 : (uint)SDL_WindowFlags.SDL_WINDOW_FULLSCREEN_DESKTOP) != 0)
        {
            Console.Error.WriteLine($"Could not change fullscreen: {SDL_GetError()}");
        }
    }

    private static void HandleMouseButton(SDL_MouseButtonEvent button)
    {
        bool down = button.type == SDL_EventType.SDL_MOUSEBUTTONDOWN;

        if (down)
        {
            MouseMovedTo(button.x, button.y);
        }

        if (button.button == SDL_BUTTON_LEFT || button.button == SDL_BUTTON_RIGHT)
        {
            InputBridge.MouseButton(button.button == SDL_BUTTON_LEFT, down);
        }
    }

    // SDL reports mouse positions in the 640x400 logical screen (it handles the letterboxing), so
    // only the doubled rows need undoing.
    private static void MouseMovedTo(int logicalX, int logicalY) => InputBridge.MouseMove(logicalX, logicalY / ROW_SCALE);
}
