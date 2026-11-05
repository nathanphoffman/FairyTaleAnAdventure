using FaeryTale.Game.Creatures;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Land;

/// Moving the hero somewhere new, and working out where the hero is.
///
/// FindPlace looks up the sector the hero stands on in a table to get the place name message
/// ("You are in a village" ...), and looks the position up in the extent list to learn what kind
/// of area it is (ExtentType): safe, a monster zone, a boss, a bird or turtle area, and so on.
/// Stepping into some of them starts something: monsters appear, or a carrier is loaded.
public static class Places
{
    // Moving the hero picks the region from the centre sector: the row there is 3 bits (8 rows).
    private const uint TRANSFER_ROW_MASK = 7;

    // The place tables have a row of three bytes (first sector, last sector, message) per entry.
    private const int PLACE_TABLE_ROWS = 256;
    private const int PLACE_ROW_BYTES = 3;
    private const short SECTOR_BYTE_MASK = 255;

    /// Regions up to this number are in the north half of the map.
    private const ushort LAST_NORTHERN_REGION = 3;

    // A sector is 256 pixels (a shift of 8); a sector column with bit 6 set, or a row with bit 5
    // set, is outside the loaded region.
    private const int SECTOR_SHIFT = 8;
    private const int OUTSIDE_COLUMN_BIT = 64;
    private const int OUTSIDE_ROW_BIT = 32;

    /// The necromancer and the turtle eggs appear again when fewer than this many actors are active.
    private const short MIN_BOSS_ACTORS = 4;

    // The place message that stands for "the mountains", which depends on the region.
    private const byte MOUNTAINS_MESSAGE = 4;
    private const byte NORTHERN_MOUNTAINS_MESSAGE = 0;
    private const byte SOUTHERN_MOUNTAINS_MESSAGE = 5;

    /// How many extents there are (the last one covers the whole world).
    private const int EXTENT_COUNT = 22;

    /// Sector numbers inside buildings are offset by this in the place table.
    private const short INSIDE_SECTOR_OFFSET = 256;

    // Extent types.
    private const ushort XT_TURTLE_EGGS = 61;
    private const ushort XT_NECROMANCER = 60;
    private const ushort XT_SPIDER_PIT = 52;
    private const ushort XT_MONSTER_ZONE_MIN = 50;
    private const ushort XT_PRINCESS = 83;
    private const ushort XT_SPECIAL_MIN = 60;
    private const ushort XT_CARRIER_MIN = 70;
    private const ushort XT_CARRIER = 70;

    /// Slot in the interior object list of the princess object.
    private const int PRINCESS_OBJECT = 9;
    private const int INSIDE_OBJECTS = 8;

    // Slot of the first monster in the actor list, and the first slot after the monsters.
    private const short FIRST_MONSTER = 3;
    private const short MONSTER_SLOTS_END = 7;

    /// How far from the encounter spot monsters are placed.
    private const ushort ENCOUNTER_SPREAD = 63;
    private const short ENEMY_SLOT = 2;

    /// Encounter type used in the spider pit.
    private const ushort SPIDER_ENCOUNTER = 8;

    /// Moves the hero to the world position (x, y) and reloads what is needed there. With flag
    /// set the region is worked out from the position (like walking through a door); otherwise
    /// NewRegion must already be set. If the spot is blocked the hero is moved down until it is free.
    public static int TransferHero(Data.Game game, IOutput output, ushort x, ushort y, ushort flag)
    {
        game.World.MapX = (ushort)(game.World.MapX + (ushort)(x - game.Hero.X));
        game.World.MapY = (ushort)(game.World.MapY + (ushort)(y - game.Hero.Y));
        game.Hero.X = x;
        game.Actors.List[0].AbsX = x;
        game.Hero.Y = y;
        game.Actors.List[0].AbsY = y;

        game.Encounter.Number = 0;

        if (flag != 0)
        {
            (uint sectorColumn, uint sectorRow) = Regions.CenterSector(game.World);

            // Only the lowest bit of the sector column matters: which half of the region it is in;
            // the sector row's low bits pick the region row.
            uint column = (sectorColumn >> Regions.REGION_WIDTH_SHIFT) & 1;
            uint row = (sectorRow >> Regions.REGION_HEIGHT_SHIFT) & TRANSFER_ROW_MASK;

            game.World.NewRegion = (ushort)(column + row + row);
        }

        game.Ui.HeldDirection = 0;
        Regions.LoadRegionNow(game, output);
        Regions.RefreshMinimap(game, output); // to set the region origin
        game.Ui.ViewStatus = ViewStatus.REBUILD_ALL;
        Music.SetMood(game, 1);

        while (Blocking.BlockedAt(game, (short)game.Hero.X, (short)game.Hero.Y, 0) != 0)
        {
            game.Hero.Y++;
        }

        return 0;
    }

