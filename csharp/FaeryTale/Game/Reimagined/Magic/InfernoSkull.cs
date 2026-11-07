using FaeryTale.Game.Creatures.Combat;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Reimagined;

/// The inferno skull spell (Reimagined): a fireball that flies the way the hero faces and, where
/// it hits an enemy or a wall or runs out of flight, bursts into flame and hurts every enemy near.
public static class InfernoSkull
{
    /// The skull's place in the pack (the bird totem's, which Reimagined does not use for the map).
    public const int SKULL_SLOT = 13;

    private const int FIREBALL_SPEED = 8;
    private const int FIREBALL_HIT_RANGE = 12;
    private const int MAX_FLIGHT_FRAMES = 40;

    /// How far from where the fireball burst enemies are burned, and how many flames show.
    internal const int BLAST_RADIUS = 80;
    private const int FLAME_COUNT = 14;
    internal const int FLAME_FRAMES = 30;

    private const int BLAST_DAMAGE_BASE = 30;
    private const int FIRE_START_HEIGHT = 10;
    private const int FACING_COUNT = 8;
    private const int RANDOM_RANGE = 256;

    // The terrain a fireball bursts against: solid wall and the map edge, as for arrows.
    private const int WALL_TERRAIN = 1;
    private const int EDGE_TERRAIN = 15;

    /// The name of an item of the pack: the skull's in Reimagined at the totem's place, else the usual.
    public static string NameOf(Data.Game game, int slot) =>
        game.Experience == Experience.Reimagined && slot == SKULL_SLOT ? "Inferno Skull" : Tables.ItemList[slot].Name;

    /// The labels of a menu: Reimagined's Magic menu says Fire where the original says Totem.
    public static byte[] MenuLabels(Data.Game game, byte[] labels) =>
        game.Experience == Experience.Reimagined && labels == Tables.Label6 ? Tables.Label6Reimagined : labels;

    /// Casts the spell: sends a fireball from the hero the way he faces.
    public static void Cast(Data.Game game)
    {
        Shape hero = game.Actors.List[0];

        game.Inferno.Fireball = new Missile
        {
            AbsX = hero.AbsX,
            AbsY = (ushort)(hero.AbsY - FIRE_START_HEIGHT),
            MissileType = Missiles.MISSILE_ROCK,
            Direction = hero.Facing,
        };
    }

    /// Moves the fireball and ages the flames, once a frame (nothing moves while time is frozen).
    public static void Advance(Data.Game game, IOutput output)
    {
        if (!game.Inferno.Active || game.World.FreezeTimer != 0)
        {
            return;
        }

        if (game.Inferno.Fireball is { } fireball)
        {
            Fly(game, output, fireball);
        }

        AgeFlames(game.Inferno);
    }

    // The fireball moves on, and bursts when it meets a wall or an enemy or has flown far enough.
    private static void Fly(Data.Game game, IOutput output, Missile fireball)
    {
        fireball.TimeOfFlight++;

        int terrain = MapLookup.TerrainAt(game, fireball.AbsX, fireball.AbsY);
        bool hitWall = terrain == WALL_TERRAIN || terrain == EDGE_TERRAIN;

        if (hitWall || fireball.TimeOfFlight > MAX_FLIGHT_FRAMES || EnemyNear(game, fireball))
        {
            Burst(game, output, fireball);

            return;
        }

        fireball.AbsX = (ushort)MapSteps.StepX(fireball.AbsX, fireball.Direction, FIREBALL_SPEED);
        fireball.AbsY = (ushort)MapSteps.StepY(fireball.AbsY, fireball.Direction, FIREBALL_SPEED);
    }

    // Whether an enemy is close enough to the fireball to be hit by it.
    private static bool EnemyNear(Data.Game game, Missile fireball) =>
        Enumerable.Range(1, game.Actors.Active - 1)
            .Any(index => IsLiveEnemy(game.Actors.List[index])
                && Melee.DistanceToPoint(game.Actors.List[index], (short)fireball.AbsX, (short)fireball.AbsY) < FIREBALL_HIT_RANGE);

    private static bool IsLiveEnemy(Shape actor) =>
        actor.Kind == ActorKind.ENEMY && actor.State != ActorState.DEAD && actor.Vitality > 0;

    // The fireball is gone: flames are lit round the spot and every enemy near it is burned.
    private static void Burst(Data.Game game, IOutput output, Missile fireball)
    {
        game.Inferno.Fireball = null;

        for (int flame = 0; flame < FLAME_COUNT; flame++)
        {
            game.Inferno.Flames.Add(new Missile
            {
                AbsX = (ushort)(fireball.AbsX + RandomOffset(game)),
                AbsY = (ushort)(fireball.AbsY + RandomOffset(game)),
                MissileType = Missiles.MISSILE_ROCK,
                Direction = (sbyte)(flame % FACING_COUNT),
            });
        }

        BurnEnemiesNear(game, output, fireball);
    }

    // A random distance from the middle of the burst, up to the blast radius either way.
    private static int RandomOffset(Data.Game game) => game.Rng.Rand256() * (2 * BLAST_RADIUS + 1) / RANDOM_RANGE - BLAST_RADIUS;

    // Every living enemy within the blast radius takes the blast's damage (the hero and friends do not).
    private static void BurnEnemiesNear(Data.Game game, IOutput output, Missile fireball)
    {
        for (int target = 1; target < game.Actors.Active; target++)
        {
            Shape actor = game.Actors.List[target];

            if (IsLiveEnemy(actor) && Melee.DistanceToPoint(actor, (short)fireball.AbsX, (short)fireball.AbsY) < BLAST_RADIUS)
            {
                short damage = (short)(BLAST_DAMAGE_BASE + game.Rng.Rand8());

                Hits.HitActor(game, output, Missiles.BY_FIREBALL, target, fireball.Direction, damage);
            }
        }
    }

    // Flames burn for a while, spinning their pictures, then go out.
    private static void AgeFlames(InfernoBlast blast)
    {
        foreach (Missile flame in blast.Flames)
        {
            flame.TimeOfFlight++;
            flame.Direction = (sbyte)((flame.Direction + 1) % FACING_COUNT);
        }

        blast.Flames.RemoveAll(flame => flame.TimeOfFlight > FLAME_FRAMES);
    }

    /// Puts the fireball and the flames in the actor list so they are drawn.
    public static void QueueSprites(Data.Game game)
    {
        if (!game.Inferno.Active)
        {
            return;
        }

        List<Missile> all = [.. game.Inferno.Flames];

        if (game.Inferno.Fireball is { } fireball)
        {
            all.Add(fireball);
        }

        Missiles.QueueSprites(game, all);
    }
}
