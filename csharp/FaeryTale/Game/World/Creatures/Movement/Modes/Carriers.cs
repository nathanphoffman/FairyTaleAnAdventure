using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;

namespace FaeryTale.Game.Creatures.Movement;

// The raft, the swan and the turtle (when they carry the hero, and where the turtle swims on its
// own), and people standing about (set figures).
internal sealed partial class MoveContext
{
    /// The swan carries the hero once he is near, with the swan on its slot and the lasso in the pack.
    internal static bool SwanCarries(short nearness, short carrierSlot, bool hasLasso) =>
        nearness != 0 && carrierSlot == CARRIER_SLOT && hasLasso;

    /// The turtle carries the hero whenever he is near and on its slot.
    internal static bool TurtleCarries(short nearness, short carrierSlot) => nearness != 0 && carrierSlot == CARRIER_SLOT;

    /// The turtle's direction of swimming: straight on while there is deep water ahead, otherwise
    /// one step right, then two to the left of that, then one more to the left; the last is used
    /// whether or not it is water. waterAhead(dir) says if the turtle could swim that way.
    internal static short TurtleHeading(short dir, Func<short, bool> waterAhead)
    {
        if (waterAhead(dir))
        {
            return dir;
        }

        short right = (short)((dir + 1) & FACING_MASK);

        if (waterAhead(right))
        {
            return right;
        }

        short left = (short)((right - 2) & FACING_MASK);

        return waterAhead(left) ? left : (short)((left - 1) & FACING_MASK);
    }

    /// The witch's dying and dead frames come two further on than other people's.
    internal static short WitchExtra(bool witch) => witch ? WITCH_EXTRA_OFFSET : (short)0;

    // The swan (sprite file 11) or the turtle: carries the hero when the hero is close enough.
    private Pc Carrier()
    {
        environ = 0;
        terrainHere = 0;

        if (game.Actors.LoadedActorFile == SWAN_FILE)
        {
            SwanMoves();
        }
        else
        {
            TurtleMoves();
        }

        Extents.CenterExtentOn(game, step, (short)game.Actors.TestX, (short)game.Actors.TestY);

        return Pc.Raise;
    }

    // The swan goes where the hero goes if he is near and has the lasso, otherwise it stays.
    private void SwanMoves()
    {
        Shape hero = game.Actors.List[0];

        if (SwanCarries(game.Actors.RaftNearness, game.Actors.CarrierSlot, game.Hero.Stuff[STUFF_SWAN_ITEM] != 0))
        {
            facing = hero.Facing;
            actor.AbsX = hero.AbsX;
            game.Actors.TestX = hero.AbsX;
            actor.AbsY = hero.AbsY;
            game.Actors.TestY = hero.AbsY;
            game.Hero.Riding = RIDING_SWAN;
        }
        else
        {
            game.Actors.TestX = actor.AbsX;
            game.Actors.TestY = actor.AbsY;
        }

        image = facing;
        step = 0;
    }

    // The turtle carries the hero when he is near, otherwise it swims on its own.
    private void TurtleMoves()
    {
        Shape hero = game.Actors.List[0];

        if (TurtleCarries(game.Actors.RaftNearness, game.Actors.CarrierSlot))
        {
            facing = hero.Facing;
            game.Actors.TestX = hero.AbsX;
            game.Actors.TestY = hero.AbsY;
            game.Hero.Riding = RIDING_TURTLE;
            image = (short)(facing + facing);

            if (hero.State == ActorState.WALKING)
            {
                image += (short)(cycle & 1);
            }
        }
        else
        {
            TurtleSwims();
        }

        terrainHere = TerrainAtPoint(game.Actors.TestX, game.Actors.TestY);

        if (terrainHere == TERRAIN_DEEP_WATER)
        {
            actor.AbsX = game.Actors.TestX;
            actor.AbsY = game.Actors.TestY;
        }

        step = 1;
    }

    // The turtle on its own: keeps going the same way while there is deep water ahead, otherwise
    // tries turning right, then left, then back to straight on the left again.
    private void TurtleSwims()
    {
        int x = actor.AbsX;
        int y = actor.AbsY;

        facing = TurtleHeading(facing, direction => TerrainAtPoint(PosX(x, direction, TURTLE_STEP), PosY(y, direction, TURTLE_STEP)) == TERRAIN_DEEP_WATER);
        game.Actors.TestX = PosX(x, facing, TURTLE_STEP);
        game.Actors.TestY = PosY(y, facing, TURTLE_STEP);
        game.Hero.Riding = 0;
        image = (short)(facing + facing + (cycle & 1));
    }

    // The raft: follows the hero when the hero is very near and the terrain is water.
    private Pc Raft()
    {
        game.Hero.Riding = 0;

        if (game.Actors.CarrierSlot != RIDING_RAFT || game.Actors.RaftNearness != 2)
        {
            return Pc.Statc;
        }

        game.Actors.TestX = game.Actors.List[0].AbsX;
        game.Actors.TestY = game.Actors.List[0].AbsY;
        terrainHere = TerrainAtPoint(game.Actors.TestX, game.Actors.TestY);

        if (terrainHere < TERRAIN_WADE || terrainHere > TERRAIN_DEEP_WATER)
        {
            return Pc.Statc;
        }

        actor.AbsX = game.Actors.TestX;
        actor.AbsY = game.Actors.TestY;
        game.Hero.Riding = RIDING_RAFT;

        return Pc.Statc;
    }

    // People standing about: images from the setfig table, talking, and the witch watching the hero.
    private Pc SetFig()
    {
        terrainHere = (short)(actor.Race & PERSON_MASK);
        image = Tables.Entry(Tables.SetfigTable, terrainHere).ImageBase;
        bool witch = terrainHere == SETFIG_WITCH;

        if (state == ActorState.DYING)
        {
            image += (short)(PERSON_DYING_OFFSET + WitchExtra(witch));
        }
        else if (state == ActorState.DEAD)
        {
            image += (short)(PERSON_DEAD_OFFSET + WitchExtra(witch));
        }
        else if (witch)
        {
            Steering.SteerActor(game, slot, game.Hero.X, game.Hero.Y, 0);
            image = (short)(actor.Facing / 2);
            game.Witch.OnScreen = 1;
        }
        else if (state == ActorState.TALKING)
        {
            image += (short)game.Rng.Rand2();
            actor.Tactic = unchecked((sbyte)(actor.Tactic - 1));

            if (actor.Tactic == 0)
            {
                actor.State = ActorState.STILL;
            }
        }
        else
        {
            return Pc.Statc;
        }

        environ = 0;
        terrainHere = 0;

        return Pc.Tail;
    }
}
