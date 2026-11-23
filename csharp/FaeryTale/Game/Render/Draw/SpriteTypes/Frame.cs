namespace FaeryTale.Game.Render;

/// What is left of the frame's save buffers as the actors are drawn one by one.
internal sealed class Frame
{
    /// How many spare planes have been used for small background saves.
    public byte Crack;

    /// Next free byte of the page's background-save buffer.
    public Loc BackAlloc;
}
