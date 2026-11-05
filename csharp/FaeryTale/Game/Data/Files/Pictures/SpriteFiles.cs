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

    public static readonly CFileInfo JULIAN = new(1, 32, 67, 42, PHIL, 1376);
    public static readonly CFileInfo PHILLIP = new(1, 32, 67, 42, PHIL, 1418);
    public static readonly CFileInfo KEVIN = new(1, 32, 67, 42, PHIL, 1460);
    public static readonly CFileInfo OBJECT_ITEMS = new(1, 16, 116, 36, OBJECTS, 1312);
    public static readonly CFileInfo RAFT_VEHICLE = new(2, 32, 2, 3, RAFT, 1348);
    public static readonly CFileInfo TURTLE = new(2, 32, 16, 20, CARRIER, 1351);
    public static readonly CFileInfo OGRE = new(1, 32, 64, 40, ENEMY, 960);
    public static readonly CFileInfo GHOST = new(1, 32, 64, 40, ENEMY, 1080);
    public static readonly CFileInfo DARK_KNIGHT_AND_SPIDER = new(1, 32, 64, 40, ENEMY, 1000);
    public static readonly CFileInfo NECROMANCER_FARMER_LORAII = new(1, 32, 64, 40, ENEMY, 1040);
    public static readonly CFileInfo DRAGON_MONSTER = new(3, 40, 5, 12, DRAGON, 1160);
    public static readonly CFileInfo BIRD = new(4, 64, 8, 40, CARRIER, 1120);

    // Uses the same block as JULIAN, which looks like a leftover placeholder in the original;
    // it is copied as it was.
    public static readonly CFileInfo SNAKE_AND_SALAMANDER = new(1, 32, 64, 40, ENEMY, 1376);
    public static readonly CFileInfo WIZARD_AND_PRIEST = new(1, 32, 8, 5, SETFIG, 936);
    public static readonly CFileInfo ROYAL_COURT = new(1, 32, 8, 5, SETFIG, 931);
    public static readonly CFileInfo BARTENDER = new(1, 32, 8, 5, SETFIG, 941);
    public static readonly CFileInfo WITCH = new(1, 32, 8, 5, SETFIG, 946);
    public static readonly CFileInfo RANGER_AND_BEGGAR = new(1, 32, 8, 5, SETFIG, 951);

    /// All the sprite files, in the order the rest of the game numbers them.
    public static readonly CFileInfo[] All =
    [
        JULIAN, PHILLIP, KEVIN, OBJECT_ITEMS, RAFT_VEHICLE, TURTLE,
        OGRE, GHOST, DARK_KNIGHT_AND_SPIDER, NECROMANCER_FARMER_LORAII,
        DRAGON_MONSTER, BIRD, SNAKE_AND_SALAMANDER,
        WIZARD_AND_PRIEST, ROYAL_COURT, BARTENDER, WITCH, RANGER_AND_BEGGAR,
    ];
}
