using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;

namespace FaeryTale.Game.Creatures.Ai;

/// Choosing a tactic for an enemy that wants to fight, and what a hostile enemy does about it.
public static class EnemyTactics
{
    /// What PickTactic needs to know about the enemy.
    internal readonly record struct TacticFacts(byte Race, sbyte Weapon, short Vitality, bool EggsToGuard);

    /// What a hostile enemy does this frame.
    internal enum Hostile
    {
        /// Close enough with a hand weapon: face the hero and fight.
        Strike,

        /// The dark knight stands guard, facing one way.
        KnightGuard,

        /// Otherwise follow the tactic.
        Tactic,
    }

    // Archers back off when closer than this, shoot inside the second pair, else close in.
    internal const short ARCHER_TOO_CLOSE_X = 40;
    internal const short ARCHER_TOO_CLOSE_Y = 30;
    internal const short ARCHER_IN_RANGE = 70;

    // Distance at which a fighter closes to strike, minus its goal number (the dark knight: its own).
    internal const short STRIKE_BASE = 14;
    internal const byte DKNIGHT = 7;
    internal const short DKNIGHT_STRIKE = 16;
    internal const byte SPIDER_EGGS_RACE = 4;
    internal const short WEAK = 6;
    private const sbyte DKNIGHT_FACING = 5;
    private const int EGG_SEEK = 10;

    /// A hostile enemy picks a new tactic. May also change its mode to wandering if it has no weapon.
    public static (short Mode, short Tactic) ChooseTactic(Data.Game game, Shape actor, short mode, short xd, short yd)
    {
        TacticFacts facts = new(actor.Race, actor.Weapon, actor.Vitality, game.Actors.TurtleEggs != 0);

        return PickTactic(facts, mode, xd, yd, () => game.Rng.Rand2() != 0);
    }

    /// A hostile enemy strikes if close enough, otherwise acts on its tactic.
    public static void HostileAction(Data.Game game, int index, short mode, short tactic, (short X, short Y) distance)
    {
        Shape actor = game.Actors.List[index];

        switch (HostileChoice(actor.Race, actor.Weapon, actor.Vitality, mode, distance.X, distance.Y))
        {
            case Hostile.Strike:
                Steering.SteerActor(game, index, game.Hero.X, game.Hero.Y, 0);

                if (actor.State >= ActorState.WALKING)
                {
                    actor.State = ActorState.FIGHTING;
                }

                break;

            case Hostile.KnightGuard:
                actor.State = ActorState.STILL;
                actor.Facing = DKNIGHT_FACING;

                break;

            default:
                Tactics.RunTactic(game, index, tactic);

                break;
        }
    }

    /// The tactic for an enemy in mode, xd and yd pixels (absolute) from the hero: seek the eggs,
    /// wander if unarmed (which also changes the mode), evade when weak (coin decides, and is only
    /// asked then), keep an archer at a distance and shoot, or just pursue.
    internal static (short Mode, short Tactic) PickTactic(TacticFacts enemy, short mode, short xd, short yd, Func<bool> coin)
    {
        if (enemy.Race == SPIDER_EGGS_RACE && enemy.EggsToGuard)
        {
            return (mode, EGG_SEEK);
        }

        if (enemy.Weapon < 1)
        {
            return (ActorGoal.CONFUSED, ActorTactic.RANDOM);
        }

        if (enemy.Vitality < WEAK && coin())
        {
            return (mode, ActorTactic.EVADE);
        }

        if (mode < ActorGoal.ARCHER1)
        {
            return (mode, ActorTactic.PURSUE);
        }

        if (xd < ARCHER_TOO_CLOSE_X && yd < ARCHER_TOO_CLOSE_Y)
        {
            return (mode, ActorTactic.BACKUP);
        }

        return (mode, (short)(xd < ARCHER_IN_RANGE && yd < ARCHER_IN_RANGE ? ActorTactic.SHOOT : ActorTactic.PURSUE));
    }

    /// What a hostile enemy does this frame: strike if close enough with a hand weapon (the dark
    /// knight has its own, longer reach), the knight stands guard while alive, and anything else
    /// follows its tactic.
    internal static Hostile HostileChoice(byte race, sbyte weapon, short vitality, short mode, short xd, short yd)
    {
        short threshold = race == DKNIGHT ? DKNIGHT_STRIKE : (short)(STRIKE_BASE - mode);

        if ((weapon & EnemyGoals.BOW_BIT) == 0 && xd < threshold && yd < threshold)
        {
            return Hostile.Strike;
        }

        return race == DKNIGHT && vitality != 0 ? Hostile.KnightGuard : Hostile.Tactic;
    }
}
