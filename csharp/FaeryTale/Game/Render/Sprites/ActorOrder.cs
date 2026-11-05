using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;

namespace FaeryTale.Game.Render;

/// The order the actors are drawn in, back to front, who is the nearest person, and putting back
/// the background under last frame's sprites.
public static class ActorOrder
{
    private const short BIRD = 11;

    /// Closest a person can be to count as the nearest person.
    private const short PERSON_RANGE = 50;

    // Dead actors, the hero on a mount and the raft are drawn this much lower in the order.
    private const short DRAW_LOWER = 32;
    private const sbyte WADING = 25;
    private const byte RAFT_ACTOR = 1;

    /// Finds the nearest person, and sorts DrawOrder by y so that actors lower on the screen are
    /// drawn later.
    public static void SortActors(Data.Game game)
    {
        game.Actors.NearestPerson = 0;
        game.Actors.NearestPersonDistance = PERSON_RANGE;

        for (int index = 0; index < game.Actors.Drawn; index++)
        {
            game.Actors.DrawOrder[index] = (byte)index;
        }

        for (int index = 0; index < game.Actors.Drawn; index++)
        {
            Shape actor = game.Actors.List[index];

            if (index != 0 && actor.Kind != ActorKind.OBJECTS && actor.State != ActorState.DEAD && game.Hero.Riding != BIRD)
            {
                short distance = (short)Blocking.ActorDistance(game, index, 0);

                if (distance < game.Actors.NearestPersonDistance)
                {
                    game.Actors.NearestPersonDistance = distance;
                    game.Actors.NearestPerson = (short)index;
                }
            }

            for (int j = 1; j < game.Actors.Drawn; j++)
            {
                BubbleStep(game.Actors, game.Hero, j);
            }
        }
    }

    /// Puts back the background under last frame's sprites, then undoes the witch effect on that page.
    public static void UndoSprites(Data.Game game, Video video)
    {
        video.Screen.DrawTarget = video.Screen.DrawingBitmapId;

        for (int index = video.Screen.DrawingPage.ObCount; index >= 1; index--)
        {
            video.Blitter.SShape = video.Screen.DrawingPage.ShapeQueue[index - 1];
            SpriteBlitter.RestoreBackground(video, video.Blitter.SShape.BackSave);
        }

        video.Screen.GetPort(PortId.Map).Target = video.Screen.DrawingBitmapId;

        if (video.Screen.DrawingPage.WFlag != 0)
        {
            WitchBeam.WitchFx(game.Witch, game.World, game.Hero, video, video.Screen.Drawing);
        }
    }

    // One compare-and-swap of neighbours j - 1 and j in the draw order.
    private static void BubbleStep(Actors actors, Hero hero, int j)
    {
        byte k1 = actors.DrawOrder[j - 1];
        byte k2 = actors.DrawOrder[j];
        Shape a = actors.List[k1];
        Shape b = actors.List[k2];
        short y1 = (short)a.AbsY;
        short y2 = (short)b.AbsY;

        if (a.State == ActorState.DEAD || (k2 == 0 && hero.Riding != 0) || k1 == RAFT_ACTOR)
        {
            y1 = unchecked((short)(y1 - DRAW_LOWER));
        }

        if (b.State == ActorState.DEAD || (k1 == 0 && hero.Riding != 0) || k2 == RAFT_ACTOR)
        {
            y2 = unchecked((short)(y2 - DRAW_LOWER));
        }

        if (a.Environ > WADING)
        {
            y1 = unchecked((short)(y1 + DRAW_LOWER));
        }

        if (b.Environ > WADING)
        {
            y2 = unchecked((short)(y2 + DRAW_LOWER));
        }

        if (y2 < y1)
        {
            actors.DrawOrder[j - 1] = k2;
            actors.DrawOrder[j] = k1;
        }
    }
}
