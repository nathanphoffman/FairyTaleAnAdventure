using FaeryTale.Gfx;

namespace FaeryTale.Game.Output;

/// Recorder: loading sprites and image banks.
public sealed partial class Recorder
{
    public void ForgetSprites() => Record("ForgetSprites");

    public void RewindSpritesTo(byte slot) => Record("RewindSpritesTo", slot);

    public void StartSpriteSheet(byte slot) => Record("StartSpriteSheet", slot);

    public void LoadSpriteFile(int file) => Record("LoadSpriteFile", file);

    public void FinishSpriteLoad(short slot) => Record("FinishSpriteLoad", slot);

    public void LoadImageBank(int bank, short firstBlock) => Record("LoadImageBank", bank, firstBlock);
}
