using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Creatures.Movement;

// What happens at the end of an actor's move: the other states (dying, sinking, frozen ...), the
// sprite frame, the environment, keeping the hero inside the world and working out where to draw.
internal sealed partial class MoveContext
{
    // A falling hero is fallen after this many steps (the tactic counts them, 5 to a frame).
    private const sbyte FALL_STEPS = 30;
    private const short FALL_STEPS_PER_FRAME = 5;
    private const short FALL_FRAMES = 6;

    // A dying actor shows one of two frames, flipping at half the death time.
    private const sbyte DYING_FIRST_HALF = 4;

    // Facings above this (and 0) show the dying actor on its other side.
    private const short DYING_FACING_LIMIT = 4;
    private const short FRAME_DYING_BACK = 80;
    private const short FRAME_DYING_FRONT = 81;

    // Slides on ice and in a fall keep a quarter of the velocity per frame (3/4 when falling).
    private const int VELOCITY_QUARTER = 4;
    private const int FALL_DRAG_NUMERATOR = 3;
    private const int FALL_DRAG_DENOMINATOR = 4;

    /// What the witch leaves behind: her body keeps this much vitality.
    private const short WITCH_BODY_VITALITY = 10;

    // Races with special sprite frames. Races that cannot drown (the wraith and race 3).
    private static readonly sbyte[] DROWN_IMMUNE_RACES = [2, 3];
    private const sbyte RACE_SPECTRE = 8;
    private const sbyte RACE_SPECTRE_ALIVE = 7;
    private const short SPECTRE_DEAD_FRAME = 0x3f;
    private const short SPECTRE_BASE_FRAME_A = 0x25;
    private const short SPECTRE_BASE_FRAME_B = 0x28;
    private const short SPECTRE_BASE_FRAME_C = 0x30;
    private const int SPECTRE_FRAME_GROUPS = 3;

    // The spectre cycles through 4 images two apart, and skips one from the image after this.
    private const int SPECTRE_CYCLE_MASK = 3;
    private const short SPECTRE_SKIP_FRAME = 4;

    /// An actor drowning loses vitality every eighth frame.
    private const int DROWN_PERIOD_MASK = 7;

    // Where an actor is drawn relative to its position: offsets for a swan rider, for the raft,
    // carriers and dragon, and for everyone else.
    private const int ANCHOR_SWAN_X = 32;
    private const int ANCHOR_SWAN_Y = 40;
    private const int ANCHOR_LARGE_X = 16;
    private const int ANCHOR_LARGE_Y = 16;
    private const int ANCHOR_PERSON_X = 8;
    private const int ANCHOR_PERSON_Y = 26;

    // Burning in lava, and running out of breath: the death message for CheckDeath.
    private const int DEATH_LAVA = 27;
    private const int DEATH_DROWN = 6;

    // The remaining states: dying, dead, frozen, oscillating, asleep and falling.
    private Pc OtherStates()
    {
        if (state == ActorState.DYING)
        {
            DyingFrame();
        }
        else if (state == ActorState.DEAD || state == FROZEN)
        {
            image = FRAME_FLAT;
        }
        else if (state == OSCIL)
        {
            image = (short)(FRAME_OSCIL + (cycle & 1));
        }
        else if (state == OSCIL + 1)
        {
            image = FRAME_OSCIL;
        }
        else if (state == ActorState.SLEEP)
        {
            image = FRAME_SLEEP;
        }
        else if (state == ActorState.FALL && actor.Tactic < FALL_STEPS)
        {
            FallingFrame();
        }

        return Pc.Cpx;
    }

    // Works out the terrain under the actor, and carries on sliding if it is on ice.
    private Pc Cpx()
    {
        terrainHere = TerrainAtPoint(actor.AbsX, actor.AbsY);

        if (environ == ENVIRON_ICE)
        {
            actor.AbsX = (ushort)(actor.AbsX + actor.VelX / VELOCITY_QUARTER);
            actor.AbsY = (ushort)(actor.AbsY + actor.VelY / VELOCITY_QUARTER);
        }

        return Pc.Tail;
    }

    // After the movement: finishes a death, then turns the terrain under the actor into its
    // environ (water depth, ice, the pit).
    private Pc Tail()
    {
        if (state == ActorState.DYING)
        {
            FinishDying();
        }

        SettleEnviron();

        if (environ == 0 && state == ActorState.SINK)
        {
            actor.State = ActorState.STILL;
        }

        return Pc.Raise;
    }

    // Stores the environ, adjusts the sprite for some enemies, keeps the hero inside the world and
    // scrolls the map to it, and applies lava and drowning damage.
    private Pc Raise()
    {
        if (game.World.NewRegion >= NO_REGION)
        {
            actor.Environ = environ;
        }

        if (actor.Kind == ActorKind.ENEMY)
        {
            EnemyFrame((sbyte)actor.Race);
        }

        actor.Index = (sbyte)image;

        if (slot == 0)
        {
            HeroWrap();
            game.Hero.X = actor.AbsX;
            game.Hero.Y = actor.AbsY;
            MapCamera.MoveCameraTo(game.World, game.Hero.X, game.Hero.Y);
            game.Hero.SafeFlag = (sbyte)terrainHere;
        }

        if (actor.Vitality == 0)
        {
            return Pc.Statc;
        }

        TakeLavaDamage();
        TakeDrowningDamage();

        return Pc.Statc;
    }

    // Where the actor is on screen: relative to the map, with the sprite's anchor offset.
    private void Statc()
    {
        (int offsetX, int offsetY) = AnchorFor();

        actor.RelX = (ushort)MapSteps.WrapCoordinate(actor.AbsX - game.World.MapX - offsetX);
        actor.RelY = (ushort)MapSteps.WrapCoordinate(actor.AbsY - game.World.MapY - offsetY);
    }

    private (int X, int Y) AnchorFor()
    {
        if (actor.Kind == ActorKind.CARRIER && game.Hero.Riding == RIDING_SWAN)
        {
            return (ANCHOR_SWAN_X, ANCHOR_SWAN_Y);
        }

        bool large = actor.Kind == ActorKind.RAFT || actor.Kind == ActorKind.CARRIER || actor.Kind == ActorKind.DRAGON;

        return large ? (ANCHOR_LARGE_X, ANCHOR_LARGE_Y) : (ANCHOR_PERSON_X, ANCHOR_PERSON_Y);
    }
}
