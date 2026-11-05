// Generated from the Rust tables by a one-off script, then kept by hand.
using System.Text;
using FaeryTale.Game.Data;

namespace FaeryTale.Game.Data;

public static partial class ObjectTables
{
    private static byte[] Latin1(string text) => Encoding.Latin1.GetBytes(text);

    /// Builds an object list entry: position, item id and state.
    private static WorldObject Obj(ushort x, ushort y, byte id, sbyte stat) =>
        new() { Xc = x, Yc = y, ObId = unchecked((sbyte)id), ObStat = stat };

    private static CompassPart Part(byte xrect, byte yrect, byte xsize, byte ysize) =>
        new() { Xrect = xrect, Yrect = yrect, Xsize = xsize, Ysize = ysize };

    public const byte QUIVER = 11;

    public const byte MONEY = 13;

    public const byte URN = 14;

    public const byte CHEST = 15;

    public const byte SACKS = 16;

    public const byte G_RING = 17;

    public const byte B_STONE = 18;

    public const byte G_JEWEL = 19;

    public const byte SCRAP = 20;

    public const byte C_ORB = 21;

    public const byte VIAL = 22;

    public const byte B_TOTEM = 23;

    public const byte J_SKULL = 24;

    public const byte GOLD_KEY = 25;

    public const byte GREY_KEY = 26;

    public const byte FOOTSTOOL = 31;

    public const byte TURTLE = 102;

    public const byte BLUE_KEY = 114;

    public const byte M_WAND = 145;

    public const byte ROSE = 147;

    public const byte FRUIT = 148;

    public const byte STATUE = 149;

    public const byte BOOK = 150;

    public const byte SHELL = 151;

    public const byte GREEN_KEY = 153;

    public const byte WHITE_KEY = 154;

    public const byte RED_KEY = 242;

    // What the bartender sells: pairs of (inventory item number, price). Item 0 is food (eaten
    // at once), item 8 is a bundle of 10 arrows, the others are single items. The buy menu choice
    // (from 5) picks the pair.
    public static readonly sbyte[] Jtrans = [
    0,3, 8,10, 11,15, 1,30, 2,45, 3,75, 13,20
];

    // What a dead enemy drops: 8 entries per treasure class (the enemy's class times 8, plus a
    // random 0-7 picks the entry). The entry is an inventory item number (gold amounts for the high
    // numbers) or 0 for nothing.
    public static readonly sbyte[] TreasureProbs = [
    0, 0, 0, 0, 0, 0, 0, 0, // no treasure
    9,11,13,31,31,17,17,32, // stone,vial,totem,gold,keys
    12,14,20,20,20,31,33,31, // keys,skull,gold,nothing
    10,10,16,16,11,17,18,19, // magic and keys
    15,21,0,0,0,0,0,0 // jade skull and white key
];

    // The weapon an enemy carries: 4 entries per weapon class (the enemy type's `arms` times 4, plus
    // a random 0-3). 1 dirk, 2 mace, 3 sword, 4 bow, 5 magic wand, 8 touch attack, 0 none.
    public static readonly sbyte[] WeaponProbs = [
    0,0,0,0, // no weapons
    1,1,1,1, // dirks only
    1,2,1,2, // dirks and maces
    1,2,3,2, // mostly maces
    4,4,3,2, // swords and bows
    5,5,5,5, // magic wand
    8,8,8,8, // touch attack!
    3,3,3,3, // swords only
];

    // Sprite frames for the falling animation: 6 per brother, picked by how far into the fall the actor is.
    public static readonly byte[] Fallstates = [
    0,0,0,0,0,0,
    0x20,0x22,0x3a,0x6f,0x70,0x71,
    0x24,0x27,0x3c,0x6f,0x70,0x71,
    0x37,0x38,0x3d,0x6f,0x70,0x71
];

    // Where the bow is drawn relative to the shooter, x, for each of the 32 shooting animation frames.
    public static readonly sbyte[] BowX = [
    1,2,3,4,3,2,1,0, 3,2,0,-2,-3,-2,0,2,
    -3,-3,-3,-3,-3,-3,-3,-2, 0,1,1,1,0,-2,-3,-2
];

