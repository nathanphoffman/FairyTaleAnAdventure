using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Combat;

/// Swinging weapons and biting: who is hit, how far the weapon reaches and the whoosh of a near miss.
public static class Melee
{
    /// What decides how a swing at one target turns out.
    internal readonly record struct SwingFacts(bool ByHero, short Dist, short Reach, bool Frozen, short Strength);

    /// How a swing at one target turned out.
    internal enum SwingResult
    {
        Hit,

        /// Near enough to make the whoosh sound, but no hit.
        Whoosh,

        Miss,
    }

    /// A weapon with this bit is a bow (it is not swung).
    internal const short BOW_BIT = 4;

    /// Touch attacks (monsters' weapon codes 8 and up) are as strong as this weapon.
    internal const short TOUCH_WEAPON = 5;

    internal const short BRAVE_PER_REACH = 20;
    internal const short HERO_BASE_REACH = 5;
    internal const short ENEMY_BASE_REACH = 2;
    internal const short MAX_REACH = 14;
    internal const short REACH_LIMIT = 15;

    /// Weapon numbers from here on are natural weapons (bites, claws).
    private const short FIRST_TOUCH_WEAPON = 8;

    /// The tip of the weapon is placed this far (plus a little noise) in front of the actor.
    private const int TIP_NOISE = 3;
    private const short SWING_SOUND = 1;
    private const int SWING_SOUND_PITCH = 150;

    /// How much further than the reach a swing still makes the whoosh sound.
    private const short WHOOSH_MARGIN = 2;

    private const int RAFT = 1;

    /// Everything not walking swings its weapon (or bites) at whatever is within reach of the
    /// tip: HitActor if it lands, a whoosh if it nearly does. Only the hero acts while time is frozen.
    public static void ResolveMelee(Data.Game game, IOutput output)
    {
        for (int index = 0; index < game.Actors.Active; index++)
        {
            if (index > 0 && game.World.FreezeTimer != 0)
            {
                break;
            }

            if (index != RAFT && game.Actors.List[index].State < ActorState.WALKING)
            {
                Swing(game, output, index);
            }
        }
    }

    /// The weapon an actor swings in melee, or null for a bow (archers shoot instead). Anything
    /// past the hand weapons (8 and up: bites, claws) counts as a touch.
    internal static short? MeleeWeapon(short weapon)
    {
        if ((weapon & BOW_BIT) != 0)
        {
            return null;
        }

        return weapon >= FIRST_TOUCH_WEAPON ? TOUCH_WEAPON : weapon;
    }

    /// How close the weapon tip has to get to hit. Reach beyond the maximum drops to the limit.
    internal static short LimitedReach(short reach) => reach > MAX_REACH ? REACH_LIMIT : reach;

    /// The hero's reach grows with bravery.
    internal static short HeroReach(short brave) => LimitedReach((short)(brave / BRAVE_PER_REACH + HERO_BASE_REACH));

    /// An enemy's reach is a fixed base plus a random bonus.
    internal static short EnemyReach(short bonus) => LimitedReach((short)(ENEMY_BASE_REACH + bonus));

    /// The tip of the weapon is dist from the target and the reach is reach. The hero always hits
    /// within reach; an enemy also has to get past the hero's bravery (dodgeRoll is a 0-255 draw,
    /// taken before anything else is checked, only for enemies). Nothing hits while time is
    /// frozen, and touch attacks make no sound.
    internal static SwingResult SwingResultFor(SwingFacts facts, short heroBravery, Func<int> dodgeRoll)
    {
        if ((facts.ByHero || dodgeRoll() > heroBravery) && facts.Dist < facts.Reach && !facts.Frozen)
        {
            return SwingResult.Hit;
        }

        if (facts.Dist < facts.Reach + WHOOSH_MARGIN && facts.Strength != TOUCH_WEAPON)
        {
            return SwingResult.Whoosh;
        }

        return SwingResult.Miss;
    }

    // One actor's swing. The random draws happen in this order: strength, the weapon tip's wobble,
    // then (for enemies) the reach.
    private static void Swing(Data.Game game, IOutput output, int index)
    {
        Shape attacker = game.Actors.List[index];

        if (MeleeWeapon(attacker.Weapon) is not { } baseStrength)
        {
            return;
        }

        short strength = (short)(baseStrength + game.Rng.BitRand(2));
        int facing = attacker.Facing;
        (short tipX, short tipY) = WeaponTip(game, attacker, facing, strength);
        short reach = index == 0 ? HeroReach(game.Hero.Brave) : EnemyReach((short)game.Rng.Rand4());

        for (int target = 0; target < game.Actors.Active; target++)
        {
            if (SwingAt(game, output, index, target, facing, strength, reach, (tipX, tipY)))
            {
                break;
            }
        }
    }

    // Swings at actor target if it is a target (not the raft, the attacker, the dead, or by a
    // carrier). Returns true when the swing hit, which ends it.
    private static bool SwingAt(Data.Game game, IOutput output, int attackerIndex, int target, int facing, short strength, short reach, (short X, short Y) tip)
    {
        Shape attacker = game.Actors.List[attackerIndex];
        Shape other = game.Actors.List[target];

        if (target == RAFT || target == attackerIndex || other.State == ActorState.DEAD || attacker.Kind == ActorKind.CARRIER)
        {
            return false;
        }

        SwingFacts facts = new(attackerIndex == 0, DistanceToPoint(other, tip.X, tip.Y), reach, game.World.FreezeTimer != 0, strength);

        switch (SwingResultFor(facts, game.Hero.Brave, () => game.Rng.Rand256()))
        {
            case SwingResult.Hit:
                Hits.HitActor(game, output, attackerIndex, target, facing, strength);

                return true;

            case SwingResult.Whoosh:
                Music.PlayEffect(game, SWING_SOUND, SWING_SOUND_PITCH + game.Rng.Rand256());

                return false;

            default:
                return false;
        }
    }

    // Where the tip of the weapon is: twice the weapon's strength in front of the attacker, with a
    // little random wobble.
    private static (short X, short Y) WeaponTip(Data.Game game, Shape attacker, int facing, short strength)
    {
        int reach = strength + strength;
        short x = (short)(MapSteps.StepX(attacker.AbsX, facing, reach) + game.Rng.Rand8() - TIP_NOISE);
        short y = (short)(MapSteps.StepY(attacker.AbsY, facing, reach) + game.Rng.Rand8() - TIP_NOISE);

        return (x, y);
    }

    /// Distance between two points as the larger of the x and y differences (they wrap in 16
    /// bits, as in the original).
    internal static short Chebyshev(short dx, short dy) => (short)Math.Max(Math.Abs((int)dx), Math.Abs((int)dy));

    /// How far an actor is from the point (x, y), by Chebyshev.
    internal static short DistanceToPoint(Shape target, short x, short y) =>
        Chebyshev((short)(target.AbsX - x), (short)(target.AbsY - y));
}
