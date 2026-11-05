using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Revive;

/// Bringing the hero back: starting a brother's adventure at the beginning, or reviving the same
/// brother at the last safe spot after the good fairy.
///
/// The three brothers (Julian, Phillip, Kevin) play in turn. When one dies for good the next takes
/// over, with the first one's bones and goods left where they fell, until the third has died.
public static class Revival
{
    // Values of ViewStatus for "fade in" and "game over screen".
    private const sbyte VIEW_FADE_IN = 3;
    private const sbyte VIEW_GAME_OVER = 2;

    // The hero's actor kind and goal (the user controls the hero).
    private const sbyte HERO_KIND = 0;
    private const sbyte USER_GOAL = 0;

    /// Revives the hero. With newBrother set, a brother starts fresh: the previous one's body is
    /// left in the world, the next brother's stats and an empty pack (just a dirk) are loaded, the
    /// story text is shown, and the hero starts at the first safe spot. Without it (the good
    /// fairy), the same brother returns to the last safe spot after a fade.
    ///
    /// Either way the hero gets full health for the bravery, the clock is set to the morning
    /// (8000), and hunger and fatigue are cleared.
    public static void Revive(Data.Game game, IOutput output, short newBrother)
    {
        Brothers.ResetRaftSetfig(game.Actors);

        Shape hero = game.Actors.List[0];

        hero.Kind = HERO_KIND;
        hero.Goal = USER_GOAL; // yes, we can usurp control from the user

        InputHandler.ResetKeyBuffer();
        game.Actors.BattleFlag = 0;
        game.Hero.GoodFairyTimer = 0;
        game.Actors.NextMissile = 0;

        if (newBrother != 0)
        {
            Music.Player.StopScore();
            Brothers.LeaveFallenBrother(game.Objects, game.Hero);
            Brothers.TakeOverNextBrother(game);
            Brothers.ShowBrotherPlacards(game, output);
            Brothers.AnnounceBrother(game, output);
        }
        else
        {
            Fading.FadeDown(game, output);
        }

        SafeSpot.PutHeroAtSafeSpot(game, output);
        SafeSpot.RestoreHero(game);

        if (game.Hero.Brother > Brothers.LAST_BROTHER)
        {
            game.Ui.ViewStatus = VIEW_GAME_OVER;
        }
        else
        {
            game.Ui.ViewStatus = VIEW_FADE_IN;
            Music.SetMood(game, 1);
        }

        game.World.ExtentType = 0;
        game.Actors.FieryDeath = 0;
    }
}
