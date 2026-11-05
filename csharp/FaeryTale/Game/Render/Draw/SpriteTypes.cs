namespace FaeryTale.Game.Render;

/// Which image to draw for an actor, and where on the screen it goes.
internal sealed class Sprite
{
    /// The kind of sprite sheet the image is in (PHIL, OBJECTS, ...).
    public short Atype;

    /// The image number in that sheet.
    public short Inum;

    public short XStart;
    public short YStart;
    public short XStop;
    public short YStop;

    /// The line the actor stands on; things hidden behind scenery are cut against it.
    public short Ground;

    public Sprite Copy() => (Sprite)MemberwiseClone();
}

/// The part of a sprite that is on the screen.
internal sealed class Clip
{
    public short XStart;
    public short YStart;
    public short XStop;
    public short YStop;

    /// How many pixels of the left edge, and lines of the top, are cut off.
    public sbyte XOff;
    public sbyte YOff;
}

/// Where the clipped sprite starts and how big it is, in words and lines.
internal readonly record struct BlitExtent(sbyte Xbw, sbyte BlitWide, sbyte BlitHigh);

/// A sprite cut to the screen, ready to blit.
internal sealed record Placed(Sprite Sprite, Clip Clip, sbyte BytesWide);

/// What is left of the frame's save buffers as the actors are drawn one by one.
internal sealed class Frame
{
    /// How many spare planes have been used for small background saves.
    public byte Crack;

    /// Next free byte of the page's background-save buffer.
    public Loc BackAlloc;
}
