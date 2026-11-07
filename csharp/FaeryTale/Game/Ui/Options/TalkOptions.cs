using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Ui.Options;

/// The Talk and Yell menu entries: who the hero is talking to and what answers, and what each
/// kind of person says. Some also change things (the priest heals, the sorceress gives luck).
public static class TalkOptions
{
    /// Who the hero is talking to.
    internal sealed record Target(sbyte Kind, sbyte State, byte Race, bool CanTalk);

    /// What a Talk or Yell does.
    internal abstract record TalkAction
    {
        internal sealed record Nothing : TalkAction;

        /// Yelling at someone standing right next to you.
        internal sealed record TooCloseToYell : TalkAction;

        /// A person answers (and turns to talk if it can).
        internal sealed record Person(bool StartsTalking) : TalkAction;

        internal sealed record TurtleHasShell : TalkAction;

        internal sealed record TurtleGivesShell : TalkAction;

        /// An enemy says its line (the speech number is its race).
        internal sealed record EnemyShouts(byte Race) : TalkAction;
    }

    /// What decides the lines of the people who only talk.
    internal sealed record Talker(short Kindness, short Fatigue, short DayPeriod, ushort Region, int Goal, bool PrincessFreed);

    internal enum SorceressAction
    {
        /// She is free and the roll was kind to the hero.
        GiveLuck,

        /// She is free but the roll was not high enough.
        Nothing,

        /// First meeting.
        AskForHelp,
    }

    /// What the priest does for the hero.
    internal abstract record PriestAction
    {
        /// Has the writ and the reward (object list slot) is still hidden: it appears.
        internal sealed record WritReward : PriestAction;

        /// Has the writ, reward already given.
        internal sealed record WritAgain : PriestAction;

        /// Not kind enough.
        internal sealed record Scold : PriestAction;

        /// Heals fully, with one of three blessings picked by the time of day.
        internal sealed record Bless(int Speech) : PriestAction;
    }

    /// The tactic that makes a person stand and talk.
    private const sbyte TACTIC_TALK = 15;
    private const int SHELL = 6;

    /// A person's race number is the low 7 bits of the race (bit 7 marks a variant).
    private const byte PERSON_MASK = 0x7f;

    // Speech numbers.
    private const int SPEECH_TOO_CLOSE_TO_YELL = 8;
    private const int SPEECH_TURTLE_HAS_SHELL = 57;
    private const int SPEECH_TURTLE_GIVES_SHELL = 56;

    // FindNearestFigure search ranges for yelling and for the quieter talk options.
    private const short YELL_RANGE = 100;
    internal const short TALK_RANGE = 50;

    /// Anyone closer than this is told off for yelling.
    internal const int TOO_CLOSE_TO_YELL = 35;

    // Talk menu entries and the carrier number of the turtle.
    internal const short YELL = 5;
    private const short CARRIER_TURTLE = 5;

    // The people (race & 0x7f).
    private const byte WIZARD = 0;
    private const byte PRIEST = 1;
    private const byte GUARD_FRONT = 2;
    private const byte GUARD_BACK = 3;
    private const byte PRINCESS = 4;
    private const byte KING = 5;
    private const byte NOBLE = 6;
    private const byte SORCERESS = 7;
    private const byte BARTENDER = 8;
    private const byte WITCH = 9;
    private const byte SPECTRE = 10;
    private const byte GHOST = 11;
    private const byte RANGER = 12;
    private const byte BEGGAR = 13;

    // Speech numbers (the lines in the narration table).
    private const int SPEECH_WIZARD_UNKIND = 35;
    private const int SPEECH_WIZARD_BASE = 27;
    private const int SPEECH_GUARD = 15;
    private const int SPEECH_PRINCESS = 16;
    private const int SPEECH_KING = 17;
    private const int SPEECH_NOBLE = 20;
    private const int SPEECH_BARTENDER_RESTED = 13;
    private const int SPEECH_BARTENDER_NIGHT = 12;
    private const int SPEECH_BARTENDER_EVENING = 14;
    private const int SPEECH_WITCH = 46;
    private const int SPEECH_SPECTRE = 47;
    private const int SPEECH_GHOST = 49;
    private const int SPEECH_RANGER_SWAMP = 22;
    private const int SPEECH_RANGER_BASE = 53;
    private const int SPEECH_BEGGAR = 23;
    private const int SPEECH_SORCERESS_HELP = 45;
    private const int SPEECH_PRIEST_WRIT_REWARD = 39;
    private const int SPEECH_PRIEST_WRIT_AGAIN = 19;
    private const int SPEECH_PRIEST_SCOLD = 40;
    private const int SPEECH_PRIEST_BLESSING_BASE = 36;

    /// Kindness the hero needs for the wizard and priest to be friendly.
    internal const short KIND_ENOUGH = 10;

