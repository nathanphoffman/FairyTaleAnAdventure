using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Interaction;

/// Hitting an actor: who can be hurt, the sound, the knock-back, and what is said after a fight.
public static class Hits
{
    /// What to say after a fight.
    internal abstract record BattleReport
    {
        /// The hero is dead, or nothing worth saying.
        internal sealed record Silent : BattleReport;

        internal sealed record Bravely : BattleReport;

        internal sealed record Foes(int Dead, int Fled) : BattleReport;
    }

    /// Which sound a hit makes.
    internal enum HitSound
    {
        Arrow,
        Rock,
        HeroHurt,
        FoeHurt,
    }

    // Races with special rules when hit.
    private const byte RACE_SPIRIT = 9;
    private const byte RACE_GHOST_KEEPER = 0x89;
    private static readonly byte[] RACE_UNHITTABLE = [0x8a, 0x8b];

    // The weapon level needed to hurt spirits, and the item that also does.
    internal const sbyte MIN_HURT_WEAPON = 4;
    private const int SPIRIT_HURT_ITEM = 7;
    private const int MESSAGE_NEEDS_WEAPON = 58;

    /// How far hit actors are pushed back.
    private const short KNOCKBACK = 2;
    private const int WORLD_DEAD_CHECK = 5;

    // Sound effects and their speeds.
    private const short EFFECT_ARROW_HIT = 2;
    private const short EFFECT_ROCK_HIT = 5;
    private const short EFFECT_HERO_HURT = 0;
    private const short EFFECT_FOE_HURT = 3;

    // Base pitch of each hit sound, and the random spread added to some of them.
    private const int PITCH_ARROW = 500;
    private const int PITCH_ROCK = 3200;
    private const int PITCH_HERO_HURT = 800;
    private const int PITCH_FOE_HURT = 400;
    private const int PITCH_SPREAD = 511;

    /// Slots from here on in the actor list are monsters.
    private const int FIRST_MONSTER_SLOT = 3;

    /// Message count shown after a fight only while the area is this peaceful (extent type below).
    internal const ushort PEACEFUL_EXTENT = 50;
    internal const short BRAVE_LOW_VITALITY = 5;

    /// A hit from actor index on actor j: wt damage, then the target is pushed back in direction
    /// fc. index is -1 or -2 for hits by missiles (which have different sounds).
    ///
    /// Special cases, as in the original: creatures of race 9 and the witch (race 0x89, a person
    /// of kind 9) cannot be hurt by weak weapons (weapon below 4); the witch can be hurt anyway if
    /// the hero holds the item in inventory slot 7 (the sun stone). Races 0x8a and 0x8b (the
    /// spectre and the ghost brothers) are never hurt. Dragons and people are not pushed back.
    public static void HitActor(Data.Game game, IOutput output, int index, int j, int fc, short wt)
    {
        Shape target = game.Actors.List[j];
        sbyte heroWeapon = game.Actors.List[0].Weapon;

        if (TooWeakToHurt(target.Race, heroWeapon, game.Hero.Stuff[SPIRIT_HURT_ITEM] != 0))
        {
            TextRoutines.Speak(game, output, MESSAGE_NEEDS_WEAPON);

            return;
        }

        if (RACE_UNHITTABLE.Contains(target.Race))
        {
            return;
        }

        // Remastered and Reimagined scale the damage by the difficulty (Original is left as it was).
        wt = Difficulty.ScaleDamage(game, index, j, wt);
        target.Vitality = VitalityAfterHit(target.Vitality, wt);
        PlayHitSound(game, HitSoundFor(index, j));

        // Push back the foe.
        if (CanBePushed(target.Kind)
            && Blocking.StepFigure(game, (short)j, (short)fc, KNOCKBACK) != 0
            && index >= 0)
        {
            Blocking.StepFigure(game, (short)index, (short)fc, KNOCKBACK);
        }

        Encounters.CheckDeath(game, output, j, WORLD_DEAD_CHECK);
    }

