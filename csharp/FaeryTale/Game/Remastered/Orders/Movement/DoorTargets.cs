using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Land.TravelDoors;

namespace FaeryTale.Game.Remastered;

/// Finding the doors between the outdoors and the buildings under the mouse, and where the hero has
/// to stand to go through one. The tests for "is this the door" are the game's own (the same ones
/// the hero triggers by walking into a door).
public static class DoorTargets
{
    // Tiles are 16 pixels wide and 32 high; doors are found by the tile a position is in.
    private const int TILE_X_MASK = ~15;
    private const int TILE_Y_MASK = ~31;

    // Where to stand, from a door's corner, to go in or out: the middle of a horizontal door, a
    // little in from the side of a vertical one, and the half of the tile the game asks for.
    // (a horizontal door is two tiles wide, so its middle is a whole tile in from its corner, where
    // both of the hero's ground probes, 4 pixels either side, are inside the doorway)
    private const int HORIZONTAL_MIDDLE_X = 16;
    private const int UPPER_HALF_Y = 8;
    private const int LOWER_HALF_Y = 24;
    private const int VERTICAL_SIDE_X_OUT = 3;
    private const int VERTICAL_SIDE_X_IN = 8;
    private const int VERTICAL_MIDDLE_Y = 16;

    /// How far in front of a closed door to stop before pushing in.
    private const int FRONT_GAP = 6;
    private const int TILE_HEIGHT = 32;

    // The black outside of the rooms is tile 0.
    private const int TILE_X_SHIFT = 4;
    private const int TILE_Y_SHIFT = 5;
    private const int BLACK_TILE = 0;

    /// An exit further than this from the hero is not the exit of this room.
    private const int MAX_EXIT_DISTANCE = 512;

    /// The number of the door at a world position, if there is one: outdoors an entrance, indoors
    /// the way out.
    public static int? DoorAt(Data.Game game, (int X, int Y) world)
    {
        ushort x = (ushort)(world.X & TILE_X_MASK);
        ushort y = (ushort)(world.Y & TILE_Y_MASK);

        if (game.World.Region < EnteringDoors.REGION_INSIDE)
        {
            return EnteringDoors.FindOutdoorDoor(x, y);
        }

        for (int index = 0; index < TravelDoors.DOOR_COUNT; index++)
        {
            if (LeavingDoors.IsExitHere(EnteringDoors.DoorAt(index), x, y))
            {
                return index;
            }
        }

        return null;
    }

    /// When indoors and the world position is the black outside of the rooms (tile 0), the door that
    /// leads out of the hero's room, so a click there means "leave".
    public static int? ExitForBlackClick(Data.Game game, (int X, int Y) world)
    {
        bool indoors = game.World.Region >= EnteringDoors.REGION_INSIDE;

        if (!indoors || MapLookup.TileNumber(game, (ushort)(world.X >> TILE_X_SHIFT), (ushort)(world.Y >> TILE_Y_SHIFT)) != BLACK_TILE)
        {
            return null;
        }

        return NearestExit(game);
    }

    /// Where to walk to before pushing into the door at an entry point. A closed outdoor door only
    /// opens when bumped from the front: from below for a horizontal door and from the left for a
    /// vertical one. Indoors the way out is open, so the hero walks straight to it.
    public static (int X, int Y) ApproachPoint(Data.Game game, int doorIndex)
    {
        Door door = EnteringDoors.DoorAt(doorIndex);

        if (game.World.Region >= EnteringDoors.REGION_INSIDE)
        {
            return EntryPoint(game, doorIndex);
        }

        return (door.Kind & 1) != 0
            ? (door.Xc1 + HORIZONTAL_MIDDLE_X, door.Yc1 + TILE_HEIGHT + FRONT_GAP)
            : (door.Xc1 - FRONT_GAP, door.Yc1 + VERTICAL_MIDDLE_Y);
    }

    /// Where the hero has to stand to go through a door: a spot the game's own "lined up with the
    /// door" test accepts.
    public static (int X, int Y) EntryPoint(Data.Game game, int doorIndex)
    {
        Door door = EnteringDoors.DoorAt(doorIndex);
        bool horizontal = (door.Kind & 1) != 0;

        if (game.World.Region < EnteringDoors.REGION_INSIDE)
        {
            return horizontal
                ? (door.Xc1 + HORIZONTAL_MIDDLE_X, door.Yc1 + UPPER_HALF_Y)
                : (door.Xc1 + VERTICAL_SIDE_X_OUT, door.Yc1 + VERTICAL_MIDDLE_Y);
        }

        return horizontal
            ? (door.Xc2 + HORIZONTAL_MIDDLE_X, door.Yc2 + LOWER_HALF_Y)
            : (door.Xc2 + VERTICAL_SIDE_X_IN, door.Yc2 + VERTICAL_MIDDLE_Y);
    }

    // The door out of the building that is nearest to the hero, if one is close enough.
    private static int? NearestExit(Data.Game game)
    {
        Shape hero = game.Actors.List[0];
        int? nearest = null;
        int nearestDistance = MAX_EXIT_DISTANCE;

        for (int index = 0; index < TravelDoors.DOOR_COUNT; index++)
        {
            (int x, int y) = EntryPoint(game, index);
            int distance = Math.Max(Math.Abs(x - hero.AbsX), Math.Abs(y - hero.AbsY));

            if (distance < nearestDistance)
            {
                (nearest, nearestDistance) = (index, distance);
            }
        }

        return nearest;
    }
}