    // Object list slots whose state the conversations change.
    private const int WRIT = 28;
    private const int PRIEST_WRIT_SLOT = 10;
    private const int SORCERESS_SLOT = 9;
    private const int PRINCESS_FREED_SLOT = 9;
    private const int INSIDE_OBJECTS = 8;

    /// Luck given by the sorceress, and the roll it must beat.
    private const short SORCERESS_LUCK = 5;

    /// Hero vitality when healed by the priest is 15 + brave / 4.
    private const short FULL_VITALITY_BASE = 15;
    internal const ushort PRIEST_BLESSINGS = 3;
    private const short BRAVERY_DIVISOR = 4;
    internal const short NIGHT_PERIOD = 7;
    internal const short TIRED = 5;
    internal const ushort RANGER_SWAMP_REGION = 2;

    /// Yell (louder, reaches further) or say something to the nearest person.
    public static bool TalkOption(Data.Game game, IOutput output, short chosenEntry)
    {
        int distance = Blocking.FindNearestFigure(game, 1, chosenEntry == YELL ? YELL_RANGE : TALK_RANGE);

        if (game.Actors.NearestObject == 0)
        {
            return true;
        }

        Converse(game, output, game.Actors.NearestObject, chosenEntry, distance);

        return true;
    }

    /// Yell, say or ask something to one chosen actor instead of the nearest (Remastered: the person
    /// that was clicked on).
    public static void TalkToActor(Data.Game game, IOutput output, int actorNumber, short chosenEntry)
    {
        game.Actors.NearestObject = (short)actorNumber;
        Converse(game, output, actorNumber, chosenEntry, Blocking.ActorDistance(game, actorNumber, 0));
    }

    /// What a Talk or Yell does to the figure nearby. The dead say nothing; people answer (but not
    /// a yell from right next to them); the turtle gives the shell once; an enemy shouts its line.
    internal static TalkAction TalkActionFor(Target target, bool yelling, int distance, short activeCarrier, bool hasShell)
    {
        if (target.State == ActorState.DEAD)
        {
            return new TalkAction.Nothing();
        }

        if (target.Kind == ActorKind.SETFIG)
        {
            return yelling && (uint)distance < TOO_CLOSE_TO_YELL ? new TalkAction.TooCloseToYell() : new TalkAction.Person(target.CanTalk);
        }

        if (target.Kind == ActorKind.CARRIER && activeCarrier == CARRIER_TURTLE)
        {
            return hasShell ? new TalkAction.TurtleHasShell() : new TalkAction.TurtleGivesShell();
        }

        return target.Kind == ActorKind.ENEMY ? new TalkAction.EnemyShouts(target.Race) : new TalkAction.Nothing();
    }

    /// The speech number a person says, if they say anything. The priest and the sorceress are not
    /// here because they also change things.
    internal static int? PlainSpeech(byte person, Talker t) => person switch
    {
        WIZARD => t.Kindness < KIND_ENOUGH ? SPEECH_WIZARD_UNKIND : SPEECH_WIZARD_BASE + t.Goal,
        GUARD_FRONT or GUARD_BACK => SPEECH_GUARD,
        PRINCESS when t.PrincessFreed => SPEECH_PRINCESS,
        KING when t.PrincessFreed => SPEECH_KING,
        NOBLE => SPEECH_NOBLE,
        BARTENDER => t.Fatigue < TIRED ? SPEECH_BARTENDER_RESTED : t.DayPeriod > NIGHT_PERIOD ? SPEECH_BARTENDER_NIGHT : SPEECH_BARTENDER_EVENING,
        WITCH => SPEECH_WITCH,
        SPECTRE => SPEECH_SPECTRE,
        GHOST => SPEECH_GHOST,
        RANGER => t.Region == RANGER_SWAMP_REGION ? SPEECH_RANGER_SWAMP : SPEECH_RANGER_BASE + t.Goal,
        BEGGAR => SPEECH_BEGGAR,
        _ => null,
    };

    /// roll is a 0-63 draw, taken only once she is free: luck is given when the hero's luck is below it.
    internal static SorceressAction SorceressActionFor(bool freed, short luck, Func<short> roll)
    {
        if (!freed)
        {
            return SorceressAction.AskForHelp;
        }

        return luck < roll() ? SorceressAction.GiveLuck : SorceressAction.Nothing;
    }

    /// What the priest does: give the writ's reward once, say so again, scold an unkind hero, or
    /// bless and heal one who is kind enough. The blessing is picked from the clock.
    internal static PriestAction PriestActionFor(bool hasWrit, sbyte rewardStat, short kindness, ushort clock)
    {
        if (hasWrit)
        {
            return rewardStat == 0 ? new PriestAction.WritReward() : new PriestAction.WritAgain();
        }

        if (kindness < KIND_ENOUGH)
        {
            return new PriestAction.Scold();
        }

        return new PriestAction.Bless(SPEECH_PRIEST_BLESSING_BASE + clock % PRIEST_BLESSINGS);
    }

