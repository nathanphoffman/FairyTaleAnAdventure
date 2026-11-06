using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Ui;

/// Taking what lies nearby: gold, fruit, paper, a brother's bones and plain items, and searching
/// dead bodies. Chests, urns and sacks are opened in ContainerContents.
public static class TakeItems
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

    /// Whether an actor is a body that has not been searched yet (a dead enemy or person; a searched
    /// body has its weapon set to -1).
    public static bool IsUnsearchedBody(Shape actor) =>
        (actor.Kind == ActorKind.ENEMY || actor.Kind == ActorKind.SETFIG)
        && actor.State == ActorState.DEAD
        && actor.Weapon >= 0
        && actor.Vitality == 0;

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
    internal static Take ClassifyTake(uint id, short hunger, ushort region) => id switch
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

    /// The inventory slot for object id in the (id, slot) pair table Itrans, which ends with a 0 id.
    internal static int? ItemSlot(uint id, byte[] table)
    {
        for (int at = 0; table[at] != 0; at += 2)
        {
            if (id == table[at])
            {
                return table[at + 1];
            }
        }

        return null;
    }

    /// The weapon a body yields (0 for none): the unarmed and odd values count as nothing.
    internal static uint BodyWeapon(sbyte raw) => (byte)raw > MAX_BODY_WEAPON ? 0u : (uint)(byte)raw;

    /// The hero keeps a better weapon in hand and only takes up a found one that is better.
    internal static sbyte BetterWeapon(sbyte current, uint found) => (sbyte)found > current ? (sbyte)found : current;

    /// What searching a body turns up: roll draws the arrows or the treasure when asked.
    internal static BodyLoot BodyLootFor(sbyte bodyWeaponRaw, sbyte heroWeapon, Func<LootRoll, int> roll)
    {
        uint weapon = BodyWeapon(bodyWeaponRaw);
        sbyte newHeroWeapon = weapon != 0 ? BetterWeapon(heroWeapon, weapon) : heroWeapon;

        if (weapon == ARCHER_WEAPON)
        {
            return new BodyLoot(weapon, newHeroWeapon, roll(LootRoll.Arrows), 0);
        }

        return new BodyLoot(weapon, newHeroWeapon, null, (uint)roll(LootRoll.Treasure));
    }

    // Removes the picked-up object from the world, turns any arrows picked up into the arrow
    // count, and ends the game if the talisman was found.
    private static void FinishPickup(Data.Game game, IOutput output)
    {
        Objects.ChangeObject(game.Objects, game.Actors, game.World, game.Actors.NearestObject, 2);
        ItemsMenu.AddStuff(game, ItemsMenu.ARROWS, (byte)(game.Hero.Stuff[ItemsMenu.ARROWBASE] * ARROWS_PER_QUIVER));

        if (game.Hero.Stuff[TALISMAN] != 0)
        {
            game.Ui.QuitRequested = 1;
            game.Ui.ViewStatus = ItemsMenu.VIEW_GAME_OVER;
            MessagePage.MapMessage(game, output);
            output.UseStoryFont();
            ScreenEffects.WinColors(game, output);
        }
    }

    // Handles taking the object NearestObject. Returns true if it was taken (so it must be removed).
    private static bool TakeObject(Data.Game game, IOutput output)
    {
        Shape actor = game.Actors.List[game.Actors.NearestObject];
        uint id = (byte)actor.Index;
        uint owner = (uint)actor.Vitality & OWNER_MASK;

        switch (ClassifyTake(id, game.Hero.Hunger, game.World.Region))
        {
            case Take.Gold:
                Messages.AnnounceTreasure(game, output, "50 gold pieces");
                game.Hero.Wealth += GOLD_FOUND;

                return true;

            case Take.Paper paper:
                TextRoutines.ShowEvent(game, output, EVENT_PAPER);
                TextRoutines.ShowEvent(game, output, paper.Indoors ? EVENT_PAPER_INDOORS : EVENT_PAPER_OUTDOORS);

                return true;

            case Take.Fruit fruit:
                return TakeFruit(game, output, fruit.Keep);

            case Take.Bones:
                return TakeBrothersGoods(game, output, owner);

            case Take.Container container:
                Messages.AnnounceContainer(game, output, container.Name);

                return ContainerContents.Open(game, output);

            case Take.Plain:
                return TakePlainItem(game, output, id);

            default:
                return false; // eggs and left things are not taken
        }
    }

    // Fruit is kept in the pack when the hero is not hungry, and eaten otherwise.
    private static bool TakeFruit(Data.Game game, IOutput output, bool keep)
    {
        if (keep)
        {
            ItemsMenu.AddStuff(game, FRUIT_SLOT, 1);
            TextRoutines.ShowEvent(game, output, EVENT_FRUIT_KEPT);
        }
        else
        {
            Food.EatFood(game, output, FRUIT_HUNGER);
        }

        return true;
    }

    // Picks up a fallen brother's bones and everything he carried (owner 1 is Julian, otherwise Phillip).
    private static bool TakeBrothersGoods(Data.Game game, IOutput output, uint owner)
    {
        Messages.AnnounceTreasure(game, output, "his brother's bones.");
        game.Objects.Global[FIRST_BONES].ObStat = 0;
        game.Objects.Global[SECOND_BONES].ObStat = 0;

        for (int slot = 0; slot < ItemsMenu.GOLDBASE; slot++)
        {
            ItemsMenu.AddStuff(game, slot, owner == 1 ? game.Hero.Julstuff[slot] : game.Hero.Philstuff[slot]);
        }

        return true;
    }

    // A single item lying about: the slot is found in the Itrans table. Unknown objects are left.
    private static bool TakePlainItem(Data.Game game, IOutput output, uint id)
    {
        if (ItemSlot(id, ObjectTables.Itrans) is not { } slot)
        {
            return false;
        }

        ItemsMenu.AddStuff(game, slot, 1);
        Messages.AnnounceTreasure(game, output, "a ");
        output.PrintCont(ItemsMenu.ItemName(slot));
        output.PrintCont(".");

        return true;
    }

    // Searches the dead body NearestObject: takes its weapon and any treasure.
    private static void SearchBody(Data.Game game, IOutput output)
    {
        Shape actor = game.Actors.List[game.Actors.NearestObject];

        Messages.PrintWrapped(game, output, "% searched the body and found"u8);
        output.Print("");

        byte race = actor.Race;
        BodyLoot loot = BodyLootFor(actor.Weapon, game.Actors.List[0].Weapon, roll =>
            roll == LootRoll.Arrows ? game.Rng.Rand8() + EXTRA_ARROWS_MIN : (int)RollBodyTreasure(race, game.Rng));

        actor.Weapon = -1;
        TakeFoundWeapon(game, output, loot);

        if (loot.Arrows is { } arrows)
        {
            output.PrintCont(" and ");
            output.PrintNumber(arrows, 1);
            output.PrintCont(" Arrows.");
            ItemsMenu.AddStuff(game, ItemsMenu.ARROWS, (byte)arrows);

            return;
        }

        TakeTreasure(game, output, loot);
        output.PrintCont(".");
    }

    // Says what weapon was found, adds it to the pack, and arms the hero with it if it is better.
    private static void TakeFoundWeapon(Data.Game game, IOutput output, BodyLoot loot)
    {
        if (loot.Weapon == 0)
        {
            return;
        }

        output.PrintCont("a ");
        output.PrintCont(ItemsMenu.ItemName((int)loot.Weapon - 1));
        ItemsMenu.AddStuff(game, (int)loot.Weapon - 1, 1);
        game.Actors.List[0].Weapon = loot.HeroWeapon;
    }

    // Says what other treasure was found and takes it: gold into the purse, anything else into the
    // pack. Says "nothing" if there was neither weapon nor treasure.
    private static void TakeTreasure(Data.Game game, IOutput output, BodyLoot loot)
    {
        if (loot.Treasure == 0)
        {
            if (loot.Weapon == 0)
            {
                output.PrintCont(game.Experience == Experience.Original ? "nothing" : "nothing in their pockets");
            }

            return;
        }

        if (loot.Weapon != 0)
        {
            output.PrintCont(" and ");
        }

        if (loot.Treasure < ItemsMenu.GOLDBASE)
        {
            output.PrintCont("a ");
        }

        output.PrintCont(ItemsMenu.ItemName((int)loot.Treasure));

        if (loot.Treasure >= ItemsMenu.GOLDBASE)
        {
            game.Hero.Wealth += Tables.ItemList[loot.Treasure].Maxshown;
        }
        else
        {
            ItemsMenu.AddStuff(game, (int)loot.Treasure, 1);
        }
    }

    // Picks what a dead enemy carries besides its weapon, from the treasure table for its race.
    // Returns an item number (0 for nothing); characters like shopkeepers (a set figure) carry none.
    private static uint RollBodyTreasure(byte race, Rng rng)
    {
        if ((race & SETFIG_RACE_BIT) != 0)
        {
            return 0;
        }

        sbyte treasureClass = Tables.Entry(Tables.EncounterChart, race).Treasure;

        return (uint)(int)ObjectTables.TreasureProbs[treasureClass * TREASURE_CLASS_SIZE + rng.Rand8()];
    }
}
