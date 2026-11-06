using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Remastered;

/// Finding the things lying about (chests, gold, weapons) under the mouse that Take would pick up.
public static class ItemTargets
{
    // Objects are one 16 pixel word wide.
    private const int PICTURE_WIDTH = 16;

    /// The actor number of the pick-up-able object drawn at a world position, if there is one. The
    /// one drawn last (in front) wins when pictures overlap.
    public static int? ItemAt(WorldCapture capture, (int X, int Y) world)
    {
        for (int index = capture.Sprites.Count - 1; index >= 0; index--)
        {
            CapturedSprite sprite = capture.Sprites[index];

            if (sprite.Actor is { } actor && TakeItems.CanTake(actor) && PictureHolds(sprite, world))
            {
                return sprite.Slot;
            }
        }

        return null;
    }

    /// Whether the hero is near enough to take an object.
    public static bool IsCloseEnough(Data.Game game, int actorNumber) =>
        Blocking.ActorDistance(game, actorNumber, 0) < TakeItems.TAKE_RANGE - 1;

    /// Whether the actor number is still an object that can be taken (it may have been taken).
    public static bool StillThere(Data.Game game, int actorNumber) =>
        actorNumber < game.Actors.Drawn && TakeItems.CanTake(game.Actors.List[actorNumber]);

    // Whether a world position is inside an object's picture.
    private static bool PictureHolds(CapturedSprite sprite, (int X, int Y) world) =>
        world.X >= sprite.Left && world.X < sprite.Left + PICTURE_WIDTH
        && world.Y >= sprite.Top && world.Y < sprite.Top + sprite.Rows;
}
