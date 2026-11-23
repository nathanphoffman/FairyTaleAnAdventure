using FaeryTale.Game.Remastered;

namespace FaeryTale.Game.Render;

/// The window the finished frames go to. The platform supplies the real one; the video only
/// describes what is on screen (a Scene) and asks for it to be shown. It is the original parts
/// and the Remastered parts together.
public interface IDisplayDevice : IOriginalDisplay, IRemasteredDisplay
{
}
