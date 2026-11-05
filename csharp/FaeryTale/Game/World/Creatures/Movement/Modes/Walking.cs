using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Creatures.Movement;

// Walking, standing still and fighting: sliding on ice, going around obstacles, being blocked.
internal sealed partial class MoveContext
{
    // Blocked for this many frames the hero oscillates; for this many more, he looks frustrated.
    private const sbyte FRUSTRATION_OSCILLATE = 20;
    private const sbyte FRUSTRATION_FRUSTRATED = 40;

    // Step lengths in pixels per frame.
    private const short STEP_TURTLE = 3;
    private const short STEP_BACKWARDS = -2;
    private const short STEP_SLOW_GROUND = 4;
    private const short STEP_SHALLOW = 1;
    private const short STEP_NORMAL = 2;

    /// Water deeper than this (or exactly ENVIRON_SHALLOW) slows the walk.
    private const sbyte ENVIRON_DEEP = 6;

    // Sliding on ice: the top speed (while riding the swan, and not), and the offset used to read
    // the direction's step from the step tables.
    private const short ICE_TOP_SPEED_SWAN = 40;
    private const short ICE_TOP_SPEED = 42;
    private const int ICE_STEP_ORIGIN = 20;
    private const int ICE_STEP_SPEED = 2;
    private const short ICE_X_MARGIN = 8;

    /// A walking frame cycles through 8 (flipping the low 3 bits plays them backwards).
    private const int WALK_FRAMES_MASK = 7;

    /// Snake pictures start here, two frames for each pair of facings.
    private const short SNAKE_FIRST_FRAME = 0x24;

    /// Velocity is kept as four times the movement of the last step.
    private const int VELOCITY_SCALE = 4;

    // The fighting states 6 and 7 are shown as state 8 (the follow-through).
    private const short SWING_STATE_FOLLOW_THROUGH = 8;
    private const short SWING_STATE_FIRST = 6;
    private const short SWING_STATE_SECOND = 7;
    private const int FIRST_NPC_SLOT = 2;

    /// The hero's frame once the way has been blocked for a while: oscillating, then a frustrated one.
    internal static short? FrustratedFrame(sbyte frustration, short cycle)
    {
        if (frustration > FRUSTRATION_FRUSTRATED)
        {
            return FRAME_FRUSTRATED;
        }

        // Swap between the two impatient frames every second tick.
        return frustration > FRUSTRATION_OSCILLATE ? (short)(FRAME_OSCIL + ((cycle >> 1) & 1)) : null;
    }

    /// How far an actor walks in a frame: the hero riding the turtle goes fastest, walking
    /// backwards or through slow ground is slower, and a little water (environ 2 or deeper than 6)
    /// slows to 1.
    internal static short StepLength(bool heroOnTurtle, sbyte environ)
    {
        if (heroOnTurtle)
        {
            return STEP_TURTLE;
        }

        if (environ == ENVIRON_BACKWARDS)
        {
            return STEP_BACKWARDS;
        }

        if (environ == ENVIRON_SLOW)
        {
            return STEP_SLOW_GROUND;
        }

        return environ == ENVIRON_SHALLOW || environ > ENVIRON_DEEP ? STEP_SHALLOW : STEP_NORMAL;
    }

    // The velocity after one more push toward the facing: the wanted step's size added to the
    // current velocity.
    private static (short X, short Y) IceVelocityToward(short facing, sbyte velX, sbyte velY)
    {
        int pushX = MapSteps.StepX(ICE_STEP_ORIGIN, facing, ICE_STEP_SPEED) - ICE_STEP_ORIGIN;
        int pushY = MapSteps.StepY(ICE_STEP_ORIGIN, facing, ICE_STEP_SPEED) - ICE_STEP_ORIGIN;

        return ((short)(velX + pushX), (short)(velY + pushY));
    }

    // Walking: sliding on ice, picking a step, and trying to go around what is in the way.
    private Pc Walking()
    {
        if (environ == ENVIRON_ICE)
        {
            if (SlideOnIce() is { } next)
            {
                return next;
            }

            environ = 0;
        }

        step = StepLength(slot == 0 && game.Hero.Riding == RIDING_TURTLE, environ);
        TestStepFromHere();
        terrainHere = (short)Blocking.BlockedAt(game, (short)game.Actors.TestX, (short)game.Actors.TestY, (short)slot);

        if (slot == 0 && HeroBumpsIntoDoor())
        {
            return Pc.NewLoc;
        }

        return terrainHere != 0 ? Pc.CheckDev1 : Pc.NewLoc;
    }

    // Works out where a step in the facing direction would end.
    private void TestStepFromHere()
    {
        game.Actors.TestX = PosX(actor.AbsX, facing, step);
        game.Actors.TestY = PosY(actor.AbsY, facing, step);
    }

    private bool TestStepBlocked() =>
        Blocking.BlockedAt(game, (short)game.Actors.TestX, (short)game.Actors.TestY, (short)slot) != 0;

    // The hero walking into a door opens it (key doors have their own keys); walking away from one
    // forgets that it was locked. Returns true when the gold key lets him through.
    private bool HeroBumpsIntoDoor()
    {
        if (terrainHere == BLOCK_DOOR)
        {
            KeyDoors.OpenDoorNear(game, output, game.Actors.TestX, game.Actors.TestY, 0);
        }
        else
        {
            game.World.BumpedLockedDoor = 0;
        }

        return game.Hero.Stuff[STUFF_GOLD_KEY_SLOT] != 0 && terrainHere == BLOCK_PASSABLE_WITH_KEY;
    }

