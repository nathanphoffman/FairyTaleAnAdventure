using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Ai;

/// What each enemy wants to do each frame: picks a goal and a tactic from how near the hero is
/// and what the enemy carries, and starts or ends the battle music.
public static class EnemyGoals
{
    /// What decides an actor's mode, from plain values.
    internal readonly record struct Situation(sbyte HeroState, short LeaderSlot, short Vitality, ushort ExtentType, byte Race, byte ZoneRace);

    /// Weapon bit for a bow.
    internal const sbyte BOW_BIT = 4;

    // A new tactic is rolled one frame in 16.
    private const int NEW_TACTIC_ODDS = 16;

    // After being frustrated, archers pick one of tactics 2 to 5 and the others 3 or 4.
    private const int ARCHER_FRUSTRATION_TACTIC = 2;
    private const int MELEE_FRUSTRATION_TACTIC = 3;

    /// The first actors are the hero and the raft.
    private const int FIRST_AI_ACTOR = 2;

    /// The good fairy is on screen below this countdown; nobody acts then.
    private const byte FAIRY_ARRIVES = 120;

    /// An actor within this many pixels (in x and y) of the hero counts as on screen.
    internal const short SCREEN_RANGE = 300;

    /// Special encounters (extent type above this) only fight if they are the encounter's race.
    internal const ushort SPECIAL_XTYPE = 59;

    // Carriers (the turtle) re-aim every 16 ticks, steering mode 5.
    private const ushort CARRIER_AIM_MASK = 15;
    private const int CARRIER_AIM_MODE = 5;
    internal const short NEARLY_DEAD = 2;

    /// Decides each enemy's goal and tactic, and starts or ends the battle mood.
    public static void UpdateEnemyGoals(Data.Game game, IOutput output)
    {
        game.Actors.OnScreen = 0;
        game.Actors.Leader = 0;
        sbyte previousBattle = game.Actors.BattleFlag;
        game.Actors.BattleFlag = 0;

        for (int index = FIRST_AI_ACTOR; index < game.Actors.Active; index++)
        {
            if (game.Hero.GoodFairyTimer != 0 && game.Hero.GoodFairyTimer < FAIRY_ARRIVES)
            {
                break;
            }

            Think(game, index, previousBattle);
        }

        if (previousBattle == 0 && game.Actors.BattleFlag != 0)
        {
            Music.SetMood(game, 1);
        }

        if (game.Actors.BattleFlag == 0 && previousBattle != 0)
        {
            TextRoutines.QueuePrint(game.Ui, PrintRequest.STATS);
            TextRoutines.QueuePrint(game.Ui, PrintRequest.VITALITY);
            Hits.ReportBattleResult(game, output);
        }
    }

    /// The mode after the hero's state and the actor's own health are taken into account:
    /// everyone flees (or follows the leader) when the hero is dead or falling, the nearly dead
    /// flee, and so do actors of a special encounter that are not the zone's own race in the 60+ zones.
    internal static short AdjustedMode(short mode, Situation situation)
    {
        if (situation.HeroState == ActorState.DEAD || situation.HeroState == ActorState.FALL)
        {
            return situation.LeaderSlot == 0 ? ActorGoal.FLEE : ActorGoal.FOLLOWER;
        }

        if (situation.Vitality < NEARLY_DEAD || (situation.ExtentType > SPECIAL_XTYPE && situation.Race != situation.ZoneRace))
        {
            return ActorGoal.FLEE;
        }

        return mode;
    }

    /// Whether an actor xd, yd (absolute) pixels from the hero counts as on screen.
    internal static bool OnScreen(short xd, short yd) => xd < SCREEN_RANGE && yd < SCREEN_RANGE;

    // One actor's decision. Returns early (skipping the goal update) for carriers, people and
    // anyone out of hit points who is on screen.
    private static void Think(Data.Game game, int index, sbyte previousBattle)
    {
        Shape actor = game.Actors.List[index];

        if (actor.Kind == ActorKind.CARRIER)
        {
            AimCarrier(game, index);

            return;
        }

        if (actor.Kind == ActorKind.SETFIG)
        {
            return;
        }

        (short xd, short yd) = DistanceFromHero(game.Hero, actor);

        if (!NoticeActor(game.Actors, actor, xd, yd, previousBattle))
        {
            return;
        }

        short tactic = actor.Tactic;
        bool newTacticRoll = game.Rng.BitRand(NEW_TACTIC_ODDS - 1) == 0;
        short mode = AdjustModeForSituation(game, actor, actor.Goal);

        mode = ActOnGoal(game, index, mode, tactic, (xd, yd), newTacticRoll);
        actor.Goal = (sbyte)mode;

        if (game.Actors.Leader == 0)
        {
            game.Actors.Leader = (short)index;
        }
    }

