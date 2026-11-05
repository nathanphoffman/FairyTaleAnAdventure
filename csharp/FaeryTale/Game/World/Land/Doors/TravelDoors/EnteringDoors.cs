using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Land.TravelDoors;

/// Going in through a door from outdoors: finding the door and where to arrive.
public static class EnteringDoors
{
    internal const sbyte DESERT = 17;

    // Inventory slot of the gold statues, five are needed to enter the desert.
    private const int STATBASE = 25;
    internal const byte STATUES_NEEDED = 5;
    internal const ushort REGION_INSIDE = 8;
    internal const ushort REGION_INSIDE_OTHER = 9;
    private const short FADE_WHITE = 100;

    // Arrival offsets from a door's corner (pixels): in through a cave door, a horizontal door and
    // a vertical door.
    private const ushort CAVE_IN_X = 24;
    private const ushort CAVE_IN_Y = 16;
    private const ushort HORIZONTAL_IN_X = 16;
    private const ushort HORIZONTAL_IN_Y = 0;
    private const ushort VERTICAL_IN_X = 1;
    private const ushort VERTICAL_IN_Y = 16;
    private const ushort IN_COLUMN_LIMIT = 6;

    /// The door with this number.
    internal static Door DoorAt(int index) => Tables.DoorList[index];

    /// Goes in through the door at the tile (x, y), if there is one and the hero may enter it.
    public static void EnterDoor(Data.Game game, IOutput output, ushort x, ushort y)
    {
        if (FindOutdoorDoor(x, y) is not { } index)
        {
            return;
        }

        Door door = DoorAt(index);

        if (!MayEnter(door, game.Hero.X, game.Hero.Y, game.Hero.Stuff[STATBASE]))
        {
            return;
        }

        (ushort arriveX, ushort arriveY) = InsideSpot(door);

        game.World.NewRegion = RegionInside(door);
        Places.TransferHero(game, output, arriveX, arriveY, 0);
        Places.FindPlace(game, output, 2);
        Fading.FadePage(game, output, (FADE_WHITE, FADE_WHITE, FADE_WHITE), 1, (ushort[])game.PageColors.Clone());
    }

    /// Where to arrive on the other side of a door going in.
    internal static (ushort X, ushort Y) InsideSpot(Door d)
    {
        if (d.Kind == TravelDoors.CAVE)
        {
            return ((ushort)(d.Xc2 + CAVE_IN_X), (ushort)(d.Yc2 + CAVE_IN_Y));
        }

        if ((d.Kind & 1) != 0)
        {
            return ((ushort)(d.Xc2 + HORIZONTAL_IN_X), (ushort)(d.Yc2 + HORIZONTAL_IN_Y));
        }

        return ((ushort)(d.Xc2 - VERTICAL_IN_X), (ushort)(d.Yc2 + VERTICAL_IN_Y));
    }

    /// The hero can go in when lined up with the door (horizontal doors by y, vertical ones by x),
    /// and the desert door needs the gold statues.
    internal static bool MayEnter(Door d, ushort heroX, ushort heroY, byte statues)
    {
        bool linedUp = (d.Kind & 1) != 0
            ? (heroY & TravelDoors.LOWER_HALF_BIT) == 0
            : (heroX & TravelDoors.TILE_COLUMN_MASK) <= IN_COLUMN_LIMIT;

        return linedUp && (d.Kind != DESERT || statues >= STATUES_NEEDED);
    }

    /// Which indoor region a door leads to.
    internal static ushort RegionInside(Door d) => d.Secs == 1 ? REGION_INSIDE : REGION_INSIDE_OTHER;

    /// Binary search of the (sorted) outdoor doors for the one whose entrance covers the point. A
    /// door matches within 16 pixels to the right of its x; vertical doors need the exact x.
    internal static int? FindOutdoorDoor(ushort x, ushort y)
    {
        int low = 0;
        int high = TravelDoors.DOOR_COUNT - 1;

        while (high >= low)
        {
            int middle = (high + low) / 2;
            Door d = DoorAt(middle);

            if (d.Xc1 > x)
            {
                high = middle - 1;
            }
            else if (d.Xc1 + TravelDoors.TILE_WIDTH < x)
            {
                low = middle + 1;
            }
            else if (d.Xc1 < x && (d.Kind & 1) == 0)
            {
                low = middle + 1; // vertical door: its x must match exactly
            }
            else if (d.Yc1 > y)
            {
                high = middle - 1;
            }
            else if (d.Yc1 < y)
            {
                low = middle + 1;
            }
            else
            {
                return middle;
            }

            if (low >= TravelDoors.DOOR_COUNT || high < 0)
            {
                break;
            }
        }

        return null;
    }
}
