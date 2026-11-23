using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Ui.Options;

/// TalkOptions: what each kind of person says.
public static partial class TalkOptions
{
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
