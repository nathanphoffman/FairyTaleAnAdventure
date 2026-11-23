using FaeryTale.Game.Data;

namespace FaeryTale.Game.Render;

// Choosing which picture of the sprite sheet an actor is drawn with, and how high it is, and the
// weapon an actor holds: which pass it is drawn in and which picture.
internal static partial class SpriteImages
{
    // Actor kinds as sprite sheet numbers.
    internal const short PHIL = 0;
    internal const short OBJECTS = 1;
    internal const short ENEMY = 2;
    internal const short RAFT = 3;
    internal const short CARRIER = 5;

    // Enemy race 8 (the spectre) and 4 (snakes) look different.
    private const byte RACE_SPECTRE = 8;
    private const byte RACE_SNAKE = 4;

    /// A sprite is 32 pixels from the top of its box to the ground line.
    private const short SPRITE_GROUND_OFFSET = 32;

    /// A falling actor with fewer tactic steps than this still shows as an enemy figure.
    private const sbyte FALL_AS_ENEMY_STEPS = 16;

    // The first snake picture, and the pairing of left/right facing pictures.
    private const short SNAKE_FIRST_IMAGE = 0x24;
    private const short IMAGE_PAIR_BIT = 1;

    // The swan's actor file number (it is drawn as the raft when nobody rides).
    private const short SWAN_FILE = 11;
    private const short RIDING_SWAN = 11;
    private const short RAFT_IMAGE = 1;

    // Objects whose picture is only 8 lines high: 0x1b, 8 to 12, 25, 26, 0x11 to 0x17, and any
    // with bit 7 set.
    private const sbyte SMALL_OBJECT_LINES = 8;
    private const short OBJECT_FLAG_SMALL = 128;

    // Sinking: the hero on the swan sits 16 lines higher; shallow water (depth 2) hides 10 lines;
    // beyond 29 the actor is under the surface and only bubbles show, drawn 27 lines down and 7 high.
    private const short SWAN_LIFT = 16;
    private const short SHALLOW_DEPTH = 2;
    private const short SHALLOW_HIDDEN_LINES = 10;
    private const short UNDER_SURFACE_DEPTH = 29;
    private const short BUBBLES_DROP = 27;
    private const short BUBBLES_HEIGHT = 7;

    // Sprite numbers (object images).
    private const short OBJ_BURNING = 0x58;
    private const short OBJ_SUNK = 97;

    // Weapons.
    private const short OBJ_BOW_UP = 30;
    private const short OBJ_BOW_MID = 0x53;
    private const short OBJ_BOW_DOWN = 0x51;
    private const short OBJ_ARROW_BASE = 103;
    private const sbyte WEAPON_BOW = 4;

    /// Weapon numbers from 1 to 7 are shown held; the others (the wand and missiles) are not.
    private const sbyte LAST_HELD_WEAPON = 7;

    // Bit 2 of the facing (and of the facing shifted by 2) says whether the weapon is drawn before
    // or after the body.
    private const int FACING_LEFT_BIT = 4;
    private const int FACING_PICK_OFFSET = 2;

    /// Extents whose type is above this are peaceful, and the weapon is drawn even when resting.
    private const ushort PEACEFUL_XTYPE = 80;

    /// The bow's drawn image changes every 8 frames: bit 0 up, bit 1 mid, else down.
    private const short BOW_PHASE_FRAMES = 8;

    // A fireball facing east (2) is drawn this many lines higher.
    private const sbyte FIREBALL_FACING_EAST = 2;
    private const short FIREBALL_EAST_LIFT = 6;
    private const short WEAPON_FIREBALL = 5;

    /// The first frame of the bow's own drawing table; below this frames come from BowX/BowY.
    private const short BOW_FRAMES = 32;

    private static readonly (sbyte Weapon, short Base)[] WEAPON_SPRITE_BASE = [(2, 32), (3, 48), (1, 64), (4, 0)];

    internal static StateEntry StateEntryFor(short inum) => Tables.Entry(Tables.AnimationFrames, inum);

    internal static short ByteOf(sbyte[] table, short inum) => inum >= 0 && inum < table.Length ? table[inum] : (short)0;

    /// Where the images of one kind of sprite are (SeqList entry atype).
    internal static SeqInfo SpriteSheet(Video video, short atype) =>
        atype >= 0 && atype < video.Graphics.SeqList.Length ? video.Graphics.SeqList[atype] : default;

    /// Picks the image for this pass: the weapon's (passmode) or the actor's own, and where its
    /// top left corner is on the screen. Null means nothing is drawn.
    internal static Sprite? ChooseImage(Actors actors, Hero hero, Shape actor, int slot, bool passmode)
    {
        short inum = actor.Index;
        short xStart = (short)actor.RelX;
        short yStart = (short)actor.RelY;
        short ground = unchecked((short)(yStart + SPRITE_GROUND_OFFSET));
        Sprite blank = new() { Atype = OBJECTS, Inum = inum, XStart = xStart, YStart = yStart, Ground = ground };

        if (passmode)
        {
            return WeaponImage(actor, blank);
        }

        if (BodyImage(actors, hero, actor, slot, inum) is not { } body)
        {
            return null;
        }

        blank.Atype = body.Atype;
        blank.Inum = body.Inum;

        return blank;
    }

    /// The height of an image in lines: the sheet's, except for some small objects.
    internal static sbyte ImageHeight(short atype, short inum, sbyte sheetHeight)
    {
        bool smallObject = inum == 0x1b || inum == 25 || inum == 26
            || (inum >= 8 && inum <= 12)
            || (inum >= 0x11 && inum <= 0x17)
            || (inum & OBJECT_FLAG_SMALL) != 0;

        return atype == OBJECTS && smallObject ? SMALL_OBJECT_LINES : sheetHeight;
    }

    // The actor's own sprite sheet and image for this animation frame. Null means nothing is
    // drawn (a burnt body, or an object with no image).
    private static (short Atype, short Inum)? BodyImage(Actors actors, Hero hero, Shape actor, int slot, short inum)
    {
        short atype = actor.Kind;

        if ((atype == ENEMY && actor.Race != RACE_SPECTRE) || atype == PHIL)
        {
            if (actors.FieryDeath != 0 && actor.Environ > 0)
            {
                if (actor.State == ActorState.DEAD)
                {
                    return null;
                }

                if (actor.State == ActorState.DYING)
                {
                    // A body burning in the lava: nothing more to work out.
                    return (OBJECTS, OBJ_BURNING);
                }
            }
            else if (actor.State == ActorState.FALL)
            {
                atype = actor.Tactic < FALL_AS_ENEMY_STEPS ? ENEMY : OBJECTS;
            }
            else
            {
                inum = StateEntryFor(inum).Figure;
            }

            inum = PickOfPair(actor, slot, inum);
        }

        if (atype == OBJECTS && actor.Race == 0)
        {
            return null;
        }

        if (atype == CARRIER && hero.Riding == 0 && actors.LoadedActorFile == SWAN_FILE)
        {
            return (RAFT, RAFT_IMAGE);
        }

        return (atype, inum);
    }
}
