// Generated from the Rust tables by a one-off script, then kept by hand.
using System.Text;
using FaeryTale.Game.Data;

namespace FaeryTale.Game.Data;

public static partial class Tables
{
    private static byte[] Latin1(string text) => Encoding.Latin1.GetBytes(text);

    /// Entry index of a table, or the first entry if index is out of range. (The C code read
    /// whatever lay after the table; this keeps the game running with a harmless value instead.)
    public static T Entry<T>(T[] table, int index) => index >= 0 && index < table.Length ? table[index] : table[0];

    // The kinds of people (setfigs): which sprite file, the first image, and whether they can talk.
    public static readonly SetFigEntry[] SetfigTable = [
    new SetFigEntry { CfileEntry = 13, ImageBase = 0, CanTalk = 1 },
    new SetFigEntry { CfileEntry = 13, ImageBase = 4, CanTalk = 1 },
    new SetFigEntry { CfileEntry = 14, ImageBase = 0, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 14, ImageBase = 1, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 14, ImageBase = 2, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 14, ImageBase = 4, CanTalk = 1 },
    new SetFigEntry { CfileEntry = 14, ImageBase = 6, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 14, ImageBase = 7, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 15, ImageBase = 0, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 16, ImageBase = 0, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 16, ImageBase = 6, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 16, ImageBase = 7, CanTalk = 0 },
    new SetFigEntry { CfileEntry = 17, ImageBase = 0, CanTalk = 1 },
    new SetFigEntry { CfileEntry = 17, ImageBase = 4, CanTalk = 1 },
];

    // The monsters: hit points, aggression, weapon row, cleverness, treasure and sprite file.
    public static readonly Encounter[] EncounterChart = [
    new Encounter { Hitpoints = 18, Agressive = 1, Arms = 2, Cleverness = 0, Treasure = 2, FileId = 6 },
    new Encounter { Hitpoints = 12, Agressive = 1, Arms = 4, Cleverness = 1, Treasure = 1, FileId = 6 },
    new Encounter { Hitpoints = 16, Agressive = 1, Arms = 6, Cleverness = 1, Treasure = 4, FileId = 7 },
    new Encounter { Hitpoints = 8, Agressive = 1, Arms = 3, Cleverness = 0, Treasure = 3, FileId = 7 },
    new Encounter { Hitpoints = 16, Agressive = 1, Arms = 6, Cleverness = 1, Treasure = 0, FileId = 8 },
    new Encounter { Hitpoints = 9, Agressive = 1, Arms = 3, Cleverness = 0, Treasure = 0, FileId = 7 },
    new Encounter { Hitpoints = 10, Agressive = 1, Arms = 6, Cleverness = 1, Treasure = 0, FileId = 8 },
    new Encounter { Hitpoints = 40, Agressive = 1, Arms = 7, Cleverness = 1, Treasure = 0, FileId = 8 },
    new Encounter { Hitpoints = 12, Agressive = 1, Arms = 6, Cleverness = 1, Treasure = 0, FileId = 9 },
    new Encounter { Hitpoints = 50, Agressive = 1, Arms = 5, Cleverness = 0, Treasure = 0, FileId = 9 },
    new Encounter { Hitpoints = 4, Agressive = 0, Arms = 0, Cleverness = 0, Treasure = 0, FileId = 9 },
];

    // Next state of a weapon swing, for each state and each of 4 random picks.
    public static readonly sbyte[][] SwingTransitions = [
    [1, 8, 0, 1],
    [2, 0, 1, 0],
    [3, 1, 2, 8],
    [4, 2, 3, 7],
    [5, 3, 4, 6],
    [6, 4, 5, 5],
    [8, 5, 6, 4],
    [8, 6, 7, 3],
    [0, 6, 8, 2],
];

