using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Land.TravelDoors;

/// Going out through a door from inside: finding the door the hero stands in and where to arrive.
public static class LeavingDoors
{
    private const ushort CAVE_OUT_X = 4;
    private const ushort CAVE_OUT_Y = 16;
    private const ushort HORIZONTAL_OUT_X = 16;
    private const ushort HORIZONTAL_OUT_Y = 34;
    private const ushort VERTICAL_OUT_X = 20;
    private const ushort VERTICAL_OUT_Y = 16;
    private const ushort OUT_COLUMN_LIMIT = 2;

    /// Indoors: if the hero stands at the inside entrance of a door and is lined up with it, he is
    /// moved to the outside.
    public static void LeaveDoor(Data.Game game, IOutput output, ushort x, ushort y)
    {
        for (int index = 0; index < TravelDoors.DOOR_COUNT; index++)
        {
            Door door = EnteringDoors.DoorAt(index);

            if (!IsExitHere(door, x, y))
            {
                continue;
            }

            if (MayLeave(door, game.Hero.X, game.Hero.Y))
            {
                (ushort arriveX, ushort arriveY) = OutsideSpot(door);

                Places.TransferHero(game, output, arriveX, arriveY, 1);
                Places.FindPlace(game, output, 0);
            }

            break;
        }
    }

    /// Where to arrive going out.
    internal static (ushort X, ushort Y) OutsideSpot(Door d)
    {
        if (d.Kind == TravelDoors.CAVE)
        {
            return ((ushort)(d.Xc1 - CAVE_OUT_X), (ushort)(d.Yc1 + CAVE_OUT_Y));
        }

        if ((d.Kind & 1) != 0)
        {
            return ((ushort)(d.Xc1 + HORIZONTAL_OUT_X), (ushort)(d.Yc1 + HORIZONTAL_OUT_Y));
        }

        return ((ushort)(d.Xc1 + VERTICAL_OUT_X), (ushort)(d.Yc1 + VERTICAL_OUT_Y));
    }

    /// Whether a door's inside entrance is where the hero stands (x, y: the hero's position
    /// rounded to a tile). Horizontal doors are also found one tile to the right.
    internal static bool IsExitHere(Door d, ushort x, ushort y)
    {
        bool horizontal = (d.Kind & 1) != 0;

        return d.Yc2 == y && (d.Xc2 == x || (d.Xc2 == x - TravelDoors.TILE_WIDTH && horizontal));
    }

    /// The hero can go out when lined up with the door the other way round to MayEnter.
    internal static bool MayLeave(Door d, ushort heroX, ushort heroY) =>
        (d.Kind & 1) != 0
            ? (heroY & TravelDoors.LOWER_HALF_BIT) != 0
            : (heroX & TravelDoors.TILE_COLUMN_MASK) >= OUT_COLUMN_LIMIT;
}
