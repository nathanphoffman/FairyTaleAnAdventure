using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Land;

/// Objects: adding objects, people and items as actors.
public static partial class Objects
{
    private static void ChooseSetfigFile(Data.Game game)
    {
        setfigFileId = -1;

        if (game.Experience == Experience.Original)
        {
            return;
        }

        Shape hero = game.Actors.List[0];
        int loaded = SpriteFiles.All[game.Actors.LoadedSetfigFile].FileId;
        int nearestDistance = int.MaxValue;

        foreach ((WorldObject[] list, int length) in new[] { (game.Objects.List(ObjectList.Global), (int)ObjectTables.Glbobs), (game.Objects.Region(game.World.Region), (int)game.Objects.Used[game.World.Region]) })
        {
            for (int index = 0; index < length; index++)
            {
                WorldObject item = list[index];

                if (item.ObStat is not (STAT_PERSON or STAT_DEAD_PERSON) || !InReach(game, item, WIDE_PERSON_LOAD))
                {
                    continue;
                }

                int fileId = SpriteFiles.All[Tables.Entry(Tables.SetfigTable, item.ObId).CfileEntry].FileId;
                int distance = Math.Max(Math.Abs(item.Xc - hero.AbsX), Math.Abs(item.Yc - hero.AbsY));

                if (fileId == loaded && InReach(game, item, ORIGINAL_PERSON_LOAD))
                {
                    setfigFileId = loaded;

                    return;
                }

                if (distance < nearestDistance)
                {
                    (nearestDistance, setfigFileId) = (distance, fileId);
                }
            }
        }
    }

    // Makes sure the sprite file for the kind of person id is loaded. Several kinds share one
    // file, so it is only reloaded when the needed file differs from the loaded one.
    private static void LoadSetFigure(Actors actors, IOutput output, sbyte id)
    {
        int entry = Tables.Entry(Tables.SetfigTable, id).CfileEntry;

        if (SpriteFiles.All[entry].FileId == SpriteFiles.All[actors.LoadedSetfigFile].FileId)
        {
            return;
        }

        actors.LoadedSetfigFile = (short)entry;
        output.RewindSpritesTo(SpriteFiles.SETFIG);
        output.LoadSpriteFile(entry);
        output.FinishSpriteLoad(SpriteFiles.SETFIG);
    }

    // Adds one object of a list as an actor, if it is near enough and not hidden or taken.
    private static void AddObject(Data.Game game, IOutput output, ref WorldObject item, short index, int f)
    {
        short xStart = (short)(item.Xc - game.World.MapX - SPRITE_CENTER_OFFSET);
        short yStart = (short)(item.Yc - game.World.MapY - SPRITE_CENTER_OFFSET);

        // States 3 and 4 are people; everything else is an item.
        bool isPerson = item.ObStat == STAT_PERSON || item.ObStat == STAT_DEAD_PERSON;

        if (isPerson && !LoadPersonIfNear(game, output, item, xStart, yStart))
        {
            return;
        }

        if (item.ObStat == STAT_TAKEN || item.ObStat == STAT_HIDDEN)
        {
            return;
        }

        Reach shown = game.Experience == Experience.Original ? ORIGINAL_SHOWN : WIDE_SHOWN;

        if (!(Inside(xStart, shown.XLow, shown.XHigh) && Inside(yStart, shown.YLow, shown.YHigh)))
        {
            return;
        }

        Shape actor;

        if (isPerson)
        {
            if (AddPersonActor(game.Actors, ref item, index, yStart) is not { } placed)
            {
                return;
            }

            (actor, yStart) = placed;
        }
        else
        {
            actor = AddItemActor(game.Actors, item, index, f);
        }

        actor.AbsX = item.Xc;
        actor.AbsY = item.Yc;
        actor.RelX = (ushort)xStart;
        actor.RelY = (ushort)yStart;
    }

    // A person close enough to the screen gets their pictures loaded (the witch also marks herself
    // as on screen). Returns false if the person is too far away.
    private static bool LoadPersonIfNear(Data.Game game, IOutput output, WorldObject item, short xStart, short yStart)
    {
        Reach reach = game.Experience == Experience.Original ? ORIGINAL_PERSON_LOAD : WIDE_PERSON_LOAD;

        if (!(Inside(xStart, reach.XLow, reach.XHigh) && Inside(yStart, reach.YLow, reach.YHigh)))
        {
            return false;
        }

        if (game.Experience != Experience.Original && SpriteFiles.All[Tables.Entry(Tables.SetfigTable, item.ObId).CfileEntry].FileId != setfigFileId)
        {
            return false;
        }

        LoadSetFigure(game.Actors, output, item.ObId);

        if (item.ObId == WITCH && item.ObStat == STAT_PERSON)
        {
            game.Witch.OnScreen = 1;
        }

        return true;
    }

    // Makes a person (a set figure) out of a world object. Returns the actor and the screen row to
    // use (people stand a little higher than the object's spot), or null if the person is already
    // there as an actor (the object is then marked dead if the actor died).
    private static (Shape Actor, short Row)? AddPersonActor(Actors actors, ref WorldObject item, short index, short yStart)
    {
        sbyte id = item.ObId;
        Shape actor = actors.List[actors.FreeSlot];

        actors.FreeSlot++;

        if (actor.AbsX == item.Xc && actor.AbsY == item.Yc)
        {
            if (actor.State == ActorState.DEAD)
            {
                item.ObStat = STAT_DEAD_PERSON;
            }

            return null;
        }

        actor.Kind = ActorKind.SETFIG;
        actor.Index = Tables.Entry(Tables.SetfigTable, id).ImageBase;
        actor.Weapon = 0;
        actor.Vitality = (short)(2 + id + id);
        actor.Goal = (sbyte)index;
        actor.Environ = 0;
        actor.State = item.ObStat == STAT_DEAD_PERSON ? ActorState.DEAD : ActorState.STILL;

        // A person's race is 0x80 plus their kind, so they can be told from monsters.
        actor.Race = (byte)(id + PERSON_RACE_BASE);

        if (actors.FreeSlot > actors.Drawn)
        {
            actors.Drawn = actors.FreeSlot;
        }

        return (actor, (short)(yStart - PERSON_RAISE));
    }

    // Makes an item lying about (or the turtle's eggs, a raft...) out of a world object.
    private static Shape AddItemActor(Actors actors, WorldObject item, short index, int f)
    {
        if (item.ObId == (sbyte)ObjectTables.TURTLE)
        {
            actors.TurtleEggs = (sbyte)actors.Drawn;
        }

        Shape actor = actors.List[actors.Drawn];

        actors.Drawn++;
        actor.Kind = ActorKind.OBJECTS;
        actor.Index = item.ObId;
        actor.Weapon = 0;
        actor.Vitality = (short)(index + f);
        actor.Race = item.ObStat switch
        {
            STAT_SECOND_ITEM => 2,
            STAT_RAFT_ITEM => 0,
            _ => 1,
        };

        return actor;
    }
}