    // Where the bow is drawn relative to the shooter, y, for each of the 32 shooting animation frames.
    public static readonly sbyte[] BowY = [
    8,8,8,7,8,8,8,8, 11,12,13,13,13,13,13,12,
    8,7,6,5,6,7,8,9, 12,12,12,12,12,12,11,12
];

    // Where an arrow starts relative to the shooter, x, for each of the 8 facings.
    public static readonly sbyte[] Bowshotx = [
    0, 0, 3, 6,-3,-3,-3,-6
];

    // Where an arrow starts relative to the shooter, y, for each of the 8 facings.
    public static readonly sbyte[] Bowshoty = [
    -6,-6,-1, 0, 6, 8, 0,-1
];

    // Where a fireball (non-bow missile) starts relative to the shooter, y, for each of the 8 facings.
    public static readonly sbyte[] Gunshoty = [
    2, 0, 4, 7, 9, 4, 7, 8
];

    // The witch beam: 64 directions of 4 numbers (x1, y1, x2, y2), the near and far end of each edge,
    // going around a circle of radius 100. See `witch_fx`.
    public static readonly sbyte[] Witchpoints = [
    // 64 points - 256 entries
    0,100,0,10, 9,99,0,9, 19,98,1,9, 29,95,2,9,
    38,92,3,9, 47,88,4,8, 55,83,5,8, 63,77,6,7,
    70,70,7,7, 77,63,7,6, 83,55,8,5, 88,47,8,4,
    92,38,9,3, 95,29,9,2, 98,19,9,1, 99,9,9,0,
    100,0,10,0, 99,-10,9,-1, 98,-20,9,-2, 95,-30,9,-3,
    92,-39,9,-4, 88,-48,8,-5, 83,-56,8,-6, 77,-64,7,-7,
    70,-71,7,-8, 63,-78,6,-8, 55,-84,5,-9, 47,-89,4,-9,
    38,-93,3,-10, 29,-96,2,-10, 19,-99,1,-10, 9,-100,0,-10,
    0,-100,0,-10, -10,-100,-1,-10,-20,-99,-2,-10, -30,-96,-3,-10,
    -39,-93,-4,-10, -48,-89,-5,-9, -56,-84,-6,-9, -64,-78,-7,-8,
    -71,-71,-8,-8, -78,-64,-8,-7, -84,-56,-9,-6, -89,-48,-9,-5,
    -93,-39,-10,-4, -96,-30,-10,-3, -99,-20,-10,-2, -100,-10,-10,-1,
    -100,-1,-10,-1, -100,9,-10,0, -99,19,-10,1, -96,29,-10,2,
    -93,38,-10,3, -89,47,-9,4, -84,55,-9,5, -78,63,-8,6,
    -71,70,-8,7, -64,77,-7,7, -56,83,-6,8, -48,88,-5,8,
    -39,92,-4,9, -30,95,-3,9, -20,98,-2,9, -10,99,-1,9,
];

    // Colours of the sky as the sun rises in the win sequence (53 steps from night to day).
    public static readonly short[] SunColors = [
    0x000, 0x000, 0x001, 0x002, 0x002, 0x012, 0x112, 0x113,
    0x113, 0x214, 0x224, 0x225, 0x326, 0x326, 0x337, 0x338,
    0x438, 0x439, 0x449, 0x54A, 0x54B, 0x54B, 0x55C, 0x65C,
    0x65D, 0x66E, 0x76E, 0x76F, 0x86F, 0x97E, 0xA7E, 0xB7E,
    0xB8D, 0xC8D, 0xD8D, 0xD8D, 0xE9C, 0xE9C, 0xE9B, 0xE9B,
    0xFAA, 0xFAA, 0xF99, 0xF98, 0xF98, 0xF97, 0xF86, 0xF85,
    0xF84, 0xF84, 0xF93, 0xF92, 0x76F
];

