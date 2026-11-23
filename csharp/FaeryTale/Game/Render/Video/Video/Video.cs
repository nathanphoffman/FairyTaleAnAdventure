namespace FaeryTale.Game.Render;

/// Everything about how the game is drawn: the screen, the picture data, the sprite blitter's
/// work values. The rules never read it, they only ask for things to be drawn.
public sealed partial class Video
{
    public Screen Screen = new();
    public Graphics Graphics = new();
    public BlitterRegisters Blitter = new();

    /// Where finished frames go. The platform phase supplies the real window; until then frames
    /// are only counted.
    public IDisplayDevice Device = new NullDisplayDevice();

    /// What the new world drawer of Remastered and Reimagined is told about each frame.
    public Remastered.WorldCapture Capture = new();
}