    // What to draw for each animation state: the figure image, the weapon image, and where the weapon is.
    public static readonly StateEntry[] AnimationFrames = [
    new StateEntry { Figure = 0, WpnNo = 11, WpnX = -2, WpnY = 11 },
    new StateEntry { Figure = 1, WpnNo = 11, WpnX = -3, WpnY = 11 },
    new StateEntry { Figure = 2, WpnNo = 11, WpnX = -3, WpnY = 10 },
    new StateEntry { Figure = 3, WpnNo = 11, WpnX = -3, WpnY = 9 },
    new StateEntry { Figure = 4, WpnNo = 11, WpnX = -3, WpnY = 10 },
    new StateEntry { Figure = 5, WpnNo = 11, WpnX = -3, WpnY = 11 },
    new StateEntry { Figure = 6, WpnNo = 11, WpnX = -2, WpnY = 11 },
    new StateEntry { Figure = 7, WpnNo = 11, WpnX = -1, WpnY = 11 },
    new StateEntry { Figure = 8, WpnNo = 9, WpnX = -12, WpnY = 11 },
    new StateEntry { Figure = 9, WpnNo = 9, WpnX = -11, WpnY = 12 },
    new StateEntry { Figure = 10, WpnNo = 9, WpnX = -8, WpnY = 13 },
    new StateEntry { Figure = 11, WpnNo = 9, WpnX = -4, WpnY = 13 },
    new StateEntry { Figure = 12, WpnNo = 9, WpnX = 0, WpnY = 13 },
    new StateEntry { Figure = 13, WpnNo = 9, WpnX = -4, WpnY = 13 },
    new StateEntry { Figure = 14, WpnNo = 9, WpnX = -8, WpnY = 13 },
    new StateEntry { Figure = 15, WpnNo = 9, WpnX = -11, WpnY = 12 },
    new StateEntry { Figure = 16, WpnNo = 14, WpnX = -1, WpnY = 1 },
    new StateEntry { Figure = 17, WpnNo = 14, WpnX = -1, WpnY = 2 },
    new StateEntry { Figure = 18, WpnNo = 14, WpnX = -1, WpnY = 3 },
    new StateEntry { Figure = 19, WpnNo = 14, WpnX = -1, WpnY = 4 },
    new StateEntry { Figure = 20, WpnNo = 14, WpnX = -1, WpnY = 3 },
    new StateEntry { Figure = 21, WpnNo = 14, WpnX = -1, WpnY = 2 },
    new StateEntry { Figure = 22, WpnNo = 14, WpnX = -1, WpnY = 1 },
    new StateEntry { Figure = 23, WpnNo = 14, WpnX = -1, WpnY = 1 },
    new StateEntry { Figure = 24, WpnNo = 10, WpnX = 5, WpnY = 12 },
    new StateEntry { Figure = 25, WpnNo = 10, WpnX = 3, WpnY = 12 },
    new StateEntry { Figure = 26, WpnNo = 10, WpnX = 2, WpnY = 12 },
    new StateEntry { Figure = 27, WpnNo = 10, WpnX = 3, WpnY = 12 },
    new StateEntry { Figure = 28, WpnNo = 10, WpnX = 5, WpnY = 12 },
    new StateEntry { Figure = 29, WpnNo = 10, WpnX = 6, WpnY = 12 },
    new StateEntry { Figure = 30, WpnNo = 10, WpnX = 6, WpnY = 11 },
    new StateEntry { Figure = 31, WpnNo = 10, WpnX = 6, WpnY = 12 },
    new StateEntry { Figure = 32, WpnNo = 11, WpnX = -2, WpnY = 12 },
    new StateEntry { Figure = 32, WpnNo = 10, WpnX = 0, WpnY = 12 },
    new StateEntry { Figure = 33, WpnNo = 0, WpnX = 2, WpnY = 10 },
    new StateEntry { Figure = 34, WpnNo = 1, WpnX = 4, WpnY = 6 },
    new StateEntry { Figure = 34, WpnNo = 2, WpnX = 1, WpnY = 4 },
    new StateEntry { Figure = 34, WpnNo = 3, WpnX = 0, WpnY = 4 },
    new StateEntry { Figure = 36, WpnNo = 4, WpnX = -5, WpnY = 0 },
    new StateEntry { Figure = 36, WpnNo = 5, WpnX = -10, WpnY = 1 },
    new StateEntry { Figure = 35, WpnNo = 12, WpnX = -5, WpnY = 5 },
    new StateEntry { Figure = 36, WpnNo = 0, WpnX = 0, WpnY = 6 },
    new StateEntry { Figure = 38, WpnNo = 85, WpnX = -6, WpnY = 5 },
    new StateEntry { Figure = 37, WpnNo = 81, WpnX = -6, WpnY = 5 },
    new StateEntry { Figure = 40, WpnNo = 9, WpnX = -7, WpnY = 12 },
    new StateEntry { Figure = 40, WpnNo = 8, WpnX = -9, WpnY = 9 },
    new StateEntry { Figure = 41, WpnNo = 7, WpnX = -10, WpnY = 5 },
    new StateEntry { Figure = 42, WpnNo = 7, WpnX = -12, WpnY = 4 },
    new StateEntry { Figure = 42, WpnNo = 6, WpnX = -12, WpnY = 3 },
    new StateEntry { Figure = 42, WpnNo = 5, WpnX = -12, WpnY = 3 },
    new StateEntry { Figure = 44, WpnNo = 5, WpnX = -8, WpnY = 3 },
    new StateEntry { Figure = 44, WpnNo = 14, WpnX = -7, WpnY = 6 },
    new StateEntry { Figure = 43, WpnNo = 13, WpnX = -7, WpnY = 8 },
    new StateEntry { Figure = 42, WpnNo = 5, WpnX = -12, WpnY = 3 },
    new StateEntry { Figure = 46, WpnNo = 86, WpnX = -3, WpnY = 0 },
    new StateEntry { Figure = 45, WpnNo = 82, WpnX = -3, WpnY = 0 },
    new StateEntry { Figure = 48, WpnNo = 14, WpnX = -3, WpnY = 0 },
    new StateEntry { Figure = 48, WpnNo = 6, WpnX = -3, WpnY = -1 },
    new StateEntry { Figure = 49, WpnNo = 5, WpnX = -2, WpnY = -3 },
    new StateEntry { Figure = 50, WpnNo = 5, WpnX = -3, WpnY = -4 },
    new StateEntry { Figure = 50, WpnNo = 4, WpnX = 0, WpnY = 0 },
    new StateEntry { Figure = 50, WpnNo = 3, WpnX = 3, WpnY = 0 },
    new StateEntry { Figure = 52, WpnNo = 4, WpnX = 6, WpnY = 1 },
    new StateEntry { Figure = 52, WpnNo = 15, WpnX = 7, WpnY = 3 },
    new StateEntry { Figure = 51, WpnNo = 14, WpnX = 1, WpnY = 6 },
    new StateEntry { Figure = 50, WpnNo = 4, WpnX = 0, WpnY = 0 },
    new StateEntry { Figure = 54, WpnNo = 87, WpnX = 3, WpnY = 0 },
    new StateEntry { Figure = 53, WpnNo = 83, WpnX = 3, WpnY = 0 },
    new StateEntry { Figure = 56, WpnNo = 10, WpnX = 5, WpnY = 11 },
    new StateEntry { Figure = 56, WpnNo = 0, WpnX = 6, WpnY = 9 },
    new StateEntry { Figure = 57, WpnNo = 1, WpnX = 10, WpnY = 6 },
    new StateEntry { Figure = 58, WpnNo = 1, WpnX = 10, WpnY = 5 },
    new StateEntry { Figure = 58, WpnNo = 2, WpnX = 7, WpnY = 3 },
    new StateEntry { Figure = 58, WpnNo = 3, WpnX = 6, WpnY = 3 },
    new StateEntry { Figure = 60, WpnNo = 4, WpnX = 1, WpnY = 0 },
    new StateEntry { Figure = 60, WpnNo = 3, WpnX = 3, WpnY = 2 },
    new StateEntry { Figure = 59, WpnNo = 15, WpnX = 4, WpnY = 1 },
    new StateEntry { Figure = 58, WpnNo = 4, WpnX = 5, WpnY = 1 },
    new StateEntry { Figure = 62, WpnNo = 84, WpnX = 3, WpnY = 0 },
    new StateEntry { Figure = 61, WpnNo = 80, WpnX = 3, WpnY = 0 },
    new StateEntry { Figure = 47, WpnNo = 0, WpnX = 5, WpnY = 11 },
    new StateEntry { Figure = 63, WpnNo = 0, WpnX = 6, WpnY = 9 },
    new StateEntry { Figure = 39, WpnNo = 0, WpnX = 6, WpnY = 9 },
    new StateEntry { Figure = 55, WpnNo = 10, WpnX = 5, WpnY = 11 },
    new StateEntry { Figure = 64, WpnNo = 10, WpnX = 5, WpnY = 11 },
    new StateEntry { Figure = 65, WpnNo = 10, WpnX = 5, WpnY = 11 },
    new StateEntry { Figure = 66, WpnNo = 10, WpnX = 5, WpnY = 11 },
];