    // Works out and does what a Talk or Yell to an actor at a distance means.
    private static void Converse(Data.Game game, IOutput output, int actorNumber, short chosenEntry, int distance)
    {
        Shape actor = game.Actors.List[actorNumber];
        Target target = DescribeTarget(actor);
        TalkAction action = TalkActionFor(target, chosenEntry == YELL, distance, game.Actors.ActiveCarrier, game.Hero.Stuff[SHELL] != 0);

        CarryOut(game, output, action, actor);
    }

    // What the Talk code needs to know about the actor the hero is talking to.
    private static Target DescribeTarget(Shape actor)
    {
        int person = actor.Race & PERSON_MASK;

        return new Target(actor.Kind, actor.State, actor.Race, Tables.Entry(Tables.SetfigTable, person).CanTalk != 0);
    }

    // Does what TalkActionFor decided: speaks, and turns the person to talk or hands over the shell.
    private static void CarryOut(Data.Game game, IOutput output, TalkAction action, Shape actor)
    {
        switch (action)
        {
            case TalkAction.TooCloseToYell:
                TextRoutines.Speak(game, output, SPEECH_TOO_CLOSE_TO_YELL);

                break;

            case TalkAction.Person person:
                if (person.StartsTalking)
                {
                    actor.State = ActorState.TALKING;
                    actor.Tactic = TACTIC_TALK;
                }

                PersonSpeaks(game, output, (byte)(actor.Race & PERSON_MASK), actor);

                break;

            case TalkAction.TurtleHasShell:
                TextRoutines.Speak(game, output, SPEECH_TURTLE_HAS_SHELL);

                break;

            case TalkAction.TurtleGivesShell:
                game.Hero.Stuff[SHELL] = 1;
                TextRoutines.Speak(game, output, SPEECH_TURTLE_GIVES_SHELL);

                break;

            case TalkAction.EnemyShouts shout:
                TextRoutines.Speak(game, output, shout.Race);

                break;
        }
    }

    // What each kind of person says. Some of them also change the world.
    private static void PersonSpeaks(Data.Game game, IOutput output, byte person, Shape actor)
    {
        if (person == PRIEST)
        {
            PriestSpeaks(game, output);
        }
        else if (person == SORCERESS)
        {
            SorceressSpeaks(game, output);
        }
        else
        {
            Talker facts = new(
                game.Hero.Kindness,
                game.Hero.Fatigue,
                game.World.DayPeriod,
                game.World.Region,
                actor.Goal,
                game.Objects.Regions[INSIDE_OBJECTS][PRINCESS_FREED_SLOT].ObStat != 0);

            if (PlainSpeech(person, facts) is { } line)
            {
                TextRoutines.Speak(game, output, line);
            }
        }
    }

    // The sorceress gives luck once she has been freed; before that she asks for help.
    private static void SorceressSpeaks(Data.Game game, IOutput output)
    {
        bool freed = game.Objects.Global[SORCERESS_SLOT].ObStat != 0;

        switch (SorceressActionFor(freed, game.Hero.Luck, () => (short)game.Rng.Rand64()))
        {
            case SorceressAction.GiveLuck:
                game.Hero.Luck += SORCERESS_LUCK;

                break;

            case SorceressAction.AskForHelp:
                TextRoutines.Speak(game, output, SPEECH_SORCERESS_HELP);
                game.Objects.Global[SORCERESS_SLOT].ObStat = 1;

                break;
        }

        TextRoutines.QueuePrint(game.Ui, MenuChoices.PRQ_STATS_LINE);
    }

    // The priest gives the writ's reward, scolds the unkind, or heals.
    private static void PriestSpeaks(Data.Game game, IOutput output)
    {
        bool hasWrit = game.Hero.Stuff[WRIT] != 0;

        switch (PriestActionFor(hasWrit, game.Objects.Global[PRIEST_WRIT_SLOT].ObStat, game.Hero.Kindness, game.World.DayClock))
        {
            case PriestAction.WritReward:
                TextRoutines.Speak(game, output, SPEECH_PRIEST_WRIT_REWARD);
                game.Objects.Global[PRIEST_WRIT_SLOT].ObStat = 1;

                break;

            case PriestAction.WritAgain:
                TextRoutines.Speak(game, output, SPEECH_PRIEST_WRIT_AGAIN);

                break;

            case PriestAction.Scold:
                TextRoutines.Speak(game, output, SPEECH_PRIEST_SCOLD);

                break;

            case PriestAction.Bless bless:
                TextRoutines.Speak(game, output, bless.Speech);
                game.Actors.List[0].Vitality = (short)(FULL_VITALITY_BASE + game.Hero.Brave / BRAVERY_DIVISOR);
                TextRoutines.QueuePrint(game.Ui, MenuChoices.PRQ_VITALITY_LINE);

                break;
        }
    }
}
