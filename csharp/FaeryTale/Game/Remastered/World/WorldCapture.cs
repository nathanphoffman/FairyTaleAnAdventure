using FaeryTale.Game.Data;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Remastered;

/// One character or object as the new world drawer sees it: which picture, and where in the world.
public sealed class CapturedSprite
{
    /// Which actor it is (the slot, and 1 for the weapon pass of an actor drawn in two).
    public int Slot;
    public int Pass;
    public byte Race;
    public sbyte Kind;

    /// The sprite sheet (a copy of where it is in memory) and the picture in it.
    public SeqInfo Sheet;
    public short Inum;

    /// The world position of the picture's top left corner, how many of its lines to draw, and how
    /// many lines at the top to leave out.
    public int Left;
    public int Top;
    public int Rows;
    public int SkipRows;

    /// The actor's own position, and how far it moved in the last tick (the previous position minus
    /// this one): drawing it that much back gives the smooth movement between two ticks.
    public int ActorX;
    public int ActorY;
    public double DeltaX;
    public double DeltaY;

    /// The line of the world the actor stands on, for hiding it behind scenery.
    public int GroundY;

    /// Drawn over everything, whatever scenery is in front.
    public bool InFront;

    public Shape? Actor;
}

/// What the new world drawer (Remastered and Reimagined) needs from the game, collected while the
/// old drawing runs: where the sprites are, and where the hero is. The old drawing carries on
/// unchanged, so the game state is exactly the same as without the new drawer.
///
/// A new frame starts every game tick. The positions of the tick before are kept, so a frame in
/// between can be drawn with everything a little way along.
public sealed class WorldCapture
{
    /// How far an actor can move in one tick and still count as walking (more is a jump, such as a
    /// door or a teleport, and is not smoothed).
    private const int MAX_SMOOTH_MOVE = 64;

    private List<CapturedSprite> before = [];

    /// Turned on while a Remastered or Reimagined game is being played.
    public bool Enabled { get; set; }

    /// False while the game shows something else than the world (story pages, the item list, the map).
    public bool WorldVisible { get; set; }

    public Data.Game? Game { get; private set; }

    /// Where the player clicked to walk (a world position, and whether the hero can stand there),
    /// and whether the pointer is over a door: the new drawer shows these itself.
    public ((int X, int Y) World, bool Walkable)? Marker { get; set; }

    public bool DoorHover { get; set; }

    /// The actor number of the person under the pointer, and of the one a conversation bubble is
    /// open for (null for none).
    public int? HoverPerson { get; set; }

    public int? TalkingTo { get; set; }

    /// The pointer is over something that can be picked up: it is a hand.
    public bool ItemHover { get; set; }

    /// The pointer is over an enemy that can be attacked (a red crosshair), and the world position
    /// of the enemy being attacked (a red crosshair over it).
    public bool EnemyHover { get; set; }

    public (int X, int Y)? AttackTarget { get; set; }

    /// The classic buttons that get a red square (what the new controls are doing).
    public PanelButtons LitButtons { get; set; }

    /// Shift is held: the pointer is an eye.
    public bool LookMode { get; set; }

    public List<CapturedSprite> Sprites { get; private set; } = [];

    public (int X, int Y) HeroNow { get; private set; }

    /// How far the hero moved in the last tick, previous minus now.
    public (double X, double Y) HeroDelta { get; private set; }

    /// Counts frames, so the drawer knows when the game has moved on.
    public int Tick { get; private set; }

    private (int X, int Y) heroBefore;
    private bool prepared;

    /// Starts a frame: the sprites of the last one become the "before" and the list starts again.
    public void BeginFrame(Data.Game game)
    {
        if (!Enabled)
        {
            return;
        }

        (before, Sprites) = (Sprites, before);
        Sprites.Clear();
        heroBefore = HeroNow;
        HeroNow = (game.Actors.List[0].AbsX, game.Actors.List[0].AbsY);
        Game = game;
        WorldVisible = true;
        prepared = false;
        Tick++;
    }

    /// Notes a sprite the old drawing has just placed (sprite is in the page's own pixels, which
    /// start at a tile boundary: the map position with its last bits cut off).
    internal void Record(Data.Game game, Video video, Shape actor, int slot, bool weaponPass, Sprite sprite)
    {
        if (!Enabled)
        {
            return;
        }

        short inum = sprite.Inum;
        int skipRows = 0;

        if (sprite.Atype == SpriteImages.OBJECTS && (inum & ObjectMiddleBit) != 0)
        {
            inum &= ObjectNumberMask;
            skipRows = ObjectMiddleStart;
        }

        int originX = game.World.MapX & ~TileXMask;
        int originY = game.World.MapY & ~TileYMask;

        Sprites.Add(new CapturedSprite
        {
            Slot = slot,
            Pass = weaponPass ? 1 : 0,
            Race = actor.Race,
            Kind = actor.Kind,
            Sheet = SpriteImages.SpriteSheet(video, sprite.Atype),
            Inum = inum,
            Left = sprite.XStart + originX,
            Top = sprite.YStart + originY,
            Rows = sprite.YStop - sprite.YStart + 1,
            SkipRows = skipRows,
            ActorX = actor.AbsX,
            ActorY = actor.AbsY,
            GroundY = sprite.Ground + originY,
            InFront = SceneryMasks.DrawnInFront(game.World, game.Hero, actor, slot, sprite),
            Actor = actor,
        });
    }

    /// Works out how far everything moved since the last tick (once per tick, before drawing).
    public void Prepare()
    {
        if (prepared)
        {
            return;
        }

        prepared = true;
        HeroDelta = SmoothMove(heroBefore.X, heroBefore.Y, HeroNow.X, HeroNow.Y);

        foreach (CapturedSprite sprite in Sprites)
        {
            CapturedSprite? last = before.Find(old => old.Slot == sprite.Slot && old.Pass == sprite.Pass && old.Race == sprite.Race && old.Kind == sprite.Kind);

            (sprite.DeltaX, sprite.DeltaY) = last is null ? (0, 0) : SmoothMove(last.ActorX, last.ActorY, sprite.ActorX, sprite.ActorY);
        }
    }

    // Previous minus now, or nothing when the move was too long to be a step.
    private static (double X, double Y) SmoothMove(int fromX, int fromY, int toX, int toY) =>
        Math.Abs(toX - fromX) <= MAX_SMOOTH_MOVE && Math.Abs(toY - fromY) <= MAX_SMOOTH_MOVE ? (fromX - toX, fromY - toY) : (0, 0);

    // The same constants the old drawing uses to place a sprite on a page that is scrolled by tiles.
    private const int TileXMask = 15;
    private const int TileYMask = 31;
    private const short ObjectMiddleBit = 128;
    private const short ObjectNumberMask = 0x7f;
    private const int ObjectMiddleStart = 8;
}