    // The 32 page colours at full brightness (colour 31 changes with the region).
    // Each is 12 bits: red, green and blue in 4 bits each.
    public static readonly ushort[] InitialPageColors = [
    0x0000,0x0FFF,0x0E96,0x0B63,0x0631,0x07BF,0x0333,0x0DB8,
    0x0223,0x0445,0x0889,0x0BBC,0x0521,0x0941,0x0F82,0x0FC7,
    0x0040,0x0070,0x00B0,0x06F6,0x0005,0x0009,0x000D,0x037F,
    0x0C00,0x0F50,0x0FA0,0x0FF6,0x0EB6,0x0EA5,0x000F,0x0BDF
];

    // Item translation: pairs of (object id in the world, inventory slot). When the hero picks up
    // an object, the pair with its id says which inventory count to raise. Ended by 0, 0.
    public static readonly byte[] Itrans = [
    QUIVER, 35,
    B_STONE, 9,
    G_JEWEL, 10,
    VIAL, 11,
    C_ORB, 12,
    B_TOTEM, 13,
    G_RING, 14,
    J_SKULL, 15,
    M_WAND, 4,
    27, 5,
    8, 2,
    9, 1,
    12, 0,
    10, 3,
    ROSE, 23,
    FRUIT, 24,
    STATUE, 25,
    BOOK, 26,
    SHELL, 6,
    155, 7,
    136, 27,
    137, 28,
    138, 29,
    139, 22,
    140, 30,
    GOLD_KEY, 16,
    GREEN_KEY, 17,
    BLUE_KEY, 18,
    RED_KEY, 19,
    GREY_KEY, 20,
    WHITE_KEY, 21,
    0, 0,
];

    // Random treasure scattered over a region on first visit, picked with 4 random bits.
    public static readonly byte[] RandTreasure = [
    SACKS, SACKS, SACKS, SACKS, CHEST, MONEY, GOLD_KEY, QUIVER, GREY_KEY, GREY_KEY, GREY_KEY, RED_KEY, B_TOTEM, VIAL, WHITE_KEY, CHEST,
];

    // The compass parts for the 8 directions; 8 and 9 mean "not walking" and light nothing.
    public static readonly CompassPart[] Comptable = [
    Part(0, 0, 16, 8),
    Part(16, 0, 16, 9),
    Part(32, 0, 16, 8),
    Part(30, 8, 18, 8),
    Part(32, 16, 16, 8),
    Part(16, 13, 16, 11),
    Part(0, 16, 16, 8),
    Part(0, 8, 18, 8),
    Part(0, 0, 1, 1),
    Part(0, 0, 1, 1),
];

    // An unused entry, kept so a region has room for the random treasure added when it is visited.
    private static readonly WorldObject BLANK = Obj(0, 0, 0, 0);

    // Global objects.
    public static readonly WorldObject[] InitialObListg = [
   // global objects
    Obj(0, 0, 0, 0), // special item (for 'give')
    Obj(0, 0, 28, 0), // dead brother 1 - to be filled in later
    Obj(0, 0, 28, 0), // dead brother 2 - to be filled in later
    Obj(19316, 15747, 11, 0), // ghost brother 1 - to be filled in later
    Obj(18196, 15735, 11, 0), // ghost brother 2 - to be filled in later
    Obj(12439, 36202, 10, 3), // spectre - can be disabled
    Obj(11092, 38526, STATUE, 1), // gold statues (6 = seahold)
    Obj(25737, 10662, STATUE, 1), // (7 = ogre den)
    Obj(2910, 39023, STATUE, 1), // (8 = octal room)
    Obj(12025, 37639, STATUE, 0), // (9 = sorceress)
    Obj(6700, 33766, STATUE, 0), // (10 = priest)
];

