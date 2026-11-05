using FaeryTale.Game.Files;

namespace FaeryTale.Game.Render;

/// Loading a sprite file into the sprite memory and building its masks.
public static class SpriteSheets
{
    /// Bytes in a disk block.
    private const int BLOCK_BYTES = 512;

    /// Scratch memory for sprites: bytes available in Graphics.Shape.
    private const int SHAPE_SIZE = 78000;

    // A sprite sequence is loaded with room for its mask and the shifted copies after it.
    private const int SIZE_WITH_MASK = 6;
    private const int SIZE_WITHOUT_MASK = 5;

    /// Starts loading sprite file num into the next free shape memory, if it fits.
    ///
    /// Each sequence is stored as 5 planes of a size followed by a one-plane mask (the same size).
    /// The check makes sure that all of it, with room to spare, fits in the scratch area.
    public static void LoadSpriteFile(Video video, int num)
    {
        video.Graphics.SpriteVersion++;

        CFileInfo file = SpriteFiles.All[num];
        int next = video.Graphics.NextShape;
        ref SeqInfo slot = ref video.Graphics.SeqList[file.SeqNum];

        slot.Bytes = (short)(file.Height * file.Width * 2);
        int size = slot.Bytes * file.Count;
        slot.Location = next;
        slot.Width = file.Width;
        slot.Height = file.Height;
        slot.Count = file.Count;

        if (next + size * SIZE_WITH_MASK > SHAPE_SIZE)
        {
            return;
        }

        int end = Math.Min(next + file.NumBlocks * BLOCK_BYTES, video.Graphics.Shape.Length);

        Disk.LoadTrackRange((short)file.FileId, file.NumBlocks, video.Graphics.Shape.AsSpan(next, end - next));

        int mask = next + size * SIZE_WITHOUT_MASK;

        slot.MaskLoc = mask;
        video.Graphics.NextShape = mask + size;
    }

    /// Builds the masks for slot slot (reads have already finished).
    public static void FinishSpriteLoad(Video video, short slot)
    {
        video.Graphics.SpriteVersion++;

        SeqInfo s = video.Graphics.SeqList[slot];

        SpriteBlitter.MakeSpriteMasks(video.Graphics, s.Location, s.MaskLoc, s.Width, s.Height, s.Count);
    }
}