    /// After a fight: reports how many foes were defeated or fled, or "Bravely done!" if the hero
    /// is nearly dead. Then, if turtle eggs are around, the turtle story event is checked.
    public static void ReportBattleResult(Data.Game game, IOutput output)
    {
        (int dead, int fled) = CountCasualties(game.Actors);

        switch (BattleReportFor(game.Actors.List[0].Vitality, dead, fled, game.World.ExtentType))
        {
            case BattleReport.Bravely:
                output.Print("Bravely done!");

                break;

            case BattleReport.Foes foes:
                PrintFoes(output, foes);

                break;
        }

        // Check for presence of turtle eggs.
        if (game.Actors.TurtleEggs != 0)
        {
            Extents.SummonTurtle(game, output);
        }
    }

    /// Which sound a hit makes, by who attacked (-1 an arrow, -2 a fireball, otherwise an actor
    /// slot) and who was hit (slot 0 is the hero).
    internal static HitSound HitSoundFor(int attacker, int target) => (attacker, target) switch
    {
        (-1, _) => HitSound.Arrow,
        (-2, _) => HitSound.Rock,
        (_, 0) => HitSound.HeroHurt,
        _ => HitSound.FoeHurt,
    };

    /// Dragons and people are not pushed back by a hit.
    internal static bool CanBePushed(sbyte kind) => kind != ActorKind.DRAGON && kind != ActorKind.SETFIG;

    /// What is said after a fight; dead and fled count the enemies that fell and that ran away.
    internal static BattleReport BattleReportFor(short heroVitality, int dead, int fled, ushort extentType)
    {
        if (heroVitality < 1)
        {
            return new BattleReport.Silent();
        }

        if (heroVitality < BRAVE_LOW_VITALITY && dead != 0)
        {
            return new BattleReport.Bravely();
        }

        return extentType < PEACEFUL_EXTENT ? new BattleReport.Foes(dead, fled) : new BattleReport.Silent();
    }

    /// Spirits can only be hurt by a real weapon (MIN_HURT_WEAPON or better); the ghost keeper
    /// too, unless the hero carries the item that makes him vulnerable.
    internal static bool TooWeakToHurt(byte race, sbyte heroWeapon, bool hasSpiritItem) =>
        heroWeapon < MIN_HURT_WEAPON && (race == RACE_SPIRIT || (race == RACE_GHOST_KEEPER && !hasSpiritItem));

    /// A hit of strength wt takes that much vitality, down to a minimum of 0.
    internal static short VitalityAfterHit(short vitality, short wt) => (short)Math.Max(vitality - wt, 0);

    private static void PlayHitSound(Data.Game game, HitSound sound)
    {
        switch (sound)
        {
            case HitSound.Arrow:
                Music.PlayEffect(game, EFFECT_ARROW_HIT, PITCH_ARROW + game.Rng.Rand64());

                break;

            case HitSound.Rock:
                Music.PlayEffect(game, EFFECT_ROCK_HIT, PITCH_ROCK + game.Rng.BitRand(PITCH_SPREAD));

                break;

            case HitSound.HeroHurt:
                Music.PlayEffect(game, EFFECT_HERO_HURT, PITCH_HERO_HURT + game.Rng.BitRand(PITCH_SPREAD));

                break;

            default:
                Music.PlayEffect(game, EFFECT_FOE_HURT, PITCH_FOE_HURT + game.Rng.Rand256());

                break;
        }
    }

    private static void PrintFoes(IOutput output, BattleReport.Foes foes)
    {
        if (foes.Dead != 0)
        {
            output.Print("");
            output.PrintNumber(foes.Dead, 1);
            output.PrintCont("foes were defeated in battle.");
        }

        if (foes.Fled != 0)
        {
            output.Print("");
            output.PrintNumber(foes.Fled, 1);
            output.PrintCont("foes fled in retreat.");
        }
    }

    // How many of the enemies are dead and how many have fled.
    private static (int Dead, int Fled) CountCasualties(Actors actors)
    {
        int dead = 0;
        int fled = 0;

        for (int index = FIRST_MONSTER_SLOT; index < actors.Active; index++)
        {
            Shape actor = actors.List[index];

            if (actor.Kind != ActorKind.ENEMY)
            {
                continue;
            }

            if (actor.State == ActorState.DEAD)
            {
                dead++;
            }
            else if (actor.Goal == ActorGoal.FLEE)
            {
                fled++;
            }
        }

        return (dead, fled);
    }
}