    /// Works out where the hero is: the place name (said when it changes and flag is set), then
    /// the extent the hero stands in, and whatever that extent starts.
    ///
    /// flag 0 is silent, 1 also starts monster zones, 2 only says the place name.
    public static int FindPlace(Data.Game game, IOutput output, short flag)
    {
        while (true)
        {
            AnnouncePlace(game, output, flag);
            LocateExtent(game);

            if (!EnterExtent(game, output, flag))
            {
                // The princess was rescued: look again from the top, without the messages.
                flag = 0;

                continue;
            }

            UpdateCarrier(game, output);

            return 0;
        }
    }

    // The place-name message number for the hero's sector, or 0 for none. Mountains get the name
    // of the part of the map they are in, and nothing is said while the next region is still
    // loading or when the hero is on the edge of the loaded one.
    private static byte PlaceMessageNumber(Data.Game game)
    {
        game.Hero.Sector &= SECTOR_BYTE_MASK;
        byte sector = (byte)game.Hero.Sector;
        byte[] table = StoryText.PLACE_TBL;

        if (game.World.Region >= Regions.FIRST_INDOOR_REGION)
        {
            game.Hero.Sector += INSIDE_SECTOR_OFFSET;
            table = StoryText.INSIDE_TBL;
        }

        byte message = MountainName(game.World, FindTableMessage(table, sector));

        return HeroOnLoadedEdge(game) ? (byte)0 : message;
    }

    // The message of the table row (three bytes: first sector, last sector, message) that holds sector.
    internal static byte FindTableMessage(byte[] table, byte sector)
    {
        int rows = Math.Min(table.Length / PLACE_ROW_BYTES, PLACE_TABLE_ROWS);

        for (int row = 0; row < rows; row++)
        {
            int at = row * PLACE_ROW_BYTES;

            if (sector >= table[at] && sector <= table[at + 1])
            {
                return table[at + 2];
            }
        }

        return 0;
    }

    // Mountains are named after the part of the map they are in (outdoors only).
    private static byte MountainName(World world, byte message)
    {
        if (message != MOUNTAINS_MESSAGE || world.Region >= Regions.FIRST_INDOOR_REGION)
        {
            return message;
        }

        if ((world.Region & 1) != 0)
        {
            return NORTHERN_MOUNTAINS_MESSAGE;
        }

        return world.Region > LAST_NORTHERN_REGION ? SOUTHERN_MOUNTAINS_MESSAGE : message;
    }

    // True while the next region is still loading, or when the hero is outside the loaded one.
    private static bool HeroOnLoadedEdge(Data.Game game)
    {
        // The hero's sector column and row relative to the loaded region: 256 pixels to a sector.
        ushort column = (ushort)((game.Hero.X >> SECTOR_SHIFT) - game.World.OriginX);
        ushort row = (ushort)((game.Hero.Y >> SECTOR_SHIFT) - game.World.OriginY);

        return game.World.NewRegion < Regions.NO_REGION || (column & OUTSIDE_COLUMN_BIT) != 0 || (row & OUTSIDE_ROW_BIT) != 0;
    }

    // Remembers the place the hero is in, and says its name when it changes (if flag is set).
    private static void AnnouncePlace(Data.Game game, IOutput output, short flag)
    {
        byte message = PlaceMessageNumber(game);

        if (message == 0 || message == game.Hero.Place)
        {
            return;
        }

        game.Hero.Place = message;

        if (flag != 0)
        {
            byte[] messages = game.World.Region >= Regions.FIRST_INDOOR_REGION ? StoryText.INSIDE_MSG : StoryText.PLACE_MSG;

            TextRoutines.ShowMessage(game, output, messages, message);
        }
    }

    // Finds the first extent (special rectangle of the world) the hero is in.
    private static void LocateExtent(Data.Game game)
    {
        game.World.Extent = 0;

        for (int count = 0; count < EXTENT_COUNT; count++)
        {
            Extent e = game.Extents[game.World.Extent];

            if (game.Hero.X > e.X1 && game.Hero.X < e.X2 && game.Hero.Y > e.Y1 && game.Hero.Y < e.Y2)
            {
                break;
            }

            game.World.Extent++;
        }
    }

