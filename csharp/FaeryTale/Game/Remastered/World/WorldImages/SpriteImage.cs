using FaeryTale.Game.Render;

namespace FaeryTale.Game.Remastered;

/// A sprite picture decoded for the new drawer: a colour number (0-31) for every pixel and whether
/// the pixel is part of the figure or see-through.
public sealed record SpriteImage(int Width, int Height, byte[] Pixels, bool[] Opaque);
