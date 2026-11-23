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