    // Every door: where it is outside, where it leads inside, its kind and the sectors it joins.
    public static readonly Door[] DoorList = [
    new Door { Xc1 = 4464, Yc1 = 20576, Xc2 = 10352, Yc2 = 35680, Kind = 1, Secs = 1 },
    new Door { Xc1 = 4464, Yc1 = 20576, Xc2 = 10352, Yc2 = 35680, Kind = 1, Secs = 1 },
    new Door { Xc1 = 4464, Yc1 = 20576, Xc2 = 10352, Yc2 = 35680, Kind = 1, Secs = 1 },
    new Door { Xc1 = 4464, Yc1 = 20576, Xc2 = 10352, Yc2 = 35680, Kind = 1, Secs = 1 },
    new Door { Xc1 = 5008, Yc1 = 7008, Xc2 = 6528, Yc2 = 35936, Kind = 18, Secs = 2 },
    new Door { Xc1 = 6000, Yc1 = 27296, Xc2 = 8816, Yc2 = 38560, Kind = 9, Secs = 1 },
    new Door { Xc1 = 6512, Yc1 = 25248, Xc2 = 8048, Yc2 = 38560, Kind = 9, Secs = 1 },
    new Door { Xc1 = 6816, Yc1 = 19360, Xc2 = 5024, Yc2 = 38304, Kind = 17, Secs = 1 },
    new Door { Xc1 = 6816, Yc1 = 19552, Xc2 = 5024, Yc2 = 38752, Kind = 17, Secs = 1 },
    new Door { Xc1 = 6944, Yc1 = 19296, Xc2 = 5920, Yc2 = 38240, Kind = 17, Secs = 1 },
    new Door { Xc1 = 7040, Yc1 = 19328, Xc2 = 5504, Yc2 = 38272, Kind = 17, Secs = 1 },
    new Door { Xc1 = 7040, Yc1 = 19520, Xc2 = 5504, Yc2 = 38720, Kind = 17, Secs = 1 },
    new Door { Xc1 = 7792, Yc1 = 15200, Xc2 = 10368, Yc2 = 40032, Kind = 3, Secs = 1 },
    new Door { Xc1 = 9344, Yc1 = 13216, Xc2 = 11904, Yc2 = 36256, Kind = 1, Secs = 1 },
    new Door { Xc1 = 10592, Yc1 = 34656, Xc2 = 11008, Yc2 = 37568, Kind = 15, Secs = 1 },
    new Door { Xc1 = 11008, Yc1 = 37568, Xc2 = 10592, Yc2 = 34688, Kind = 15, Secs = 2 },
    new Door { Xc1 = 11264, Yc1 = 29024, Xc2 = 10992, Yc2 = 37728, Kind = 9, Secs = 1 },
    new Door { Xc1 = 12144, Yc1 = 11872, Xc2 = 12672, Yc2 = 39520, Kind = 3, Secs = 1 },
    new Door { Xc1 = 12144, Yc1 = 25504, Xc2 = 7280, Yc2 = 38560, Kind = 9, Secs = 1 },
    new Door { Xc1 = 12672, Yc1 = 14528, Xc2 = 10112, Yc2 = 39104, Kind = 1, Secs = 1 },
    new Door { Xc1 = 13424, Yc1 = 19296, Xc2 = 1136, Yc2 = 36576, Kind = 15, Secs = 2 },
    new Door { Xc1 = 15840, Yc1 = 7104, Xc2 = 12000, Yc2 = 37824, Kind = 7, Secs = 1 },
    new Door { Xc1 = 15872, Yc1 = 7104, Xc2 = 12032, Yc2 = 37824, Kind = 7, Secs = 1 },
    new Door { Xc1 = 17008, Yc1 = 9568, Xc2 = 11904, Yc2 = 39520, Kind = 3, Secs = 1 },
    new Door { Xc1 = 17024, Yc1 = 15296, Xc2 = 10624, Yc2 = 39104, Kind = 1, Secs = 1 },
    new Door { Xc1 = 17888, Yc1 = 21376, Xc2 = 9680, Yc2 = 38528, Kind = 10, Secs = 1 },
    new Door { Xc1 = 18304, Yc1 = 12224, Xc2 = 9600, Yc2 = 39104, Kind = 1, Secs = 1 },
    new Door { Xc1 = 18528, Yc1 = 26176, Xc2 = 7264, Yc2 = 39488, Kind = 18, Secs = 1 },
    new Door { Xc1 = 18576, Yc1 = 26272, Xc2 = 7312, Yc2 = 39584, Kind = 11, Secs = 1 },
    new Door { Xc1 = 18784, Yc1 = 23360, Xc2 = 8800, Yc2 = 39488, Kind = 18, Secs = 1 },
    new Door { Xc1 = 18832, Yc1 = 23456, Xc2 = 8848, Yc2 = 39584, Kind = 11, Secs = 1 },
    new Door { Xc1 = 18848, Yc1 = 15552, Xc2 = 2976, Yc2 = 33472, Kind = 2, Secs = 1 },
    new Door { Xc1 = 18896, Yc1 = 15808, Xc2 = 3024, Yc2 = 33984, Kind = 2, Secs = 1 },
    new Door { Xc1 = 18896, Yc1 = 15872, Xc2 = 3024, Yc2 = 34048, Kind = 2, Secs = 1 },
    new Door { Xc1 = 18960, Yc1 = 15488, Xc2 = 3344, Yc2 = 33408, Kind = 1, Secs = 1 },
    new Door { Xc1 = 18960, Yc1 = 15680, Xc2 = 3856, Yc2 = 33600, Kind = 1, Secs = 1 },
    new Door { Xc1 = 18992, Yc1 = 15808, Xc2 = 3632, Yc2 = 34240, Kind = 1, Secs = 1 },
    new Door { Xc1 = 19040, Yc1 = 16000, Xc2 = 4192, Yc2 = 34176, Kind = 1, Secs = 1 },
    new Door { Xc1 = 19056, Yc1 = 15488, Xc2 = 4976, Yc2 = 33408, Kind = 1, Secs = 1 },
    new Door { Xc1 = 19072, Yc1 = 15680, Xc2 = 4496, Yc2 = 33600, Kind = 1, Secs = 1 },
    new Door { Xc1 = 19568, Yc1 = 12896, Xc2 = 9600, Yc2 = 40032, Kind = 3, Secs = 1 },
    new Door { Xc1 = 19808, Yc1 = 21568, Xc2 = 8032, Yc2 = 40000, Kind = 18, Secs = 1 },
    new Door { Xc1 = 19856, Yc1 = 17280, Xc2 = 12416, Yc2 = 36224, Kind = 13, Secs = 1 },
    new Door { Xc1 = 19856, Yc1 = 21664, Xc2 = 8080, Yc2 = 40096, Kind = 11, Secs = 1 },
    new Door { Xc1 = 19936, Yc1 = 27520, Xc2 = 10704, Yc2 = 38528, Kind = 10, Secs = 1 },
    new Door { Xc1 = 21344, Yc1 = 22592, Xc2 = 8800, Yc2 = 38976, Kind = 18, Secs = 1 },
    new Door { Xc1 = 21392, Yc1 = 22688, Xc2 = 8848, Yc2 = 39072, Kind = 11, Secs = 1 },
    new Door { Xc1 = 21600, Yc1 = 17728, Xc2 = 7264, Yc2 = 38976, Kind = 18, Secs = 1 },
    new Door { Xc1 = 21616, Yc1 = 25728, Xc2 = 11392, Yc2 = 36224, Kind = 3, Secs = 1 },
    new Door { Xc1 = 21648, Yc1 = 17824, Xc2 = 7312, Yc2 = 39072, Kind = 11, Secs = 1 },
    new Door { Xc1 = 22000, Yc1 = 21216, Xc2 = 5856, Yc2 = 33760, Kind = 10, Secs = 1 },
    new Door { Xc1 = 22208, Yc1 = 21440, Xc2 = 7104, Yc2 = 33984, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22208, Yc1 = 21568, Xc2 = 6592, Yc2 = 34112, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22256, Yc1 = 20896, Xc2 = 6640, Yc2 = 33440, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22272, Yc1 = 21056, Xc2 = 7664, Yc2 = 33600, Kind = 14, Secs = 1 },
    new Door { Xc1 = 22288, Yc1 = 21568, Xc2 = 7184, Yc2 = 34368, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22320, Yc1 = 21248, Xc2 = 6736, Yc2 = 33792, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22320, Yc1 = 21376, Xc2 = 7216, Yc2 = 33920, Kind = 14, Secs = 1 },
    new Door { Xc1 = 22352, Yc1 = 20896, Xc2 = 7264, Yc2 = 33440, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22352, Yc1 = 21088, Xc2 = 8272, Yc2 = 33632, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22368, Yc1 = 21440, Xc2 = 8288, Yc2 = 33984, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22368, Yc1 = 21568, Xc2 = 7776, Yc2 = 34112, Kind = 13, Secs = 1 },
    new Door { Xc1 = 22624, Yc1 = 23872, Xc2 = 7264, Yc2 = 39488, Kind = 18, Secs = 1 },
    new Door { Xc1 = 22672, Yc1 = 23968, Xc2 = 7312, Yc2 = 40096, Kind = 11, Secs = 1 },
    new Door { Xc1 = 22720, Yc1 = 11872, Xc2 = 2752, Yc2 = 34912, Kind = 18, Secs = 2 },
    new Door { Xc1 = 22880, Yc1 = 28480, Xc2 = 8800, Yc2 = 39488, Kind = 18, Secs = 1 },
    new Door { Xc1 = 22928, Yc1 = 28576, Xc2 = 8848, Yc2 = 40096, Kind = 11, Secs = 1 },
    new Door { Xc1 = 22944, Yc1 = 26464, Xc2 = 10912, Yc2 = 35680, Kind = 15, Secs = 1 },
    new Door { Xc1 = 23008, Yc1 = 22656, Xc2 = 10192, Yc2 = 38528, Kind = 10, Secs = 1 },
    new Door { Xc1 = 24176, Yc1 = 6752, Xc2 = 9600, Yc2 = 39520, Kind = 3, Secs = 1 },
    new Door { Xc1 = 24256, Yc1 = 10592, Xc2 = 4544, Yc2 = 35680, Kind = 18, Secs = 2 },
    new Door { Xc1 = 24672, Yc1 = 29248, Xc2 = 6496, Yc2 = 40000, Kind = 18, Secs = 1 },
    new Door { Xc1 = 24720, Yc1 = 29344, Xc2 = 6544, Yc2 = 40096, Kind = 11, Secs = 1 },
    new Door { Xc1 = 24816, Yc1 = 12992, Xc2 = 9712, Yc2 = 35776, Kind = 3, Secs = 1 },
    new Door { Xc1 = 25792, Yc1 = 6240, Xc2 = 960, Yc2 = 34400, Kind = 18, Secs = 2 },
    new Door { Xc1 = 25952, Yc1 = 23872, Xc2 = 8032, Yc2 = 39488, Kind = 18, Secs = 1 },
    new Door { Xc1 = 26000, Yc1 = 23968, Xc2 = 8080, Yc2 = 39072, Kind = 11, Secs = 1 },
    new Door { Xc1 = 26048, Yc1 = 6688, Xc2 = 1200, Yc2 = 34880, Kind = 9, Secs = 2 },
    new Door { Xc1 = 26224, Yc1 = 10848, Xc2 = 11136, Yc2 = 39520, Kind = 3, Secs = 1 },
    new Door { Xc1 = 26624, Yc1 = 7008, Xc2 = 10992, Yc2 = 36960, Kind = 9, Secs = 1 },
    new Door { Xc1 = 27472, Yc1 = 17280, Xc2 = 10320, Yc2 = 36224, Kind = 13, Secs = 1 },
    new Door { Xc1 = 27616, Yc1 = 31872, Xc2 = 11216, Yc2 = 38528, Kind = 10, Secs = 1 },
    new Door { Xc1 = 27760, Yc1 = 11872, Xc2 = 10368, Yc2 = 39520, Kind = 3, Secs = 1 },
    new Door { Xc1 = 28000, Yc1 = 26688, Xc2 = 8032, Yc2 = 39488, Kind = 18, Secs = 1 },
    new Door { Xc1 = 28048, Yc1 = 26784, Xc2 = 8080, Yc2 = 39584, Kind = 11, Secs = 1 },
    new Door { Xc1 = 28384, Yc1 = 21120, Xc2 = 12752, Yc2 = 38528, Kind = 10, Secs = 1 },
];