    // On ice the actor keeps its momentum: pushes the velocity toward the wanted direction up to a
    // limit and moves by it. Returns where to go next, or null when it hit something and should
    // walk normally this tick.
    private Pc? SlideOnIce()
    {
        step = game.Hero.Riding == RIDING_SWAN ? ICE_TOP_SPEED_SWAN : ICE_TOP_SPEED;

        (short newVelX, short newVelY) = IceVelocityToward(facing, actor.VelX, actor.VelY);

        if (Math.Abs((int)newVelX) < step - ICE_X_MARGIN)
        {
            actor.VelX = (sbyte)newVelX;
        }

        if (Math.Abs((int)newVelY) < step)
        {
            actor.VelY = (sbyte)newVelY;
        }

        game.Actors.TestX = (ushort)(actor.AbsX + actor.VelX / VELOCITY_SCALE);
        game.Actors.TestY = (ushort)(actor.AbsY + actor.VelY / VELOCITY_SCALE);

        if (game.Hero.Riding == RIDING_SWAN)
        {
            Steering.SteerActor(game, 0, -newVelX, -newVelY, Steering.STEER_OFFSET);
            facing = actor.Facing;

            return Pc.NewLoc;
        }

        return TestStepBlocked() ? null : Pc.NewLoc;
    }

    // Blocked: try one direction to the right.
    private Pc CheckDev1()
    {
        facing = (short)((facing + 1) & FACING_MASK);
        TestStepFromHere();

        return TestStepBlocked() ? Pc.CheckDev2 : Pc.NewLoc;
    }

    // Still blocked: try one direction to the left of the original.
    private Pc CheckDev2()
    {
        facing = (short)((facing - 2) & FACING_MASK);
        TestStepFromHere();

        return TestStepBlocked() ? Pc.Blocked : Pc.NewLoc;
    }

    // A free step was found: face that way, pick the walking frame, and move (unless the terrain
    // slows the actor to a halt).
    private Pc NewLoc()
    {
        PickWalkingFrame();

        if (environ == ENVIRON_ICE)
        {
            return Pc.Cpx;
        }

        if (environ == ENVIRON_BACKWARDS)
        {
            // Walking backwards: flipping the three low bits plays the walk frames in reverse order.
            image ^= WALK_FRAMES_MASK;
        }

        LookAtNewTerrain();

        if (ClimbsOutOfWater(environ, terrainHere))
        {
            if (game.Hero.Sector != PIT_SECTOR)
            {
                environ--;
            }

            return Pc.Raise;
        }

        TakeTheStep();

        return Pc.Tail;
    }

    // Faces the actor the way it will step, and picks the walking frame for this tick (a rider and
    // the wraiths keep their one frame).
    private void PickWalkingFrame()
    {
        actor.Facing = (sbyte)facing;
        frameBase = DirectionOffset(facing);
        image = frameBase;

        if (!(game.Hero.Riding != 0 && slot == 0) && actor.Race != RACE_WRAITH)
        {
            image = (short)(image + ((cycle + slot) & WALK_FRAMES_MASK));
        }
    }

    // The terrain at the new place; wraiths and snakes are immune to water, and snakes have their
    // own pictures.
    private void LookAtNewTerrain()
    {
        terrainHere = TerrainAtPoint(game.Actors.TestX, game.Actors.TestY);

        if (actor.Race == RACE_WRAITH || actor.Race == RACE_SNAKE)
        {
            terrainHere = 0;
        }

        if (actor.Race == RACE_SNAKE)
        {
            // Snake pictures: two frames (bit 1 of the cycle) for each pair of facings.
            image = (short)((cycle & 2) + ((facing & 0xfe) << 2) + SNAKE_FIRST_FRAME);
        }
    }

    // Moves to the new place, remembering the step as the velocity (for sliding on ice).
    private void TakeTheStep()
    {
        actor.VelX = (sbyte)((short)(game.Actors.TestX - actor.AbsX) * VELOCITY_SCALE);
        actor.VelY = (sbyte)((short)(game.Actors.TestY - actor.AbsY) * VELOCITY_SCALE);
        actor.AbsX = game.Actors.TestX;
        actor.AbsY = game.Actors.TestY;
        game.Actors.Frustration = 0;
    }

    // Walking found no way through. The hero shows a frustrated frame after a while; others change
    // tactic. (The original placed this label inside the STILL branch.)
    private Pc Blocked()
    {
        if (slot == 0)
        {
            game.Actors.Frustration = unchecked((sbyte)(game.Actors.Frustration + 1));

            if (FrustratedFrame(game.Actors.Frustration, cycle) is { } frame)
            {
                image = frame;
            }

            return Pc.Cpx;
        }

        actor.Tactic = ActorTactic.FRUSTRATED;

        return Pc.Still;
    }

    // Standing still: the frame after the first of the facing's walking frames.
    private Pc Still()
    {
        image = (short)(frameBase + 1);

        if (environ == ENVIRON_ICE)
        {
            return Pc.Cpx;
        }

        terrainHere = TerrainAtPoint(actor.AbsX, actor.AbsY);

        return Pc.Tail;
    }

    // Swinging a weapon (states 0-8): steps through the swing table.
    private Pc Fighting()
    {
        frameBase = DirectionOffset(facing + 8);
        short next = Tables.Entry(Tables.SwingTransitions, state)[game.Rng.Rand4()];

        actor.State = (sbyte)next;
        state = next;

        if ((slot > FIRST_NPC_SLOT && state == SWING_STATE_FIRST) || state == SWING_STATE_SECOND)
        {
            state = SWING_STATE_FOLLOW_THROUGH;
        }

        image = (short)(state + frameBase);
        game.Actors.Frustration = 0;

        return Pc.Cpx;
    }
}
