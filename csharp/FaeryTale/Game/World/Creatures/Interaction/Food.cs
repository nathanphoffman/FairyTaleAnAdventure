using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Interaction;

/// Eating.
public static class Food
{
    private const int HUNGER_FULL_MESSAGE = 13;

    /// Eats: lowers hunger by amount. If that would go below zero the hero is simply full (message
    /// 13); otherwise "Yum!".
    public static void EatFood(Data.Game game, IOutput output, int amount)
    {
        game.Hero.Hunger = (short)(game.Hero.Hunger - amount);

        if (game.Hero.Hunger < 0)
        {
            game.Hero.Hunger = 0;
            TextRoutines.ShowEvent(game, output, HUNGER_FULL_MESSAGE);
        }
        else
        {
            output.Print("Yum!");
        }
    }
}