    // Rectangles of the world with a meaning: where monsters appear, peaceful places, the carriers.
    public static readonly Extent[] InitialExtents = [
    new Extent { X1 = 2118, Y1 = 27237, X2 = 2618, Y2 = 27637, Etype = 70, V1 = 0, V2 = 1, V3 = 11 },
    new Extent { X1 = 0, Y1 = 0, X2 = 0, Y2 = 0, Etype = 70, V1 = 0, V2 = 1, V3 = 5 },
    new Extent { X1 = 6749, Y1 = 34951, X2 = 7249, Y2 = 35351, Etype = 70, V1 = 0, V2 = 1, V3 = 10 },
    new Extent { X1 = 4063, Y1 = 34819, X2 = 4909, Y2 = 35125, Etype = 53, V1 = 4, V2 = 1, V3 = 6 },
    new Extent { X1 = 9563, Y1 = 33883, X2 = 10144, Y2 = 34462, Etype = 60, V1 = 1, V2 = 1, V3 = 9 },
    new Extent { X1 = 22945, Y1 = 5597, X2 = 23225, Y2 = 5747, Etype = 61, V1 = 3, V2 = 2, V3 = 4 },
    new Extent { X1 = 10820, Y1 = 35646, X2 = 10877, Y2 = 35670, Etype = 83, V1 = 1, V2 = 1, V3 = 0 },
    new Extent { X1 = 19596, Y1 = 17123, X2 = 19974, Y2 = 17401, Etype = 48, V1 = 8, V2 = 8, V3 = 2 },
    new Extent { X1 = 19400, Y1 = 17034, X2 = 20240, Y2 = 17484, Etype = 80, V1 = 4, V2 = 20, V3 = 0 },
    new Extent { X1 = 9216, Y1 = 33280, X2 = 12544, Y2 = 35328, Etype = 52, V1 = 3, V2 = 1, V3 = 8 },
    new Extent { X1 = 5272, Y1 = 33300, X2 = 6112, Y2 = 34200, Etype = 81, V1 = 0, V2 = 1, V3 = 0 },
    new Extent { X1 = 11712, Y1 = 37350, X2 = 12416, Y2 = 38020, Etype = 82, V1 = 0, V2 = 1, V3 = 0 },
    new Extent { X1 = 2752, Y1 = 33300, X2 = 8632, Y2 = 35400, Etype = 80, V1 = 0, V2 = 1, V3 = 0 },
    new Extent { X1 = 10032, Y1 = 35550, X2 = 12976, Y2 = 40270, Etype = 80, V1 = 0, V2 = 1, V3 = 0 },
    new Extent { X1 = 4712, Y1 = 38100, X2 = 10032, Y2 = 40350, Etype = 80, V1 = 0, V2 = 1, V3 = 0 },
    new Extent { X1 = 21405, Y1 = 25583, X2 = 21827, Y2 = 26028, Etype = 60, V1 = 1, V2 = 1, V3 = 7 },
    new Extent { X1 = 6156, Y1 = 12755, X2 = 12316, Y2 = 15905, Etype = 7, V1 = 1, V2 = 8, V3 = 0 },
    new Extent { X1 = 5140, Y1 = 34860, X2 = 6260, Y2 = 37260, Etype = 8, V1 = 1, V2 = 8, V3 = 0 },
    new Extent { X1 = 660, Y1 = 33510, X2 = 2060, Y2 = 34560, Etype = 8, V1 = 1, V2 = 8, V3 = 0 },
    new Extent { X1 = 18687, Y1 = 15338, X2 = 19211, Y2 = 16136, Etype = 80, V1 = 0, V2 = 1, V3 = 0 },
    new Extent { X1 = 16953, Y1 = 18719, X2 = 20240, Y2 = 17484, Etype = 3, V1 = 1, V2 = 3, V3 = 0 },
    new Extent { X1 = 20593, Y1 = 18719, X2 = 23113, Y2 = 22769, Etype = 3, V1 = 1, V2 = 3, V3 = 0 },
    new Extent { X1 = 0, Y1 = 0, X2 = 32767, Y2 = 40959, Etype = 3, V1 = 1, V2 = 8, V3 = 0 },
];

