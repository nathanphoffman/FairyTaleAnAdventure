using FaeryTale.Game.Input;

namespace FaeryTale.Game.Remastered;

/// The world position under the mouse, whichever way the world is drawn: the new drawer at the
/// window's own size (the pointer is in window pixels) or the old 320 x 143 play field.
public static class PointerWorld
{
    /// The play field ends and the text panel starts at this screen line.
    private const int PLAY_FIELD_BOTTOM = 143;

    /// The world position of a click.
    public static (int X, int Y) OfClick(Data.Game game, InputHandler.Click click)
    {
        if (click.Native)
        {
            (double x, double y) = WorldView.Current.ToWorld(click.X, click.Y);

            return ((int)Math.Floor(x), (int)Math.Floor(y));
        }

        return ScreenMap.ToWorld(game.World, (click.X, click.Y));
    }

    /// The world position under the pointer when it is over the play field, else null.
    public static (int X, int Y)? UnderPointer(Data.Game game)
    {
        if (InputHandler.NativePointer is { } native)
        {
            return OfClick(game, new InputHandler.Click(native.X, native.Y, true));
        }

        (int x, int y) = InputHandler.RawPointer;

        return y < PLAY_FIELD_BOTTOM ? ScreenMap.ToWorld(game.World, (x, y)) : null;
    }
}
