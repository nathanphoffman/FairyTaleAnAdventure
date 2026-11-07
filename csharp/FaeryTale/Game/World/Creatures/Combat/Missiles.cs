using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Creatures.Combat;

/// Arrows and fireballs in flight: moving, hitting actors and stopping at walls, and putting the
/// visible missiles in the actor list so they get drawn.
public static class Missiles
{
    internal const sbyte MISSILE_NONE = 0;
    internal const sbyte MISSILE_ROCK = 2;
    internal const sbyte MISSILE_SPENT = 3;

    /// A missile gives up after this many frames in flight.
    internal const sbyte MAX_FLIGHT = 40;

    /// In Remastered and Reimagined the hero's arrows and fireballs fly this many times as long,
    /// because the view shows so much more than the old screen did.
    internal const int HERO_FLIGHT_FACTOR = 2;

    // Terrain a missile stops at: solid wall and the map edge.
    private const int WALL_TERRAIN_SOLID = 1;
    private const int WALL_TERRAIN_EDGE = 15;

    // How close a missile has to be to hit (arrows and rocks, fireballs).
    private const short ARROW_HIT_RANGE = 6;
    private const short FIREBALL_HIT_RANGE = 9;

    // Other actors are treated as having this much bravery when it comes to dodging.
    private const short OTHER_DODGE = 20;
    private const int DODGE_RANGE = 512;
    private const int MISSILE_DAMAGE_BASE = 4;

    // HitActor attacker codes for an arrow and for a fireball.
    private const int BY_ARROW = -1;
    internal const int BY_FIREBALL = -2;

    private const int RAFT = 1;
    private const int MISSILE_COUNT = 6;
    private const short ACTOR_SLOTS = 20;

    // A missile is added to the actor list if its sprite would be inside this box on the screen.
    private const short SCREEN_LEFT = -20;
    private const short SCREEN_RIGHT = 340;
    private const short SCREEN_TOP = -20;
    private const short SCREEN_BOTTOM = 190;

    // Remastered and Reimagined show far more of the world than the old screen did, so missiles are
    // drawn over the same wider area as the people and things (see Objects.WIDE_SHOWN).
    private const short WIDE_LEFT = -300;
    private const short WIDE_RIGHT = 620;
    private const short WIDE_TOP = -260;
    private const short WIDE_BOTTOM = 420;

    // The sprite's centre is this far from its map position.
    private const int SPRITE_OFFSET_X = 9;
    private const int SPRITE_OFFSET_Y = 26;
    private const sbyte MISSILE_ROCK_IMAGE = 0x59;
    private const sbyte MISSILE_SPENT_IMAGE = 0x58;
    private const byte NO_RACE = 0xff;

    /// Moves the six arrows and fireballs; they hit actors in the way and stop at walls. Nothing
    /// moves while time is frozen.
    public static void AdvanceMissiles(Data.Game game, IOutput output)
    {
        if (game.World.FreezeTimer != 0)
        {
            return;
        }

        for (int position = 0; position < MISSILE_COUNT; position++)
        {
            Fly(game, output, position);
        }
    }

    /// Adds the visible missiles to the end of the actor list as objects so the drawing draws
    /// them. Stops when the list is full.
    public static void QueueMissileSprites(Data.Game game) => QueueSprites(game, game.Actors.Missiles);

    /// Adds the given missiles (the six of the game, or the fireball and flames of the inferno
    /// skull) to the actor list in the same way.
    public static void QueueSprites(Data.Game game, IReadOnlyList<Missile> missiles)
    {
        foreach (Missile missile in missiles)
        {

            if (missile.MissileType == MISSILE_NONE)
            {
                continue;
            }

            short xStart = (short)(missile.AbsX - game.World.MapX - SPRITE_OFFSET_X);
            short yStart = (short)(missile.AbsY - game.World.MapY - SPRITE_OFFSET_Y);

            bool wide = game.Experience != Experience.Original;

            if (xStart > (wide ? WIDE_LEFT : SCREEN_LEFT) && xStart < (wide ? WIDE_RIGHT : SCREEN_RIGHT) && yStart > (wide ? WIDE_TOP : SCREEN_TOP) && yStart < (wide ? WIDE_BOTTOM : SCREEN_BOTTOM))
            {
                AddMissileActor(game.Actors, missile, xStart, yStart);
                game.Actors.Drawn++;

                if (game.Actors.Drawn >= ACTOR_SLOTS)
                {
                    return;
                }
            }
        }
    }

    /// Whether a missile at dist from a target hits it: within range, and for the hero (only for
    /// the first missile slot, as in the original) a roll above the hero's bravery is needed. The
    /// roll is only drawn when it is needed.
    internal static bool MissileConnects(int slot, short dist, short range, short dodge, Func<int> roll) =>
        (slot != 0 || roll() > dodge) && dist < range;

