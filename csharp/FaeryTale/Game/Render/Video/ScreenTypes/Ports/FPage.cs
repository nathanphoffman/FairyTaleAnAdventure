using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// One of the two display pages being drawn and shown (struct fpage). The game draws on one while
/// the other is on screen, then swaps them. It also remembers what is drawn there so it can be
/// erased later, and the witch's beam position.
public sealed class FPage
{
    /// Which of the screen's Ras entries (the bitmap and its scroll offset) this page shows.
    public int Ras;
    public int IsvX;
    public int IsvY;
    public short ObCount;

    /// How each sprite drawn on this page was drawn, to undo them.
    public List<SShape> ShapeQueue = [];

    /// Where this page's saved backgrounds start.
    public Loc BackSave = Loc.NONE;
    public int SaveUsed;
    public short WitchX;
    public short WitchY;
    public short WitchDir;
    public short WFlag;
}
