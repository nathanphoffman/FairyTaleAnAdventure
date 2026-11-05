using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Land;

/// Sleeping in a bed: standing on a bed tile indoors for a while puts a tired hero to sleep.
public static class Bed
{
    /// What standing on a bed for a frame leads to.
    internal enum BedAction
    {
        Nothing,

        /// Stood on the bed long enough, but is too rested to sleep.
        NotTired,

        FallAsleep,
    }

    /// The indoors region.
    private const ushort INSIDE_REGION = 8;

    // Tiles are 16 pixels wide and 32 high.
    private const int TILE_WIDTH_SHIFT = 4;
    private const int TILE_HEIGHT_SHIFT = 5;

    // The tiles that are beds, and how long to stand still on one before sleeping.
    private static readonly byte[] BED_TILES = [161, 52, 162, 53];
    internal const short BED_WAIT = 30;
    internal const short TOO_RESTED = 50;
    private const int EVENT_NOT_TIRED = 25;
    private const int EVENT_SLEEPING = 26;

    /// Sleeping puts the hero at the bottom of the tile.
    private const ushort BED_Y_MASK = 0x1f;

    /// Copies the hero's position, and puts the hero to bed after standing on a bed for 30 ticks.
    public static void CheckBed(Data.Game game, IOutput output)
    {
        Shape hero = game.Actors.List[0];

        if (hero.State != ActorState.STILL)
        {
            game.World.BedWait = 0;
        }

        game.Hero.X = hero.AbsX;
        game.Hero.Y = hero.AbsY;

        if (game.World.Region != INSIDE_REGION)
        {
            return;
        }

        // Hero pixel position to tile column and row (shift = divide by the tile size).
        byte tile = game.Map.SectorByte(MapLookup.TileIndex(game, game.Hero.X >> TILE_WIDTH_SHIFT, game.Hero.Y >> TILE_HEIGHT_SHIFT));
        (short wait, BedAction action) = BedStep(game.World.BedWait, BED_TILES.Contains(tile), game.Hero.Fatigue);

        game.World.BedWait = wait;

        if (action == BedAction.NotTired)
        {
            TextRoutines.ShowEvent(game, output, EVENT_NOT_TIRED);
        }
        else if (action == BedAction.FallAsleep)
        {
            TextRoutines.ShowEvent(game, output, EVENT_SLEEPING);
            hero.AbsY |= BED_Y_MASK;
            game.Hero.Y = hero.AbsY;
            hero.State = ActorState.SLEEP;
        }
    }

    // One frame indoors: wait counts how long the hero has stood on a bed; after BED_WAIT frames he
    // sleeps if tired enough. Stepping off the bed starts the count again.
    internal static (short Wait, BedAction Action) BedStep(short wait, bool onBed, short fatigue)
    {
        if (!onBed)
        {
            return (0, BedAction.Nothing);
        }

        wait++;

        if (wait != BED_WAIT)
        {
            return (wait, BedAction.Nothing);
        }

        return (wait, fatigue < TOO_RESTED ? BedAction.NotTired : BedAction.FallAsleep);
    }
}
