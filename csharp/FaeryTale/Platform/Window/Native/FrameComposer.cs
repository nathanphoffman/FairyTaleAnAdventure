using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// Turns a scene and the overlay into the pixels to show, for the window and for the screenshots.
internal static class FrameComposer
{
    /// The overlay works on a grid with the rows doubled, so it can draw text with square pixels.
    public const int ROW_SCALE = 2;

    /// The pixels to show and how many rows tall they are. Without an overlay the frame itself is
    /// shown; with one, the picture's rows are repeated to match its grid and the overlay is drawn
    /// on top (the pointer goes under it, or over it for menus that ask for that).
    public static (uint[] Pixels, int Height) Build(Scene scene, Overlay overlay, uint[] frame)
    {
        if (overlay.IsEmpty)
        {
            scene.Compose(frame);

            return (frame, Scene.FRAME_HEIGHT);
        }

        if (overlay.PointerOnTop || overlay.HidePointer)
        {
            scene.ComposeWithoutPointer(frame);
        }
        else
        {
            scene.Compose(frame);
        }

        uint[] doubled = RepeatRows(frame);

        overlay.Draw(doubled, Scene.FRAME_WIDTH);

        if (overlay.PointerOnTop && !overlay.HidePointer)
        {
            scene.DrawPointerOn(doubled, ROW_SCALE);
        }

        return (doubled, Scene.FRAME_HEIGHT * ROW_SCALE);
    }

    // Every row of the frame twice.
    private static uint[] RepeatRows(uint[] frame)
    {
        uint[] doubled = new uint[frame.Length * ROW_SCALE];

        for (int row = 0; row < Scene.FRAME_HEIGHT; row++)
        {
            for (int copy = 0; copy < ROW_SCALE; copy++)
            {
                Array.Copy(frame, row * Scene.FRAME_WIDTH, doubled, (row * ROW_SCALE + copy) * Scene.FRAME_WIDTH, Scene.FRAME_WIDTH);
            }
        }

        return doubled;
    }
}
