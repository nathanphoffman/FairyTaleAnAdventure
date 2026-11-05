using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Ui;

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

    // A dying actor flips between two frames, which way round depends on its facing; when the
    // countdown (the tactic) ends it is dead and lies flat.
    private void DyingFrame()
    {
        bool facingBack = facing == 0 || facing > DYING_FACING_LIMIT;

        if (actor.Tactic > DYING_FIRST_HALF)
        {
            image = facingBack ? FRAME_DYING_BACK : FRAME_DYING_FRONT;
        }
        else if (actor.Tactic > 0)
        {
            image = facingBack ? FRAME_DYING_FRONT : FRAME_DYING_BACK;
        }
        else
        {
            actor.State = ActorState.DEAD;
            image = FRAME_FLAT;
        }

        game.Actors.Frustration = 0;
    }

    // One step of the fall into the pit: the next falling frame, and the slide slows down.
    private void FallingFrame()
    {
        terrainHere = (short)(actor.Tactic / FALL_STEPS_PER_FRAME + game.Hero.Brother * FALL_FRAMES);
        image = terrainHere >= 0 && terrainHere < ObjectTables.Fallstates.Length ? ObjectTables.Fallstates[terrainHere] : (short)0;
        actor.Tactic++;
        actor.VelX = (sbyte)(actor.VelX * FALL_DRAG_NUMERATOR / FALL_DRAG_DENOMINATOR);
        actor.VelY = (sbyte)(actor.VelY * FALL_DRAG_NUMERATOR / FALL_DRAG_DENOMINATOR);
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

    // A dying actor counts down its tactic; at 0 it is dead. The witch leaves her body and the
    // talisman, the lasso keeper the lasso.
    private void FinishDying()
    {
        actor.Tactic = unchecked((sbyte)(actor.Tactic - 1));

        if (actor.Tactic != 0)
        {
            return;
        }

        actor.State = ActorState.DEAD;

        if (actor.Race == RACE_WITCH)
        {
            actor.Race = RACE_WITCH_BODY;
            actor.Vitality = WITCH_BODY_VITALITY;
            actor.State = ActorState.STILL;
            actor.Weapon = 0;
            Objects.LeaveItem(game.Objects, game.Actors, (short)slot, TALISMAN);
        }

        if (actor.Race == RACE_LASSO_CARRIER)
        {
            Objects.LeaveItem(game.Objects, game.Actors, (short)slot, LASSO);
        }
    }

    // Turns the terrain under the actor into its environ: water depth, slow ground, ice, going
    // backwards, or falling into the pit.
    private void SettleEnviron()
    {
        short j = terrainHere;

        if (slot == 0 && game.Actors.RaftNearness != 0)
        {
            environ = 0; // can't drown on the raft
        }
        else if (j == 0)
        {
            environ = 0;
        }
        else if (j == TERRAIN_SLOW)
        {
            environ = ENVIRON_SLOW;
        }
        else if (j == TERRAIN_ICE)
        {
            environ = ENVIRON_ICE;
        }
        else if (j == TERRAIN_BACKWARDS)
        {
            environ = ENVIRON_BACKWARDS;
        }
        else if (j == TERRAIN_PIT && slot == 0 && game.World.ExtentType == PIT_XTYPE)
        {
            FallIntoPit();
        }
        else if (j == TERRAIN_SHALLOW)
        {
            environ = ENVIRON_SHALLOW;
        }
        else if (j == TERRAIN_WADE)
        {
            environ = ENVIRON_WADE;
        }
        else if (j == TERRAIN_SLOW_SINK || j == TERRAIN_DEEP_WATER)
        {
            Wade(j);
        }
    }

    // The hero steps on the pit floor: starts to fall (once) and slides.
    private void FallIntoPit()
    {
        if (actor.State != ActorState.FALL)
        {
            image = ObjectTables.Fallstates[game.Hero.Brother * FALL_FRAMES];
            actor.State = ActorState.FALL;
            actor.Tactic = 0;
            game.Hero.Luck = unchecked((short)(game.Hero.Luck - PIT_LUCK_COST));
            Music.SetMood(game, 1);
        }

        environ = ENVIRON_ICE;
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

    // While lava is flowing the actor in deep lava dies and one wading in it is hurt, unless the
    // hero carries the fireproof item.
    private void TakeLavaDamage()
    {
        if (game.Actors.FieryDeath == 0)
        {
            return;
        }

        if (slot == 0 && game.Hero.Stuff[STUFF_FIREPROOF] != 0)
        {
            actor.Environ = 0;
        }
        else if (actor.Environ > ENVIRON_SINKING)
        {
            actor.Vitality = 0;
        }
        else if (actor.Environ > ENVIRON_SHALLOW)
        {
            actor.Vitality = unchecked((short)(actor.Vitality - 1));
        }

        Encounters.CheckDeath(game, output, slot, DEATH_LAVA);
    }

    // A drowning actor loses a point of vitality every eighth frame (wraiths and snakes are immune).
    private void TakeDrowningDamage()
    {
        if (actor.Environ == ENVIRON_DROWNED && (cycle & DROWN_PERIOD_MASK) == 0 && !DROWN_IMMUNE_RACES.Contains((sbyte)actor.Race))
        {
            actor.Vitality = unchecked((short)(actor.Vitality - 1));
            Encounters.CheckDeath(game, output, slot, DEATH_DROWN);
        }
    }

    // The sprite frames of snakes (race 4), the spectre (8) and the race-7 enemy at zero vitality.
    private void EnemyFrame(sbyte race)
    {
        short currentState = actor.State;

        if (race == RACE_SNAKE && currentState < ActorState.WALKING)
        {
            image = (short)((cycle & 1) + DirectionOffset(facing));
        }
        else if (race == RACE_SNAKE && currentState < ActorState.DYING)
        {
            image = (short)(((cycle / 2) & 1) + DirectionOffset(facing));
        }
        else if (race == RACE_SPECTRE)
        {
            SpectreFrame(currentState);
        }
        else if (race == RACE_SPECTRE_ALIVE && actor.Vitality == 0)
        {
            // The original has "state == STILL" here: a comparison that does nothing (an
            // assignment may have been meant). Kept as it was.
            image = 1;
        }
    }

    private void SpectreFrame(short currentState)
    {
        if (currentState == ActorState.DEAD)
        {
            actor.AbsX = 0;

            return;
        }

        if (currentState == ActorState.DYING)
        {
            image = SPECTRE_DEAD_FRAME;

            return;
        }

        image = (short)((cycle & SPECTRE_CYCLE_MASK) * 2);

        if (image > SPECTRE_SKIP_FRAME)
        {
            image--;
        }

        switch (slot % SPECTRE_FRAME_GROUPS)
        {
            case 0:
                image = SPECTRE_BASE_FRAME_A;
                break;

            case 1:
                image += SPECTRE_BASE_FRAME_B;
                break;

            default:
                image += SPECTRE_BASE_FRAME_C;
                break;
        }
    }

    // The hero leaving the edge of the world comes back on the other side (outdoor regions only);
    // whatever the hero rides is moved along.
    private void HeroWrap()
    {
        if (game.World.Region >= OUTDOOR_REGIONS)
        {
            return;
        }

        if (actor.AbsX < WORLD_MIN)
        {
            actor.AbsX = WORLD_MAX;
        }
        else if (actor.AbsX > WORLD_MAX)
        {
            actor.AbsX = WORLD_MIN;
        }
        else if (actor.AbsY < WORLD_MIN)
        {
            actor.AbsY = WORLD_MAX;
        }
        else if (actor.AbsY > WORLD_MAX)
        {
            actor.AbsY = WORLD_MIN;
        }
        else
        {
            return;
        }

        if (game.Hero.Riding > 1)
        {
            game.Actors.List[CARRIER_SLOT].AbsX = actor.AbsX;
            game.Actors.List[CARRIER_SLOT].AbsY = actor.AbsY;
        }
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
