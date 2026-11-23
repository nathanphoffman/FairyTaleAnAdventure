using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Land;

/// Items lying in the world and the people standing around: turning the object lists of the
/// current region into actors, and changing them.
///
/// How the world's objects work: each region has a list of objects and there is one global list
/// for things that follow the story, like the dead brothers' bags and the spectre. Each entry has
/// a position, an id and a state. Only the entries near the screen become actors in the actor
/// list, so the game never handles more than 20 actors at once:
/// - people (state 3 or 4) go in slots starting at 2;
/// - ordinary items go after the monsters, counted by Actors.Drawn.
///
/// The actor's Vitality field is borrowed to remember which list entry it came from (the entry
/// number, plus 0x80 for the global list), so ChangeObject can find the entry again later.
public static partial class Objects
{
    /// What a chest turns into when opened.
    private const sbyte OPENED_CHEST = 0x1d;

    /// The witch's set figure id.
    private const sbyte WITCH = 9;

    // Object states (ObStat).
    private const sbyte STAT_HIDDEN = 0;
    private const sbyte STAT_SHOWN = 1;
    private const sbyte STAT_TAKEN = 2;
    private const sbyte STAT_PERSON = 3;
    private const sbyte STAT_DEAD_PERSON = 4;
    private const sbyte STAT_RAFT_ITEM = 5;
    private const sbyte STAT_SECOND_ITEM = 6;

    // Actors 0 and 1 are the hero and the raft; objects and people start after the carrier slots.
    private const short FIRST_OBJECT_ACTOR = 2;
    private const short MAX_ACTORS = 20;

    /// Slots above this are monsters (the hero, raft and first person come before).
    private const short FIRST_MONSTER_SLOT = 3;

    /// Random treasures added to a region the first time it is visited.
    private const int RANDOM_TREASURES = 10;

    // Offset of the global object list's ids in Vitality, and the bit marking "global list".
    private const short OBJECT_INDEX_MASK = 0x7f;
    private const short GLOBAL_LIST_BIT = 0x80;

    /// How far from the old screen (in pixels) something is looked at: low and high limits across
    /// and down.
    private readonly record struct Reach(short XLow, short XHigh, short YLow, short YHigh);

    // The original: people are looked at a little beyond the screen, and shown on it.
    private static readonly Reach ORIGINAL_PERSON_LOAD = new(-100, 400, -100, 300);
    private static readonly Reach ORIGINAL_SHOWN = new(-20, 340, -20, 190);

    // Remastered and Reimagined show more than the old screen (zoomed out the window shows several
    // times as much), so people and things come out further: well past where enemies come from
    // (150 to 213 pixels from the hero), about 460 pixels either side of the hero and 340 up and down.
    private static readonly Reach WIDE_PERSON_LOAD = new(-400, 700, -350, 500);
    private static readonly Reach WIDE_SHOWN = new(-300, 620, -260, 420);
    private const short SPRITE_CENTER_OFFSET = 8;

    /// People stand this much higher than their position.
    private const short PERSON_RAISE = 18;

    /// Actor Race for a person is their id plus this.
    private const int PERSON_RACE_BASE = 0x80;

    /// A dropped item lands this far below the actor's position.
    private const ushort DROP_OFFSET_Y = 10;

    // A region is 16384 pixels wide and 8192 high (two columns of four rows make up the map); the
    // random treasures are placed anywhere in it. Rows are 8192 apart, and the region number
    // counts them in steps of 2, so a step is 0x1000 per unit.
    private const int REGION_WIDTH = 0x4000;
    private const int REGION_HEIGHT = 0x2000;
    private const int REGION_ROW_STEP = 0x1000;

    /// The number of different random treasures.
    private const int TREASURE_KINDS = 16;

