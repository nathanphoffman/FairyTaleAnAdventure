using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Ui;

/// Taking what lies nearby: gold, fruit, paper, a brother's bones and plain items, and searching
/// dead bodies. Chests, urns and sacks are opened in ContainerContents.
public static partial class TakeItems
{
    /// What taking an object does, from its id.
    internal abstract record Take
    {
        internal sealed record Gold : Take;

        /// The scrap of paper; the second message depends on being indoors.
        internal sealed record Paper(bool Indoors) : Take;

        /// Fruit: kept in the pack when not hungry, otherwise eaten.
        internal sealed record Fruit(bool Keep) : Take;

        /// Turtle eggs cannot be taken.
        internal sealed record Eggs : Take;

        internal sealed record Bones : Take;

        /// A chest, urn or sacks (with the name announced).
        internal sealed record Container(string Name) : Take;

        /// An empty chest or a footstool: nothing happens.
        internal sealed record Left : Take;

        /// Anything else: looked up in Itrans.
        internal sealed record Plain : Take;
    }

    /// What searching a body turns up.
    internal sealed record BodyLoot(uint Weapon, sbyte HeroWeapon, int? Arrows, uint Treasure);

    /// The random draws a search can ask for: the arrows only for an archer, the treasure only
    /// when there are no arrows.
    internal enum LootRoll
    {
        Arrows,
        Treasure,
    }

    // Object ids of things that are not plain pickups.
    private const uint URN = 0x0e;
    private const uint CHEST = 0x0f;
    private const uint SACKS = 0x10;
    private const uint GOLD_PILE = 0x0d;
    private const uint SCRAP_OF_PAPER = 0x14;
    private const uint EMPTY_CHEST = 0x1d;
    private const uint BROTHER_BONES = 28;
    private const uint FOOTSTOOL = 31;
    private const uint TURTLE_EGGS = 102;
    private const uint FRUIT = 148;

    /// Picking up a quiver gives ten arrows each; a dead brother's bones keep the owner in the low 7 bits.
    private const uint ARROWS_PER_QUIVER = 10;
    private const uint OWNER_MASK = 0x7f;

    // Object list slots (in the global list) of the two brothers' bones.
    private const int FIRST_BONES = 3;
    private const int SECOND_BONES = 4;

    /// Hunger below this and the fruit is kept for later instead of being eaten.
    private const short NOT_HUNGRY = 15;
    private const int FRUIT_HUNGER = 30;
    private const short GOLD_FOUND = 50;

    /// Search range for the nearest thing to take.
    public const short TAKE_RANGE = 30;
    private const int PRQ_NOTHING_NEAR = 10;
    private const int PRQ_STATS_LINE = 7;
    private const int EVENT_PAPER = 17;
    private const int EVENT_PAPER_OUTDOORS = 18;
    private const int EVENT_PAPER_INDOORS = 19;
    private const int EVENT_ALIVE = 35;
    private const int EVENT_FRUIT_KEPT = 36;
    private const int FRUIT_SLOT = 24;
    private const int TALISMAN = 22;

    /// Regions numbered above this are indoors.
    private const ushort FIRST_INDOOR_REGION = 7;

    // Searching a body.
    private const uint ARCHER_WEAPON = 4;
    private const int EXTRA_ARROWS_MIN = 2;
    private const sbyte MAX_BODY_WEAPON = 5;
    private const byte SETFIG_RACE_BIT = 0x80;
    private const int TREASURE_CLASS_SIZE = 8;

    /// Takes the nearest object, or searches the nearest dead body.
    public static void TakeNearest(Data.Game game, IOutput output)
    {
        TextRoutines.QueuePrint(game.Ui, PRQ_STATS_LINE);
        Blocking.FindNearestFigure(game, 0, TAKE_RANGE);
        game.Hero.Stuff[ItemsMenu.ARROWBASE] = 0;

        if (game.Actors.NearestObject == 0)
        {
            TextRoutines.QueuePrint(game.Ui, PRQ_NOTHING_NEAR);

            return;
        }

        TakeNearestActor(game, output);
    }

    /// Takes one chosen object instead of the nearest (Remastered: the one clicked on).
    public static void TakeFromActor(Data.Game game, IOutput output, int actorNumber)
    {
        TextRoutines.QueuePrint(game.Ui, PRQ_STATS_LINE);
        game.Hero.Stuff[ItemsMenu.ARROWBASE] = 0;
        game.Actors.NearestObject = (short)actorNumber;
        TakeNearestActor(game, output);
    }

    /// Whether taking an actor would do anything: it is an object that can be picked up or opened
    /// (a chest, a pile of gold, a weapon lying about), not scenery, or a body that can be searched.
    public static bool CanTake(Shape actor)
    {
        if (IsUnsearchedBody(actor))
        {
            return true;
        }

        if (actor.Kind != ActorKind.OBJECTS)
        {
            return false;
        }

        return ClassifyTake((byte)actor.Index, 0, 0) switch
        {
            Take.Eggs or Take.Left => false,
            Take.Plain => ItemSlot((byte)actor.Index, ObjectTables.Itrans) is not null,
            _ => true,
        };
    }

    // Takes the object or searches the body NearestObject.
    private static void TakeNearestActor(Data.Game game, IOutput output)
    {
        Shape actor = game.Actors.List[game.Actors.NearestObject];

        if (actor.Kind == ActorKind.OBJECTS)
        {
            if (TakeObject(game, output))
            {
                FinishPickup(game, output);
            }
        }
        else if (actor.Weapon < 0)
        {
            // Already searched.
        }
        else if (actor.Vitality == 0 || game.World.FreezeTimer != 0)
        {
            SearchBody(game, output);
        }
        else
        {
            TextRoutines.ShowEvent(game, output, EVENT_ALIVE);
        }
    }

    /// Sorts a nearby object by what taking it does. Fruit is kept when the hero is not hungry and
    /// eaten otherwise; the scrap of paper reads differently indoors.
    internal static Take ClassifyTake(uint identifier, short hunger, ushort region) => identifier switch
    {
        GOLD_PILE => new Take.Gold(),
        SCRAP_OF_PAPER => new Take.Paper(region > FIRST_INDOOR_REGION),
        FRUIT => new Take.Fruit(hunger < NOT_HUNGRY),
        TURTLE_EGGS => new Take.Eggs(),
        BROTHER_BONES => new Take.Bones(),
        CHEST => new Take.Container("a chest"),
        URN => new Take.Container("a brass urn"),
        SACKS => new Take.Container("some sacks"),
        EMPTY_CHEST or FOOTSTOOL => new Take.Left(),
        _ => new Take.Plain(),
    };
}
