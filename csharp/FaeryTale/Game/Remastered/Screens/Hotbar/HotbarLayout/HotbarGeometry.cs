using FaeryTale.Platform;

namespace FaeryTale.Game.Remastered;

/// Where the hotbar goes for a window of some size: the size of a square, the board of wood under the text
/// panel that the bars grow out of, and the middle line the squares sit on.
public sealed record HotbarGeometry(int SlotSize, int Scale, int BoardLeft, int BoardRight, int BoardTop, int WindowWidth, int WindowHeight, int CenterY);
