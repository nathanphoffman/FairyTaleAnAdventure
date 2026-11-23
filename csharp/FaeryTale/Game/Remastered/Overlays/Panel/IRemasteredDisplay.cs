using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The parts of a display that only Remastered uses: the video its world drawer reads from, and
/// the text and dark panels drawn over the picture.
public interface IRemasteredDisplay
{
    /// The video the frames come from: the new world drawer reads the game's tiles and sprites
    /// from it. Set when a game starts.
    Video? Source { get; set; }

    /// Sets the text and dark panels drawn over the picture (an empty overlay removes them).
    void SetOverlay(Overlay overlay);
}