    /// Whether the missile is unused, spent, standing still (s is its doubled speed) or has been
    /// in the air too long. Counts one more tick of flight when it has not expired.
    internal static bool MissileExpired(Missile missile, short s, int maxFlight = MAX_FLIGHT)
    {
        sbyte flight = missile.TimeOfFlight;

        if (missile.MissileType == MISSILE_NONE || missile.MissileType == MISSILE_SPENT || s == 0)
        {
            return true;
        }

        missile.TimeOfFlight = unchecked((sbyte)(flight + 1));

        return flight > maxFlight;
    }

    // How long a missile may stay in the air: longer for the hero's in Remastered and Reimagined.
    private static int MaxFlightOf(Data.Game game, Missile missile) =>
        game.Experience != Experience.Original && missile.Archer == 0 ? MAX_FLIGHT * HERO_FLIGHT_FACTOR : MAX_FLIGHT;

    // One missile's flight for this frame.
    private static void Fly(Data.Game game, IOutput output, int position)
    {
        Missile missile = game.Actors.Missiles[position];
        short s = (short)(missile.Speed * 2);

        if (MissileExpired(missile, s, MaxFlightOf(game, missile)))
        {
            missile.MissileType = MISSILE_NONE;

            return;
        }

        short x = (short)missile.AbsX;
        short y = (short)missile.AbsY;
        int terrain = MapLookup.TerrainAt(game, x, y);

        if (terrain == WALL_TERRAIN_SOLID || terrain == WALL_TERRAIN_EDGE)
        {
            missile.MissileType = MISSILE_NONE;
            s = 0;
        }

        int facing = missile.Direction;

        HitFirstActorInPath(game, output, position, missile, x, y);
        missile.AbsX = (ushort)MapSteps.StepX(missile.AbsX, facing, s);
        missile.AbsY = (ushort)MapSteps.StepY(missile.AbsY, facing, s);
    }

    // Hits the first actor close enough to the missile at (x, y) and stops the missile. position
    // is the missile's own index; the hero (actor 0) gets a dodge chance from bravery, others a
    // fixed one.
    private static void HitFirstActorInPath(Data.Game game, IOutput output, int position, Missile missile, short x, short y)
    {
        int facing = missile.Direction;
        bool isRock = missile.MissileType == MISSILE_ROCK;
        short range = isRock ? FIREBALL_HIT_RANGE : ARROW_HIT_RANGE;

        // The loop bound is read again each time, as in the original.
        for (int target = 0; target < game.Actors.Active; target++)
        {
            short dodge = target == 0 ? game.Hero.Brave : OTHER_DODGE;

            if (!MissileCanHit(game.Actors, missile, target))
            {
                continue;
            }

            short dist = Melee.DistanceToPoint(game.Actors.List[target], x, y);

            // The original tests the missile's own index here, not the target's.
            if (!MissileConnects(position, dist, range, dodge, () => game.Rng.BitRand(DODGE_RANGE)))
            {
                continue;
            }

            short damage = (short)(game.Rng.Rand8() + MISSILE_DAMAGE_BASE);

            Hits.HitActor(game, output, isRock ? BY_FIREBALL : BY_ARROW, target, facing, damage);
            missile.Speed = 0;

            if (isRock)
            {
                missile.MissileType = MISSILE_SPENT;
            }

            return;
        }
    }

    // A missile cannot hit the raft, its own archer, the dead or a carrier.
    private static bool MissileCanHit(Actors actors, Missile missile, int target)
    {
        Shape actor = actors.List[target];

        return !(target == RAFT || missile.Archer == target || actor.State == ActorState.DEAD || actor.Kind == ActorKind.CARRIER);
    }

    // Makes the next free actor a flying object that looks like the missile: an arrow pointing the
    // way it flies, a rock, or the spent rock left behind.
    private static void AddMissileActor(Actors actors, Missile missile, short xStart, short yStart)
    {
        Shape actor = actors.List[actors.Drawn];

        actor.AbsX = missile.AbsX;
        actor.AbsY = missile.AbsY;
        actor.RelX = (ushort)xStart;
        actor.RelY = (ushort)yStart;
        actor.Kind = ActorKind.OBJECTS;
        actor.Weapon = 0;
        actor.Index = missile.Direction;
        actor.Race = NO_RACE;

        if (missile.MissileType == MISSILE_SPENT)
        {
            actor.Index = MISSILE_SPENT_IMAGE;
        }
        else if (missile.MissileType == MISSILE_ROCK)
        {
            actor.Index = unchecked((sbyte)(actor.Index + MISSILE_ROCK_IMAGE));
        }
    }
}
