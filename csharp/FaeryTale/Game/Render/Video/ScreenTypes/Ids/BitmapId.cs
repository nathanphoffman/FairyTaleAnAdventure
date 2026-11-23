using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The bitmaps the game keeps.
public enum BitmapId
{
    /// The two display pages.
    Page1,
    Page2,

    /// The text panel, all 4 planes.
    Text,

    /// The text panel, but only its first plane (the message area is drawn in one colour).
    TextPlane0,

    /// The two pages of the story book, which are also work areas for the picture loader.
    PageA,
    PageB,

    /// The pictures of the things the hero carries, one long strip, for the item list.
    Items,
}