    // Snow land objects.
    public static readonly WorldObject[] InitialObList0 = [
   // snow land objects
    Obj(3340, 6735, 12, 3), // ranger west
    Obj(9678, 7035, 12, 3), // ranger east
    Obj(4981, 6306, 12, 3), // ranger north
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

    // Maze forest (north) objects.
    public static readonly WorldObject[] InitialObList1 = [
   // maze forest (north) objects
    // {23297,5797,TURTLE,1},
    Obj(23087, 5667, TURTLE, 1),
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

    // Swamp land objects.
    public static readonly WorldObject[] InitialObList2 = [
   // swamp land objects
    Obj(13668, 15000, 0, 3), // wizard
    Obj(10627, 13154, 0, 3), // wizard
    Obj(4981, 10056, 12, 3), // ranger
    Obj(13950, 11087, SACKS, 1),
    Obj(10344, 36171, SHELL, 1), // shell
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

    // Maze forest (south) and manor objects (and tambry).
    public static readonly WorldObject[] InitialObList3 = [
   // maze forest (south) and manor objects (and tambry)
    Obj(19298, 16128, CHEST, 1),
    Obj(18310, 15969, 13, 3), // beggar
    Obj(20033, 14401, 0, 3), // wizard
    Obj(24794, 13102, 13, 3), // beggar
    Obj(21626, 15446, B_STONE, 1), // 3 items at stone ring

    Obj(21616, 15456, MONEY, 1),
    Obj(21636, 15456, G_RING, 1),
    Obj(20117, 14222, G_JEWEL, 1),
    Obj(24185, 9840, SACKS, 1),
    Obj(25769, 10617, MONEY, 1),

    Obj(25678, 10703, B_STONE, 1),
    Obj(17177, 10599, SCRAP, 1),
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

    // Desert objects.
    public static readonly WorldObject[] InitialObList4 = [
   // desert objects
    Obj(0, 0, 0, 0), // dummy
    Obj(0, 0, 0, 0), // dummy
    Obj(6817, 19693, 13, 3), // beggar - must be 3rd object
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

    // Farm and city objects.
    public static readonly WorldObject[] InitialObList5 = [
   // farm and city objects
    Obj(22184, 21156, 13, 3), // beggar
    Obj(18734, 17595, G_RING, 1),
    Obj(21294, 22648, CHEST, 1),
    Obj(22956, 19955, 0, 3), // wizard
    Obj(28342, 22613, 0, 3), // wizard
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

    // Lava plain objects.
    public static readonly WorldObject[] InitialObList6 = [
   // lava plain objects
    Obj(24794, 13102, 13, 3), // DUMMY OBJECT
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

    // Southern mountain land objects.
    public static readonly WorldObject[] InitialObList7 = [
   // southern mountain land objects
    Obj(23297, 5797, TURTLE, 1), // dummy object
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

    // Interiors of buildings.
    public static readonly WorldObject[] InitialObList8 = [
   // interiors of buildings
    Obj(6700, 33756, 1, 3), // priest in chapel
    Obj(5491, 33780, 5, 3), // king on throne - never mask
    Obj(5592, 33764, 6, 3), // noble
    Obj(5514, 33668, 2, 3), // guard
    Obj(5574, 33668, 2, 3), // guard
    Obj(8878, 38995, 0, 3), // wizard
    Obj(7776, 34084, 0, 3), // wizard
    Obj(5514, 33881, 3, 3), // guard
    Obj(5574, 33881, 3, 3), // guard
    Obj(10853, 35656, 4, 3), // princess
    Obj(12037, 37614, 7, 3), // sorceress

    Obj(11013, 36804, 9, 3), // witch
    Obj(9631, 38953, 8, 3), // bartender
    Obj(10191, 38953, 8, 3), // bartender
    Obj(10649, 38953, 8, 3), // bartender
    Obj(2966, 33964, 8, 3), // bartender
    Obj(9532, 40002, FOOTSTOOL, 1),
    Obj(6747, 33751, FOOTSTOOL, 1),
    Obj(11410, 36169, 27+128, 1), // sunstone
    Obj(9550, 39964, B_TOTEM, 1), // cabinet item
    Obj(9552, 39964, B_TOTEM, 1),

    Obj(9682, 39964, B_TOTEM, 1), // cabinet item
    Obj(9684, 39964, B_TOTEM, 1),
    Obj(9532, 40119, B_TOTEM, 1), // on table
    Obj(9575, 39459, URN, 1),
    Obj(9590, 39459, URN, 1),
    Obj(9605, 39459, URN, 1),
    Obj(9680, 39453, VIAL, 1),
    Obj(9682, 39453, VIAL, 1),
    Obj(9784, 39453, VIAL, 1),
    Obj(9668, 39554, CHEST, 1),

    Obj(11090, 39462, MONEY, 1),
    Obj(11108, 39458, B_TOTEM, 1),
    Obj(11118, 39459, B_TOTEM, 1),
    Obj(11128, 39459, B_TOTEM, 1),
    Obj(11138, 39458, B_TOTEM, 1),
    Obj(11148, 39459, B_TOTEM, 1),
    Obj(11158, 39459, B_TOTEM, 1),
    Obj(11855, 36206, FOOTSTOOL, 1),
    Obj(11909, 36198, CHEST, 1),
    Obj(11918, 36246, B_TOTEM, 1), // cabinet items?

    Obj(11928, 36246, B_TOTEM, 1),
    Obj(11938, 36246, B_TOTEM, 1),
    Obj(12212, 38481, CHEST, 1),
    Obj(11652, 38481, RED_KEY, 1),
    Obj(10427, 39977, FOOTSTOOL, 1),
    Obj(10323, 40071, URN, 1),
    Obj(10059, 38472, SACKS, 1),
    Obj(10344, 36171, SHELL, 1), // shell
    Obj(11936, 36207, SCRAP, 1), // spectre note
    Obj(9674, 35687, URN, 1),
    Obj(5473, 38699, ROSE, 1),
    Obj(7185, 34342, FRUIT, 1),
    Obj(7190, 34342, FRUIT, 1),
    Obj(7195, 34342, FRUIT, 1),
    Obj(7185, 34347, FRUIT, 1),
    Obj(7190, 34347, FRUIT, 1),
    Obj(7195, 34347, FRUIT, 1),
    Obj(6593, 34085, FRUIT, 1),
    Obj(6598, 34085, FRUIT, 1),
    Obj(6593, 34090, FRUIT, 1),
    Obj(6598, 34090, FRUIT, 1),

   // 'look' items
    Obj(3872, 33546, GOLD_KEY, 5),
    Obj(3887, 33510, B_TOTEM, 5),
    Obj(4495, 33510, VIAL, 5),
    Obj(3327, 33383, J_SKULL, 5),
    Obj(4221, 34119, QUIVER, 5),
    Obj(7610, 33604, VIAL, 5),
    Obj(7616, 33522, MONEY, 5),
    Obj(9570, 35768, B_STONE, 5),
    Obj(9668, 35769, QUIVER, 5),
    Obj(9553, 38951, G_RING, 5),
    Obj(10062, 39005, J_SKULL, 5),
    Obj(10577, 38951, VIAL, 5),
    Obj(11062, 39514, MONEY, 5),
    Obj(8845, 39494, WHITE_KEY, 5),
    Obj(6542, 39494, G_JEWEL, 5),
    Obj(7313, 38992, RED_KEY, 5),
];

    // Underground areas.
    public static readonly WorldObject[] InitialObList9 = [
   // underground areas
    Obj(7540, 38528, M_WAND, 1),
    Obj(9624, 36559, M_WAND, 1),
    Obj(9624, 37459, M_WAND, 1),
    Obj(8337, 36719, M_WAND, 1),
    Obj(8154, 34890, CHEST, 1),
    Obj(7826, 35741, CHEST, 1),
    Obj(3460, 37260, 0, 3), // wizard
    Obj(8485, 35725, MONEY, 1),
    Obj(3723, 39340, 128+10, 1), // king's bone
];

    // Entries in the global object list.
    public static readonly short Glbobs = 11;
}
