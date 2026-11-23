using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Movement;

/// MoveContext: dying, falling and taking damage.
internal sealed partial class MoveContext
{
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
}
