using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// How one sprite is drawn this frame and where its background is saved (struct sshape).
/// CoffOffset is the byte offset of the sprite in a bitplane and Cmod the bytes to skip to the next
/// line; BlitSize is the blitter size word (height * 64 + width in words).
public struct SShape
{
    public Loc BackSave;
    public short SaveSize;
    public short BlitSize;
    public short Coff;
    public short Cmod;
}
