namespace FaeryTale.Game.Files;

/// One loadable sprite file (struct cfile_info), a character set. Width is in 16-pixel words,
/// Height in lines, Count the number of frames, NumBlocks the disk blocks to read, SeqNum which
/// sequence slot to load into, and FileId the first disk block.
public sealed record CFileInfo(byte Width, byte Height, byte Count, byte NumBlocks, byte SeqNum, ushort FileId);
