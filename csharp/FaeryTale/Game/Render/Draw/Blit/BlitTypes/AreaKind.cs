using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The kinds of memory a blit reads and writes.
public enum AreaKind
{
    /// A bit plane of the page being drawn.
    Plane,

    /// The blitter's queue region (saved backgrounds).
    Queue,

    /// The blitter's mask region (the scenery mask).
    Mask,

    /// The sprite sheets.
    Shapes,
}
