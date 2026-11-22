using FaeryTale.Game.Files;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// Video: loading sprites and image banks.
public sealed partial class Video
{
    public void ForgetSprites() => Graphics.NextShape = 0;

    public void RewindSpritesTo(byte slot) => Graphics.NextShape = Graphics.SeqList[slot].Location;

    public void StartSpriteSheet(byte slot) => Graphics.SeqList[slot].Location = Graphics.NextShape;

    public void LoadSpriteFile(int file) => SpriteSheets.LoadSpriteFile(this, file);

    public void FinishSpriteLoad(short slot) => SpriteSheets.FinishSpriteLoad(this, slot);

    public void LoadImageBank(int bank, short firstBlock) => Graphics.LoadImageBank(bank, firstBlock);
}
