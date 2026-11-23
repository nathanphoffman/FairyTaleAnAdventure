namespace FaeryTale.Game.Render;

/// The sprite blitter's work values for the sprite being drawn.
public sealed class BlitterRegisters
{
    public SShape SShape;
    public short ShadowMaskOffset;
    public short SpriteMaskOffset;
    public short SpriteMaskModulo;
    public short SpriteXShift;
    public short LastWordMask;
}
