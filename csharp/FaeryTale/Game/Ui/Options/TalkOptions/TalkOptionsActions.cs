using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Ui.Options;

/// TalkOptions: describing the target and carrying out the action.
public static partial class TalkOptions
{
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
}