    // A carrier (raft, turtle, swan) keeps steering toward the hero every few frames.
    private static void AimCarrier(Data.Game game, int index)
    {
        if ((game.World.DayClock & CARRIER_AIM_MASK) == 0)
        {
            Steering.SteerActor(game, index, game.Hero.X, game.Hero.Y, CARRIER_AIM_MODE);
        }
    }

    // How far the actor is from the hero along each axis, never negative (wrapping like 16 bits).
    private static (short X, short Y) DistanceFromHero(Hero hero, Shape actor)
    {
        short xd = (short)(actor.AbsX - hero.X);
        short yd = (short)(actor.AbsY - hero.Y);

        return ((short)Math.Abs((int)xd), (short)Math.Abs((int)yd));
    }

    // Notes that an actor is on screen, and that a battle is on if it is awake and in view.
    // Returns false for an actor that is on screen but out of hit points (it does nothing more).
    private static bool NoticeActor(Actors actors, Shape actor, short xd, short yd, sbyte previousBattle)
    {
        if (!OnScreen(xd, yd))
        {
            return true;
        }

        actors.OnScreen = 1;

        if (actor.Vitality < 1)
        {
            return false;
        }

        if (actor.Visible != 0 || previousBattle != 0)
        {
            actors.BattleFlag = 1;
        }

        return true;
    }

    // A stuck actor picks a new tactic: archers one of the ranged ones, the others a melee one.
    private static void TrySomethingElse(Data.Game game, int index)
    {
        Shape actor = game.Actors.List[index];

        if ((actor.Weapon & BOW_BIT) != 0)
        {
            Tactics.RunTactic(game, index, game.Rng.Rand4() + ARCHER_FRUSTRATION_TACTIC);
        }
        else
        {
            Tactics.RunTactic(game, index, game.Rng.Rand2() + MELEE_FRUSTRATION_TACTIC);
        }
    }

    // Carries out the actor's goal for this frame: a stuck actor tries something else, a hostile
    // one picks a tactic now and then and acts on it, the others flee, follow, stand or wait.
    // Returns the goal for next frame, which a hostile actor may change.
    private static short ActOnGoal(Data.Game game, int index, short mode, short tactic, (short X, short Y) distance, bool newTacticRoll)
    {
        Shape actor = game.Actors.List[index];

        if (tactic == ActorTactic.FRUSTRATED || tactic == ActorTactic.SHOOT_FRUSTRATED)
        {
            TrySomethingElse(game, index);
        }
        else if (actor.State == ActorState.SHOOT1)
        {
            actor.State = ActorState.SHOOT3;
        }
        else if (mode <= ActorGoal.ARCHER2)
        {
            if ((mode & 2) == 0)
            {
                newTacticRoll = game.Rng.Rand4() == 0;
            }

            if (newTacticRoll)
            {
                (mode, tactic) = EnemyTactics.ChooseTactic(game, actor, mode, distance.X, distance.Y);
            }

            EnemyTactics.HostileAction(game, index, mode, tactic, distance);
        }
        else if (mode == ActorGoal.FLEE)
        {
            Tactics.RunTactic(game, index, ActorTactic.BACKUP);
        }
        else if (mode == ActorGoal.FOLLOWER)
        {
            Tactics.RunTactic(game, index, ActorTactic.FOLLOW);
        }
        else if (mode == ActorGoal.STAND)
        {
            Steering.SteerActor(game, index, game.Hero.X, game.Hero.Y, 0);
            actor.State = ActorState.STILL;
        }
        else if (mode == ActorGoal.WAIT)
        {
            actor.State = ActorState.STILL;
        }

        return mode;
    }

    // Overrides an actor's mode when the situation demands: flee or follow when the hero is down,
    // flee when nearly dead.
    private static short AdjustModeForSituation(Data.Game game, Shape actor, short mode) =>
        AdjustedMode(
            mode,
            new Situation(
                game.Actors.List[0].State,
                game.Actors.Leader,
                actor.Vitality,
                game.World.ExtentType,
                actor.Race,
                game.Extents[game.World.Extent].V3));
}
