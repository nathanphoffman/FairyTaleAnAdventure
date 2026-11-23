namespace FaeryTale.Game.Render;

/// Where a sprite sequence (all the frames of one kind of character) is in memory (struct
/// seq_info), and where its transparency mask is.
public struct SeqInfo
{
    public short Width;
    public short Height;
    public short Count;

    /// Where the frames start in the sprite memory, and where the masks start.
    public int Location;
    public int MaskLoc;
    public short Bytes;
}