    /// Turns the global and current region's object lists into actors. Called when the screen
    /// changes (new region, scrolling far, after loading a game).
    public static void SpawnObjectActors(Data.Game game, IOutput output)
    {
        game.Actors.FreeSlot = FIRST_OBJECT_ACTOR;
        ChooseSetfigFile(game);
        AddObjectsNear(game, output, ObjectList.Global, ObjectTables.Glbobs, GLOBAL_LIST_BIT);

        int region = game.World.Region;

        AddObjectsNear(game, output, new ObjectList(region), game.Objects.Used[region], 0);

        if (game.Actors.FreeSlot > FIRST_MONSTER_SLOT)
        {
            game.Actors.Active = game.Actors.FreeSlot;
        }
    }

    /// Drops an object where actor counter stands (slot 0 of the global list is used for this).
    /// That slot is reused for each item the hero or a monster leaves behind.
    public static void LeaveItem(ObjectLists objects, Actors actors, short counter, short item)
    {
        Shape actor = actors.List[counter];

        objects.Global[0].Xc = actor.AbsX;
        objects.Global[0].Yc = (ushort)(actor.AbsY + DROP_OFFSET_Y);
        objects.Global[0].ObId = (sbyte)item;
        objects.Global[0].ObStat = STAT_SHOWN;
    }

    /// Changes the object that actor id stands for: flag 1 = show, 2 = take. A chest opens
    /// instead (its id changes to the opened chest, rather than its state).
    public static void ChangeObject(ObjectLists objects, Actors actors, World world, int identifier, int flag)
    {
        short vitality = actors.List[identifier].Vitality;
        int index = vitality & OBJECT_INDEX_MASK;
        WorldObject[] list = (vitality & GLOBAL_LIST_BIT) != 0 ? objects.Global : objects.Region(world.Region);

        if (list[index].ObId == (sbyte)ObjectTables.CHEST)
        {
            list[index].ObId = OPENED_CHEST;
        }
        else
        {
            list[index].ObStat = (sbyte)flag;
        }
    }

    /// Turns the objects of one list into actors if they are close enough to the screen. f is
    /// added to the object's number to make the actor's Vitality (0x80 marks the global list).
    ///
    /// For each entry: work out where it is on screen (the sprite is 16 wide, so 8 is subtracted
    /// to centre it); people are only loaded if within about 100-400 pixels, and shown if within
    /// the screen; items that are hidden or taken are skipped. A person already in the actor list
    /// at the same place is not added twice (and is marked dead if its actor died).
    public static void AddObjectsNear(Data.Game game, IOutput output, ObjectList which, short length, int f)
    {
        game.Actors.TurtleEggs = 0;
        game.Witch.OnScreen = 0;

        if (game.Objects.Distributed[game.World.Region] == 0 && game.World.NewRegion >= Regions.NO_REGION)
        {
            DistributeTreasure(game);
        }

        for (short index = 0; index < length; index++)
        {
            if (game.Actors.Drawn >= MAX_ACTORS)
            {
                return;
            }

            WorldObject[] list = game.Objects.List(which);
            WorldObject copy = list[index];

            AddObject(game, output, ref copy, index, f);
            list[index] = copy;
        }
    }

    // The people's pictures come in a few files and only one is in memory at a time. With people loaded
    // from so far out, several kinds can be in range at once; so Remastered and Reimagined load the
    // people of one file only: the file the nearest person needs, keeping the one in memory while
    // any of its people is on the old screen (so it does not swap back and forth between two people).
    private static int setfigFileId = -1;

    // Whether an object is inside a reach from the old screen.
    private static bool InReach(Data.Game game, WorldObject item, Reach reach)
    {
        short xStart = (short)(item.Xc - game.World.MapX - SPRITE_CENTER_OFFSET);
        short yStart = (short)(item.Yc - game.World.MapY - SPRITE_CENTER_OFFSET);

        return Inside(xStart, reach.XLow, reach.XHigh) && Inside(yStart, reach.YLow, reach.YHigh);
    }
}
