using FaeryTale.Game.Creatures;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Ui;

/// Magic: the map and the fear spell.
public static partial class Magic
{
    /// Whether the fear spell kills this actor: a living enemy of one of the weaker races.
    internal static bool IsScaredByFearSpell(Shape actor) =>
        actor.Vitality != 0 && actor.Kind == ActorKind.ENEMY && actor.Race < FEAR_RACE_LIMIT;

    // Draws the overview map with a "+" where the hero is, and shows it.
    private static void ShowMap(Data.Game game, IOutput output)
    {
        output.DrawOverviewMap(game.Map, game.World, game.World.MapX, game.World.MapY);

        // The hero on the overview map: his tile column minus the left edge of the shown sectors,
        // and his position in 16 pixel steps minus the top edge.
        uint positionX = unchecked((uint)((game.Hero.X >> TILE_SHIFT) - ((game.World.SectorX + (long)game.World.OriginX) << TILE_SHIFT) - MARK_X_OFFSET));
        uint positionY = unchecked((uint)((game.Hero.Y >> TILE_SHIFT) - ((game.World.SectorY + (long)game.World.OriginY) << TILE_SHIFT) + MARK_Y_OFFSET));  // same idea for Y: pixels to tiles, minus the sector origin
        bool onMap = positionX > 0 && positionX < MARK_MAX_X && positionY > 0 && positionY < MARK_MAX_Y;

        output.MarkOverviewMap(onMap ? ((int)positionX, (int)positionY) : null);

        game.Ui.ViewStatus = VIEW_MAP;
        MessagePage.ShowStillScreen(game, output);
        TextRoutines.QueuePrint(game.Ui, PRQ_MAP);
    }

    // Kills every weak enemy on the screen (and costs a little bravery for each).
    private static void ScareEnemies(Data.Game game, IOutput output)
    {
        for (int counter = 1; counter < game.Actors.Active; counter++)
        {
            Shape actor = game.Actors.List[counter];

            if (IsScaredByFearSpell(actor))
            {
                actor.Vitality = 0;
                Encounters.CheckDeath(game, output, counter, 0);
                game.Hero.Brave--;
            }
        }

        if (game.Actors.BattleFlag != 0)
        {
            TextRoutines.ShowEvent(game, output, BATTLE_EVENT);
        }
    }
}
