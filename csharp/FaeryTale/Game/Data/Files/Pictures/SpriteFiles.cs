namespace FaeryTale.Game.Files;

/// One loadable sprite file (struct cfile_info), a character set. Width is in 16-pixel words,
/// Height in lines, Count the number of frames, NumBlocks the disk blocks to read, SeqNum which
/// sequence slot to load into, and FileId the first disk block.
public sealed record CFileInfo(byte Width, byte Height, byte Count, byte NumBlocks, byte SeqNum, ushort FileId);

/// The sprite files on the disk image.
public static class SpriteFiles
{
    // The sequence slots.
    public const byte PHIL = 0;
    public const byte OBJECTS = 1;
    public const byte ENEMY = 2;
    public const byte RAFT = 3;
    public const byte SETFIG = 4;
    public const byte CARRIER = 5;
    public const byte DRAGON = 6;

    /// Index 0-2 are the three brothers, 3 the objects, 4 the raft, 5 the turtle, 6-9 and 12
    /// monster sets, 10 the dragon, 11 the bird, 13-17 the people (wizard, royal court, bartender,
    /// witch, ranger).
    ///
    /// Entry 12 (snake and salamander) uses the same block as entry 0, which looks like a
    /// leftover placeholder in the original; it is copied as it was.
    public static readonly CFileInfo[] All =
    [
        new(1, 32, 67, 42, PHIL, 1376),     // julian
        new(1, 32, 67, 42, PHIL, 1418),     // phillip
        new(1, 32, 67, 42, PHIL, 1460),     // kevin
        new(1, 16, 116, 36, OBJECTS, 1312), // objects
        new(2, 32, 2, 3, RAFT, 1348),       // raft
        new(2, 32, 16, 20, CARRIER, 1351),  // turtle
        new(1, 32, 64, 40, ENEMY, 960),     // ogre file
        new(1, 32, 64, 40, ENEMY, 1080),    // ghost file
        new(1, 32, 64, 40, ENEMY, 1000),    // dknight file (spiders)
        new(1, 32, 64, 40, ENEMY, 1040),    // necromancer (farmer / loraii)
        new(3, 40, 5, 12, DRAGON, 1160),    // dragon
        new(4, 64, 8, 40, CARRIER, 1120),   // bird
        new(1, 32, 64, 40, ENEMY, 1376),    // snake and salamander
        new(1, 32, 8, 5, SETFIG, 936),      // wizard/priest
        new(1, 32, 8, 5, SETFIG, 931),      // royal set
        new(1, 32, 8, 5, SETFIG, 941),      // bartender
        new(1, 32, 8, 5, SETFIG, 946),      // witch
        new(1, 32, 8, 5, SETFIG, 951),      // ranger/begger
    ];
}
