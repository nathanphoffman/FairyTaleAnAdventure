using FaeryTale.Game.Creatures;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Testing;

/// The scripted test commands (teleport, kill, give, ...). They are queued from the test script
/// and applied at the top of the game's main loop, so they never run in the middle of a frame.
public static class TestCommands
{
    /// Death type used when the "kill the hero" command ends the game.
    private const int DEATH_BY_TEST = 5;

    /// First actor slot that can be an enemy.
    private const int FIRST_ENEMY_SLOT = 3;

    /// Runs the queued command, if any. Called at the top of every frame.
    public static void RunPendingCommand(Data.Game game, IOutput output)
    {
        if (CommandQueue.Take() is not { } command)
        {
            return;
        }

        switch (command.Kind)
        {
            case CommandKind.Teleport:
                Places.TransferHero(game, output, (ushort)command.First, (ushort)command.Second, 1);

                break;

            case CommandKind.Give:
                game.Hero.Stuff[command.First] = (byte)command.Second;

                break;

            case CommandKind.SetStat:
                SetStat(game, command.First, (short)command.Second);

                break;

            case CommandKind.SlayEnemies:
                SlayEnemies(game, output);

                break;

            case CommandKind.KillHero:
                game.Hero.Luck = (short)command.First;
                game.Actors.List[0].Vitality = 0;
                Encounters.CheckDeath(game, output, 0, DEATH_BY_TEST);

                break;
        }
    }

    // Sets a stat: 0 hunger, 1 fatigue, anything else wealth.
    private static void SetStat(Data.Game game, int stat, short value)
    {
        if (stat == 0)
        {
            game.Hero.Hunger = value;
        }
        else if (stat == 1)
        {
            game.Hero.Fatigue = value;
        }
        else
        {
            game.Hero.Wealth = value;
        }
    }

    // Kills every enemy that is out, as if each had been defeated.
    private static void SlayEnemies(Data.Game game, IOutput output)
    {
        for (int number = FIRST_ENEMY_SLOT; number < game.Actors.Active; number++)
        {
            if (game.Actors.List[number].Kind == ActorKind.ENEMY)
            {
                game.Actors.List[number].Vitality = 0;
                Encounters.CheckDeath(game, output, number, 0);
            }
        }
    }
}