    // What happens when the hero walks into a different kind of extent: the princess is rescued,
    // a boss or the turtle eggs call up their monsters, the spider pit loads the spiders, and
    // monster zones may send a fight. Returns false if the princess was rescued (the caller starts over).
    private static bool EnterExtent(Data.Game game, IOutput output, short flag)
    {
        Extent extent = game.Extents[game.World.Extent];

        if (game.World.ExtentType == extent.Etype)
        {
            return true;
        }

        game.World.ExtentType = extent.Etype;
        bool force = false;

        if (game.World.ExtentType == XT_PRINCESS && game.Objects.Regions[INSIDE_OBJECTS][PRINCESS_OBJECT].ObStat != 0)
        {
            Rescue.RescuePrincess(game, output);

            return false;
        }

        if (game.World.ExtentType >= XT_SPECIAL_MIN)
        {
            // Birds, turtles, special.
            if (BossIsMissing(game))
            {
                AimEncounterAtExtentMiddle(game);
                force = true;
            }
        }
        else if (game.World.ExtentType == XT_SPIDER_PIT)
        {
            LoadSpiders(game, output);
        }
        else if (game.World.ExtentType >= XT_MONSTER_ZONE_MIN && flag == 1)
        {
            game.Encounter.X = game.Hero.X;
            game.Encounter.Y = game.Hero.Y;
            force = true;
        }

        if (force)
        {
            StartForcedEncounter(game, output);
        }

        return true;
    }

    // Starts the monsters of the extent the hero is in: loads their pictures and places up to four
    // of them (there must be room before the carrier slots).
    private static void StartForcedEncounter(Data.Game game, IOutput output)
    {
        Extent extent = game.Extents[game.World.Extent];

        game.Encounter.Kind = extent.V3;
        game.Encounter.MixFlag = 0;
        game.Encounter.WeaponType = 0;
        Encounters.LoadEnemySprites(game, output);
        output.FinishSpriteLoad(ENEMY_SLOT);
        game.Actors.Loading = 0;
        game.Encounter.Number = extent.V1;
        game.Actors.Active = FIRST_MONSTER;

        while (game.Encounter.Number != 0 && game.Actors.Active < MONSTER_SLOTS_END)
        {
            if (Encounters.PlaceEncounter(game, (ushort)game.Actors.Active, ENCOUNTER_SPREAD) != 0)
            {
                game.Actors.Active++;
            }

            game.Encounter.Number--;
        }
    }

    // The forced encounter appears in the middle of the extent.
    private static void AimEncounterAtExtentMiddle(Data.Game game)
    {
        Extent extent = game.Extents[game.World.Extent];

        game.Encounter.X = (ushort)((extent.X1 + extent.X2) / 2);
        game.Encounter.Y = (ushort)((extent.Y1 + extent.Y2) / 2);
    }

    // The spider pit loads its spiders at once (no waiting for the disk).
    private static void LoadSpiders(Data.Game game, IOutput output)
    {
        game.Encounter.Kind = SPIDER_ENCOUNTER;
        Encounters.LoadEnemySprites(game, output);
        output.FinishSpriteLoad(ENEMY_SLOT);
        game.Actors.Loading = 0;
    }

    // The necromancer's and the turtle eggs' extents call up their monsters when those are not
    // there (yet, or any more).
    private static bool BossIsMissing(Data.Game game) =>
        (game.World.ExtentType == XT_NECROMANCER || game.World.ExtentType == XT_TURTLE_EGGS)
        && (game.Actors.List[FIRST_MONSTER].Race != game.Extents[game.World.Extent].V3 || game.Actors.Active < MIN_BOSS_ACTORS);

    // Drops the carrier when the hero is outside the carrier extents, and loads the swan, turtle
    // or dragon inside them.
    private static void UpdateCarrier(Data.Game game, IOutput output)
    {
        if (game.World.ExtentType < XT_CARRIER_MIN)
        {
            game.Actors.ActiveCarrier = 0;

            return;
        }

        short carrierFile = game.Extents[game.World.Extent].V3;

        if (game.World.ExtentType == XT_CARRIER
            && (game.Actors.ActiveCarrier == 0 || (game.Hero.Riding == 0 && game.Actors.LoadedActorFile != carrierFile)))
        {
            Encounters.LoadCarrier(game, output, carrierFile);
        }
    }
}