    // The stone ring positions, as sector x, y pairs.
    public static readonly byte[] StoneRingSectors = [54, 43, 71, 77, 78, 102, 66, 121, 12, 85, 79, 40, 107, 38, 73, 21, 12, 26, 26, 53, 84, 60];

    // Every item: its picture, where it sits on the item list, how many show, and its name.
    public static readonly InvItem[] ItemList = [
    new InvItem { ImageNumber = 12, Xoff = 10, Yoff = 0, Ydelta = 0, ImgOff = 0, ImgHeight = 8, Maxshown = 1, Name = "Dirk" },
    new InvItem { ImageNumber = 9, Xoff = 10, Yoff = 10, Ydelta = 0, ImgOff = 0, ImgHeight = 8, Maxshown = 1, Name = "Mace" },
    new InvItem { ImageNumber = 8, Xoff = 10, Yoff = 20, Ydelta = 0, ImgOff = 0, ImgHeight = 8, Maxshown = 1, Name = "Sword" },
    new InvItem { ImageNumber = 10, Xoff = 10, Yoff = 30, Ydelta = 0, ImgOff = 0, ImgHeight = 8, Maxshown = 1, Name = "Bow" },
    new InvItem { ImageNumber = 17, Xoff = 10, Yoff = 40, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Magic Wand" },
    new InvItem { ImageNumber = 27, Xoff = 10, Yoff = 50, Ydelta = 0, ImgOff = 0, ImgHeight = 8, Maxshown = 1, Name = "Golden Lasso" },
    new InvItem { ImageNumber = 23, Xoff = 10, Yoff = 60, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Sea Shell" },
    new InvItem { ImageNumber = 27, Xoff = 10, Yoff = 70, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Sun Stone" },
    new InvItem { ImageNumber = 3, Xoff = 30, Yoff = 0, Ydelta = 3, ImgOff = 7, ImgHeight = 1, Maxshown = 45, Name = "Arrows" },
    new InvItem { ImageNumber = 18, Xoff = 50, Yoff = 0, Ydelta = 9, ImgOff = 0, ImgHeight = 8, Maxshown = 15, Name = "Blue Stone" },
    new InvItem { ImageNumber = 19, Xoff = 65, Yoff = 0, Ydelta = 6, ImgOff = 0, ImgHeight = 5, Maxshown = 23, Name = "Green Jewel" },
    new InvItem { ImageNumber = 22, Xoff = 80, Yoff = 0, Ydelta = 8, ImgOff = 0, ImgHeight = 7, Maxshown = 17, Name = "Glass Vial" },
    new InvItem { ImageNumber = 21, Xoff = 95, Yoff = 0, Ydelta = 7, ImgOff = 0, ImgHeight = 6, Maxshown = 20, Name = "Crystal Orb" },
    new InvItem { ImageNumber = 23, Xoff = 110, Yoff = 0, Ydelta = 10, ImgOff = 0, ImgHeight = 9, Maxshown = 14, Name = "Bird Totem" },
    new InvItem { ImageNumber = 17, Xoff = 125, Yoff = 0, Ydelta = 6, ImgOff = 0, ImgHeight = 5, Maxshown = 23, Name = "Gold Ring" },
    new InvItem { ImageNumber = 24, Xoff = 140, Yoff = 0, Ydelta = 10, ImgOff = 0, ImgHeight = 9, Maxshown = 14, Name = "Jade Skull" },
    new InvItem { ImageNumber = 25, Xoff = 160, Yoff = 0, Ydelta = 5, ImgOff = 0, ImgHeight = 5, Maxshown = 25, Name = "Gold Key" },
    new InvItem { ImageNumber = 25, Xoff = 172, Yoff = 0, Ydelta = 5, ImgOff = 8, ImgHeight = 5, Maxshown = 25, Name = "Green Key" },
    new InvItem { ImageNumber = 114, Xoff = 184, Yoff = 0, Ydelta = 5, ImgOff = 0, ImgHeight = 5, Maxshown = 25, Name = "Blue Key" },
    new InvItem { ImageNumber = 114, Xoff = 196, Yoff = 0, Ydelta = 5, ImgOff = 8, ImgHeight = 5, Maxshown = 25, Name = "Red Key" },
    new InvItem { ImageNumber = 26, Xoff = 208, Yoff = 0, Ydelta = 5, ImgOff = 0, ImgHeight = 5, Maxshown = 25, Name = "Grey Key" },
    new InvItem { ImageNumber = 26, Xoff = 220, Yoff = 0, Ydelta = 5, ImgOff = 8, ImgHeight = 5, Maxshown = 25, Name = "White Key" },
    new InvItem { ImageNumber = 11, Xoff = 0, Yoff = 80, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Talisman" },
    new InvItem { ImageNumber = 19, Xoff = 0, Yoff = 90, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Rose" },
    new InvItem { ImageNumber = 20, Xoff = 0, Yoff = 100, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Fruit" },
    new InvItem { ImageNumber = 21, Xoff = 232, Yoff = 0, Ydelta = 10, ImgOff = 8, ImgHeight = 8, Maxshown = 5, Name = "Gold Statue" },
    new InvItem { ImageNumber = 22, Xoff = 0, Yoff = 110, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Book" },
    new InvItem { ImageNumber = 8, Xoff = 14, Yoff = 80, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Herb" },
    new InvItem { ImageNumber = 9, Xoff = 14, Yoff = 90, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Writ" },
    new InvItem { ImageNumber = 10, Xoff = 14, Yoff = 100, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Bone" },
    new InvItem { ImageNumber = 12, Xoff = 14, Yoff = 110, Ydelta = 0, ImgOff = 8, ImgHeight = 8, Maxshown = 1, Name = "Shard" },
    new InvItem { ImageNumber = 0, Xoff = 0, Yoff = 0, Ydelta = 0, ImgOff = 0, ImgHeight = 0, Maxshown = 2, Name = "2 Gold Pieces" },
    new InvItem { ImageNumber = 0, Xoff = 0, Yoff = 0, Ydelta = 0, ImgOff = 0, ImgHeight = 0, Maxshown = 5, Name = "5 Gold Pieces" },
    new InvItem { ImageNumber = 0, Xoff = 0, Yoff = 0, Ydelta = 0, ImgOff = 0, ImgHeight = 0, Maxshown = 10, Name = "10 Gold Pieces" },
    new InvItem { ImageNumber = 0, Xoff = 0, Yoff = 0, Ydelta = 0, ImgOff = 0, ImgHeight = 0, Maxshown = 100, Name = "100 Gold Pieces" },
    new InvItem { ImageNumber = 0, Xoff = 0, Yoff = 0, Ydelta = 0, ImgOff = 0, ImgHeight = 0, Maxshown = 0, Name = "quiver of arrows" },
];

    // The mouse pointer sprite.
    // The pointer sprite's image data (20 words, packed as 32 bit values).
    public static readonly uint[] PointerSpriteData = [0x0, 0x80060000, 0x60010002, 0x40012000, 0x10030000, 0x8FE0001, 0x581007F, 0x3000081, 0x6010101, 0xC800201, 0x8410401, 0x8200401, 0x8110401, 0x80C0401, 0x880F0401, 0x98074400, 0x75560FFC, 0x0, 0x0, 0x0];

    // The text panel colours.
    public static readonly ushort[] TextColors = [0x000, 0xFFF, 0xC00, 0xF60, 0x00F, 0xC0F, 0x090, 0xFF0, 0xF90, 0xF0C, 0xA50, 0xFDB, 0xEB7, 0xCCC, 0x888, 0x444, 0x000, 0xDB0, 0x740, 0xC70];

    // The intro page colours.
    public static readonly ushort[] IntroColors = [0x000, 0xFFF, 0xE00, 0xA00, 0xD80, 0xEC0, 0x390, 0x021, 0xEEB, 0xEDA, 0xEEA, 0xCB8, 0xA95, 0x973, 0x840, 0x620, 0xA52, 0xC74, 0xD96, 0xFCA, 0x449, 0x444, 0xDC9, 0x668, 0x33F, 0x888, 0xA60, 0xAAF, 0xBBB, 0xCCF, 0xDDD, 0xEEE];

    // All black, for fading out.
    public static readonly ushort[] BlackColors = new ushort[32];

    public static readonly byte[] Label1 = Latin1("ItemsMagicTalk Buy  Game \u0000");

    public static readonly byte[] Label2 = Latin1("List Take Look Use  Give \u0000");

    public static readonly byte[] Label3 = Latin1("Yell Say  Ask  \u0000");

    public static readonly byte[] Label4 = Latin1("PauseMusicSoundQuit Load \u0000");

    public static readonly byte[] Label5 = Latin1("Food ArrowVial Mace SwordBow  Totem\u0000");

    public static readonly byte[] Label6 = Latin1("StoneJewelVial Orb  TotemRing Skull\u0000");

    public static readonly byte[] Label7 = Latin1("Dirk Mace SwordBow  Wand LassoShellKey  Sun  Book \u0000");

    public static readonly byte[] Label8 = Latin1("Save Exit \u0000");

    public static readonly byte[] Label9 = Latin1("Gold GreenBlue Red  Grey White\u0000");

    public static readonly byte[] LabelA = Latin1("Gold Book Writ Bone \u0000");

    public static readonly byte[] LabelB = Latin1("  A    B    C    D    E    F    G    H  \u0000");

    // The colour of each key in the key menu.
    public static readonly sbyte[] KeyColors = [8, 6, 4, 2, 14, 1];

    // The ten menus at the start of the game: labels, entry count, colour, and each entry's state (see
    // `Menu`). The game works on its own copy, `Game::menus`.
    public static readonly Menu[] InitialMenus = [
    new Menu { LabelList = Label2, Num = 10, Color = 6, Enabled = [3, 2, 2, 2, 2, 10, 10, 10, 10, 10, 0, 0] },
    new Menu { LabelList = Label6, Num = 12, Color = 5, Enabled = [2, 3, 2, 2, 2, 8, 8, 8, 8, 8, 8, 8] },
    new Menu { LabelList = Label3, Num = 8, Color = 9, Enabled = [2, 2, 3, 2, 2, 10, 10, 10, 0, 0, 0, 0] },
    new Menu { LabelList = Label5, Num = 12, Color = 10, Enabled = [2, 2, 2, 3, 2, 10, 10, 10, 10, 10, 10, 10] },
    new Menu { LabelList = Label4, Num = 10, Color = 2, Enabled = [2, 2, 2, 2, 3, 6, 7, 7, 10, 10, 0, 0] },
    new Menu { LabelList = Label8, Num = 7, Color = 0, Enabled = [2, 2, 2, 2, 2, 10, 10, 0, 0, 0, 0, 0] },
    new Menu { LabelList = Label9, Num = 11, Color = 8, Enabled = [2, 2, 2, 2, 2, 10, 10, 10, 10, 10, 10, 0] },
    new Menu { LabelList = LabelA, Num = 9, Color = 10, Enabled = [2, 2, 2, 2, 2, 10, 0, 0, 0, 0, 0, 0] },
    new Menu { LabelList = Label7, Num = 10, Color = 8, Enabled = [10, 10, 10, 10, 10, 10, 10, 10, 10, 0, 10, 10] },
    new Menu { LabelList = LabelB, Num = 10, Color = 5, Enabled = [10, 10, 10, 10, 10, 10, 10, 10, 0, 0, 0, 0] },
];

    // Keyboard shortcuts: the key, the menu and the entry it picks.
    public static readonly Letter[] MenuShortcuts = [
    new Letter { Key = 73, Menu = 0, Choice = 5 },
    new Letter { Key = 84, Menu = 0, Choice = 6 },
    new Letter { Key = 63, Menu = 0, Choice = 7 },
    new Letter { Key = 85, Menu = 0, Choice = 8 },
    new Letter { Key = 71, Menu = 0, Choice = 9 },
    new Letter { Key = 89, Menu = 2, Choice = 5 },
    new Letter { Key = 83, Menu = 2, Choice = 6 },
    new Letter { Key = 65, Menu = 2, Choice = 7 },
    new Letter { Key = 32, Menu = 4, Choice = 5 },
    new Letter { Key = 77, Menu = 4, Choice = 6 },
    new Letter { Key = 70, Menu = 4, Choice = 7 },
    new Letter { Key = 81, Menu = 4, Choice = 8 },
    new Letter { Key = 76, Menu = 4, Choice = 9 },
    new Letter { Key = 79, Menu = 3, Choice = 5 },
    new Letter { Key = 82, Menu = 3, Choice = 6 },
    new Letter { Key = 56, Menu = 3, Choice = 7 },
    new Letter { Key = 67, Menu = 3, Choice = 8 },
    new Letter { Key = 87, Menu = 3, Choice = 9 },
    new Letter { Key = 66, Menu = 3, Choice = 10 },
    new Letter { Key = 69, Menu = 3, Choice = 11 },
    new Letter { Key = 86, Menu = 5, Choice = 5 },
    new Letter { Key = 88, Menu = 5, Choice = 6 },
    new Letter { Key = 10, Menu = 1, Choice = 5 },
    new Letter { Key = 11, Menu = 1, Choice = 6 },
    new Letter { Key = 12, Menu = 1, Choice = 7 },
    new Letter { Key = 13, Menu = 1, Choice = 8 },
    new Letter { Key = 14, Menu = 1, Choice = 9 },
    new Letter { Key = 15, Menu = 1, Choice = 10 },
    new Letter { Key = 16, Menu = 1, Choice = 11 },
    new Letter { Key = 49, Menu = 8, Choice = 0 },
    new Letter { Key = 50, Menu = 8, Choice = 1 },
    new Letter { Key = 51, Menu = 8, Choice = 2 },
    new Letter { Key = 52, Menu = 8, Choice = 3 },
    new Letter { Key = 53, Menu = 8, Choice = 4 },
    new Letter { Key = 54, Menu = 8, Choice = 5 },
    new Letter { Key = 55, Menu = 8, Choice = 6 },
    new Letter { Key = 75, Menu = 8, Choice = 7 },
];

    // The brothers' names.
    public static readonly string[] BrotherNames = ["Julian", "Phillip", "Kevin"];

    // The disk blocks each of the ten map regions needs.
    public static readonly Need[] RegionDiskBlocks = [
    new Need { Image = [320, 480, 520, 560], Terra1 = 0, Terra2 = 1, Sector = 32, Region = 160, Setchar = 22 },
    new Need { Image = [320, 360, 400, 440], Terra1 = 2, Terra2 = 3, Sector = 32, Region = 160, Setchar = 21 },
    new Need { Image = [320, 360, 520, 560], Terra1 = 2, Terra2 = 1, Sector = 32, Region = 168, Setchar = 22 },
    new Need { Image = [320, 360, 400, 440], Terra1 = 2, Terra2 = 3, Sector = 32, Region = 168, Setchar = 21 },
    new Need { Image = [320, 480, 520, 600], Terra1 = 0, Terra2 = 4, Sector = 32, Region = 176, Setchar = 0 },
    new Need { Image = [320, 280, 240, 200], Terra1 = 5, Terra2 = 6, Sector = 32, Region = 176, Setchar = 23 },
    new Need { Image = [320, 640, 520, 600], Terra1 = 7, Terra2 = 4, Sector = 32, Region = 184, Setchar = 0 },
    new Need { Image = [320, 280, 240, 200], Terra1 = 5, Terra2 = 6, Sector = 32, Region = 184, Setchar = 24 },
    new Need { Image = [680, 720, 800, 840], Terra1 = 8, Terra2 = 9, Sector = 96, Region = 192, Setchar = 0 },
    new Need { Image = [680, 760, 800, 840], Terra1 = 10, Terra2 = 9, Sector = 96, Region = 192, Setchar = 0 },
];

    // The font the text panel asks for.
    // The instrument waveforms for the music, as pairs of values.
    public static readonly short[] InstrumentWaves = [0x0000, 0x0000, 0x0000, 0x0000, 0x0005, 0x0202, 0x0101, 0x0103, 0x0004, 0x0504, 0x0100, 0x0500];

    // The first sprite image of each facing (16 entries: walking, then fighting).
    public static readonly sbyte[] FacingFrameOffsets = [16, 16, 24, 24, 0, 0, 8, 8, 56, 56, 68, 68, 32, 32, 44, 44];

    // The brothers' starting stats and their item lists.
    public static readonly Bro[] BrotherStats = [
    new Bro { Brave = 35, Luck = 20, Kind = 15, Wealth = 20 },
    new Bro { Brave = 20, Luck = 35, Kind = 15, Wealth = 15 },
    new Bro { Brave = 15, Luck = 20, Kind = 35, Wealth = 10 },
    ];
}
