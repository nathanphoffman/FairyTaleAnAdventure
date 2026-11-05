using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures;

/// What the people near the hero say when the hero comes close.
public static class NearbySpeech
{
    private const short BEGGAR = 0x8d;
    private const short WITCH = 0x89;
    private const short PRINCESS = 0x84;
    private const short NECROMANCER = 9;
    private const short DKNIGHT = 7;
    private const int PRINCESS_FREED_SLOT = 9;
    private const int INSIDE_OBJECTS = 8;

    // Speech numbers of what each person says on coming near.
    private const int SPEECH_BEGGAR = 23;
    private const int SPEECH_WITCH = 46;
    private const int SPEECH_PRINCESS = 16;
    private const int SPEECH_NECROMANCER = 43;
    private const int SPEECH_DKNIGHT = 41;

    /// The nearest person says something when they come near (once per person).
    public static void SpeakNearby(Data.Game game, IOutput output)
    {
        short race = game.Actors.List[game.Actors.NearestPerson].Race;

        if (game.Actors.NearestPerson == 0 || race == game.Actors.LastPerson)
        {
            return;
        }

        switch (race)
        {
            case BEGGAR:
                TextRoutines.Speak(game, output, SPEECH_BEGGAR);
                break;

            case WITCH:
                TextRoutines.Speak(game, output, SPEECH_WITCH);
                break;

            case PRINCESS when game.Objects.Regions[INSIDE_OBJECTS][PRINCESS_FREED_SLOT].ObStat != 0:
                TextRoutines.Speak(game, output, SPEECH_PRINCESS);
                break;

            case NECROMANCER:
                TextRoutines.Speak(game, output, SPEECH_NECROMANCER);
                break;

            case DKNIGHT:
                TextRoutines.Speak(game, output, SPEECH_DKNIGHT);
                break;
        }

        game.Actors.LastPerson = race;
    }
}
