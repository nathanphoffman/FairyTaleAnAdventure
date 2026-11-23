using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Movement;

/// MoveContext: the end of the frame for each kind of actor.
internal sealed partial class MoveContext
{
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
}
