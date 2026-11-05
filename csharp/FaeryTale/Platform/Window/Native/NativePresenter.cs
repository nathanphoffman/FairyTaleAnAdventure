using System.Runtime.InteropServices;
using static SDL2.SDL;

namespace FaeryTale.Platform;

/// Puts pictures of the window's own size on the screen (the SDL side of the native size drawing):
/// keeps one texture the size of the window and uploads each new picture to it.
internal sealed class NativePresenter : IDisposable
{
    private const int BYTES_PER_PIXEL = 4;

    private IntPtr texture;
    private int textureWidth;
    private int textureHeight;

    /// Makes the pictures and knows where the panel and the pointer are in them.
    public NativeComposer Composer { get; } = new();

    /// The size of the window's drawing area in pixels (not in screen points: on a screen with a
    /// scaling the two differ).
    public static (int Width, int Height) OutputSize(IntPtr renderer)
    {
        SDL_GetRendererOutputSize(renderer, out int width, out int height);

        return (Math.Max(width, 1), Math.Max(height, 1));
    }

    /// Shows a picture, filling the window.
    public void Show(IntPtr renderer, uint[] pixels, int width, int height)
    {
        if (texture == IntPtr.Zero || textureWidth != width || textureHeight != height)
        {
            MakeTexture(renderer, width, height);
        }

        if (texture == IntPtr.Zero)
        {
            return;
        }

        // The pixels are pinned so SDL can read them where they are.
        GCHandle pinned = GCHandle.Alloc(pixels, GCHandleType.Pinned);

        try
        {
            SDL_UpdateTexture(texture, IntPtr.Zero, pinned.AddrOfPinnedObject(), width * BYTES_PER_PIXEL);
        }
        finally
        {
            pinned.Free();
        }

        SDL_RenderClear(renderer);
        SDL_RenderCopy(renderer, texture, IntPtr.Zero, IntPtr.Zero);
        SDL_RenderPresent(renderer);
    }

    public void Dispose() => Release();

    /// Lets go of the texture (it belongs to a renderer that is about to be replaced).
    public void Release()
    {
        if (texture != IntPtr.Zero)
        {
            SDL_DestroyTexture(texture);
            texture = IntPtr.Zero;
        }
    }

    private void MakeTexture(IntPtr renderer, int width, int height)
    {
        if (texture != IntPtr.Zero)
        {
            SDL_DestroyTexture(texture);
        }

        texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_ARGB8888, (int)SDL_TextureAccess.SDL_TEXTUREACCESS_STREAMING, width, height);
        textureWidth = width;
        textureHeight = height;

        if (texture == IntPtr.Zero)
        {
            Console.Error.WriteLine($"Could not create the screen texture: {SDL_GetError()}");
        }
    }
}
