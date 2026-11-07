using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// Works out what the hero is looking at, as a key of look_text.json: the nearest creature, person
/// or object, else the scenery around him (trees, water, rock, wall). Bare ground gives nothing.
public static class LookSubject
{
    private const int SCENERY_PROBE_DISTANCE = ItemsMenu.LOOK_RANGE / 2;
    private const int TURTLE_CARRIER = 5;
    private const int SETFIG_FIRST_RACE = 0x80;
    private const int TREE_FIRST_COLOR = 16;
    private const int TREE_LAST_COLOR = 19;
    private const int FIRST_INDOOR_REGION = 8;

    // The overview colour byte of a tile table entry, as in the world map.
    private const int TERRAIN_ENTRY_BYTES = 4;
    private const int TERRAIN_COLOR_BYTE = 3;
    private const int COLOR_MASK = 31;

    // Terrain types: 1 blocks the way, 2 to 5 are water of growing depth.
    private const int BLOCKED_TERRAIN = 1;
    private const int FIRST_WATER_TERRAIN = 2;
    private const int LAST_WATER_TERRAIN = 5;

    private static readonly string[] ENEMY_KEYS =
        ["ogre", "orc", "wraith", "skeleton", "snake", "salamander", "spider", "dark_knight", "loraii", "necromancer", "woodcutter"];

    private static readonly string[] PERSON_KEYS =
        ["wizard", "priest", "guard", "guard", "princess", "king", "noble", "sorceress", "bartender", "witch", "spectre", "ghost", "ranger", "beggar"];

    // The probes: the hero's own spot and a ring of spots all round him.
    private static readonly (int X, int Y)[] PROBES =
    [
        (0, 0), (SCENERY_PROBE_DISTANCE, 0), (-SCENERY_PROBE_DISTANCE, 0), (0, SCENERY_PROBE_DISTANCE), (0, -SCENERY_PROBE_DISTANCE),
        (SCENERY_PROBE_DISTANCE, -SCENERY_PROBE_DISTANCE), (-SCENERY_PROBE_DISTANCE, -SCENERY_PROBE_DISTANCE),
        (SCENERY_PROBE_DISTANCE, SCENERY_PROBE_DISTANCE), (-SCENERY_PROBE_DISTANCE, SCENERY_PROBE_DISTANCE),
    ];

    /// The key of the thing the hero is looking at, or null when there is nothing worth a line.
    public static string? Find(Data.Game game) => NearbyActorKey(game) ?? SceneryKey(game);

    // The nearest actor within the look range, named by its kind and race.
    private static string? NearbyActorKey(Data.Game game)
    {
        int nearest = 0;
        int nearestDistance = ItemsMenu.LOOK_RANGE;

        for (int index = 1; index < game.Actors.Drawn; index++)
        {
            int distance = Blocking.ActorDistance(game, index, 0);

            if (distance < nearestDistance && ActorKey(game, game.Actors.List[index]) != null)
            {
                nearest = index;
                nearestDistance = distance;
            }
        }

        return nearest == 0 ? null : ActorKey(game, game.Actors.List[nearest]);
    }

    // What an actor is called in look_text.json, or null for the ones that get no line.
    private static string? ActorKey(Data.Game game, Shape actor) => actor.Kind switch
    {
        ActorKind.ENEMY => actor.State == ActorState.DEAD ? "body" : ENEMY_KEYS.ElementAtOrDefault(actor.Race),
        ActorKind.SETFIG => PERSON_KEYS.ElementAtOrDefault(actor.Race - SETFIG_FIRST_RACE),
        ActorKind.CARRIER => game.Actors.ActiveCarrier == TURTLE_CARRIER ? "turtle" : "swan",
        ActorKind.DRAGON => "dragon",
        ActorKind.RAFT => "raft",
        ActorKind.OBJECTS when actor.Race != 0 => ObjectKey((byte)actor.Index),
        _ => null,
    };

    // Chests, urns and the like get their own lines; keys and magic things share two, the rest one.
    private static string ObjectKey(byte id) => id switch
    {
        ObjectTables.CHEST => "chest",
        ObjectTables.URN => "urn",
        ObjectTables.SACKS => "sacks",
        ObjectTables.MONEY => "gold",
        ObjectTables.QUIVER => "arrows",
        ObjectTables.GOLD_KEY or ObjectTables.GREY_KEY or ObjectTables.BLUE_KEY or ObjectTables.GREEN_KEY or ObjectTables.WHITE_KEY or ObjectTables.RED_KEY => "key",
        >= ObjectTables.G_RING and <= ObjectTables.J_SKULL => "treasure",
        _ => "item",
    };

    // The scenery that fills most of the probed spots round the hero; null when the ground is bare.
    private static string? SceneryKey(Data.Game game)
    {
        Shape hero = game.Actors.List[0];
        int water = 0;
        int blocked = 0;
        (int X, int Y)? firstBlocked = null;

        foreach ((int dx, int dy) in PROBES)
        {
            int terrain = MapLookup.TerrainAt(game, hero.AbsX + dx, hero.AbsY + dy);

            if (terrain >= FIRST_WATER_TERRAIN && terrain <= LAST_WATER_TERRAIN)
            {
                water++;
            }
            else if (terrain == BLOCKED_TERRAIN)
            {
                blocked++;
                firstBlocked ??= (hero.AbsX + dx, hero.AbsY + dy);
            }
        }

        if (water == 0 && blocked == 0)
        {
            return null;
        }

        return water > blocked ? "water" : BlockedKey(game, firstBlocked!.Value);
    }

    // Trees are blocked ground drawn in greens; indoors it is a wall, else rock.
    private static string BlockedKey(Data.Game game, (int X, int Y) spot)
    {
        if (game.World.Region >= FIRST_INDOOR_REGION)
        {
            return "wall";
        }

        int tile = MapLookup.TileNumber(game, (ushort)(spot.X >> 4), (ushort)(spot.Y >> 5));
        int color = game.Map.TerrainByte(tile * TERRAIN_ENTRY_BYTES + TERRAIN_COLOR_BYTE) & COLOR_MASK;

        return color >= TREE_FIRST_COLOR && color <= TREE_LAST_COLOR ? "tree" : "rock";
    }
}
