using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Revive;

/// Putting the hero back at the last safe spot with full health.
public static class SafeSpot
{
    /// Where the hero is on the screen: the map is centred on the hero.
    internal const ushort CAMERA_OFFSET_X = 144;
    internal const ushort CAMERA_OFFSET_Y = 90;

    // Vitality of a brother who sets out: 15 plus a quarter of the bravery.
    private const short BASE_VITALITY = 15;
    private const short BRAVERY_DIVISOR = 4;

    /// The hero's body is marked with this race.
    private const byte HERO_RACE = 255;

    /// Actors active when a brother starts: the hero, the raft and the first person.
    private const short START_ACTORS = 3;

    // Time of day when a brother starts or revives, and the light then.
    internal const ushort START_DAYNIGHT = 8000;
    internal const ushort START_LIGHT = 300;

    /// The hero (re)appears at the last safe spot, and the region there is loaded.
    public static void PutHeroAtSafeSpot(Data.Game game, IOutput output)
    {
        MoveToSafeSpot(game.Hero, game.Actors.List[0], game.World);
        Regions.LoadRegionNow(game, output);
    }

    /// Fresh health, a new day, and no hunger or tiredness.
    public static void RestoreHero(Data.Game game)
    {
        RestoreStats(game.Hero, game.Actors.List[0], game.World);

        game.Actors.Active = START_ACTORS;
        Menus.UpdateMenuAvailability(game);
        TextRoutines.QueuePrint(game.Ui, PrintRequest.STATS);
        TextRoutines.QueuePrint(game.Ui, PrintRequest.VITALITY);
    }

    /// The hero's position becomes the safe spot, the camera is centred on it, and its region is
    /// the one to load next.
    internal static void MoveToSafeSpot(Hero hero, Shape body, World world)
    {
        hero.X = hero.SafeX;
        body.AbsX = hero.SafeX;
        hero.Y = hero.SafeY;
        body.AbsY = hero.SafeY;
        world.MapX = (ushort)(hero.X - CAMERA_OFFSET_X);
        world.MapY = (ushort)(hero.Y - CAMERA_OFFSET_Y);
        world.NewRegion = hero.SafeR;
    }

    /// Full health for the bravery, standing still on dry land, the morning clock, and no hunger
    /// or fatigue.
    internal static void RestoreStats(Hero hero, Shape body, World world)
    {
        body.Vitality = (short)(BASE_VITALITY + hero.Brave / BRAVERY_DIVISOR);
        body.Environ = 0;
        body.State = ActorState.STILL;
        body.Race = HERO_RACE;
        world.DayClock = START_DAYNIGHT;
        world.LightLevel = START_LIGHT;
        hero.Hunger = 0;
        hero.Fatigue = 0;
    }
}
