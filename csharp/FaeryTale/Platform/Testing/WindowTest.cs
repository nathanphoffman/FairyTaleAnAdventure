using static SDL2.SDL;

namespace FaeryTale.Testing;

/// Opens a window and draws grey stripes for a moment, to check that SDL works (--window-test).
public static class WindowTest
{
    private const int WINDOW_WIDTH = 640;
    private const int WINDOW_HEIGHT = 400;
    private const int SHOW_MILLISECONDS = 1500;
    private const int STRIPE_COUNT = 8;

    public static int Run()
    {
        if (SDL_Init(SDL_INIT_VIDEO) != 0)
        {
            Console.Error.WriteLine($"SDL_Init failed: {SDL_GetError()}");

            return 1;
        }

        IntPtr window = SDL_CreateWindow("Fairy Tale: An Adventure (C#)",
            SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, WINDOW_WIDTH, WINDOW_HEIGHT, SDL_WindowFlags.SDL_WINDOW_SHOWN);
        IntPtr renderer = SDL_CreateRenderer(window, -1, SDL_RendererFlags.SDL_RENDERER_SOFTWARE);

        if (window == IntPtr.Zero || renderer == IntPtr.Zero)
        {
            Console.Error.WriteLine($"Window setup failed: {SDL_GetError()}");

            return 1;
        }

        DrawStripes(renderer);
        SDL_Delay(SHOW_MILLISECONDS);

        SDL_DestroyRenderer(renderer);
        SDL_DestroyWindow(window);
        SDL_Quit();

        return 0;
    }

    // Vertical stripes of growing brightness, so a wrong colour or size is easy to spot.
    private static void DrawStripes(IntPtr renderer)
    {
        int stripeWidth = WINDOW_WIDTH / STRIPE_COUNT;

        for (int stripe = 0; stripe < STRIPE_COUNT; stripe++)
        {
            byte brightness = (byte)(stripe * 255 / (STRIPE_COUNT - 1));
            SDL_SetRenderDrawColor(renderer, brightness, brightness, brightness, 255);

            SDL_Rect area = new() { x = stripe * stripeWidth, y = 0, w = stripeWidth, h = WINDOW_HEIGHT };
            SDL_RenderFillRect(renderer, ref area);
        }

        SDL_RenderPresent(renderer);
    }
}
