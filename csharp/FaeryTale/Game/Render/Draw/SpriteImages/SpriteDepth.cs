using FaeryTale.Game.Data;

namespace FaeryTale.Game.Render;

/// SpriteImages: sinking into water and the pairs of images.
internal static partial class SpriteImages
{
    /// Sinks the sprite into water, shortens it when the hero rides the swan, and so on. Returns
    /// false if nothing is left to draw.
    internal static bool AdjustForDepth(Hero hero, Shape actor, int slot, short cycle, bool passmode, Sprite sprite)
    {
        short depth = actor.Environ;

        if (!passmode && sprite.Atype != OBJECTS)
        {
            return SinkBody(hero, actor, slot, cycle, depth, sprite);
        }

        return SinkItem(depth, sprite);
    }

    // Snakes use their own pictures; other enemies have two pictures per figure, picked by the
    // race (the hero, actor 0, has just the one).
    private static short PickOfPair(Shape actor, int slot, short inum)
    {
        if (actor.Race == RACE_SNAKE)
        {
            return actor.State < ActorState.DYING ? (short)(inum + SNAKE_FIRST_IMAGE) : inum;
        }

        if (slot > 0)
        {
            return (actor.Race & 1) != 0 ? (short)(inum | IMAGE_PAIR_BIT) : (short)(inum & ~IMAGE_PAIR_BIT);
        }

        return inum;
    }

    // A body standing in water: the raft and the swan lift it, shallow water hides the legs, and
    // deep water shows only bubbles. Returns false if there is nothing to draw (a dead body under water).
    private static bool SinkBody(Hero hero, Shape actor, int slot, short cycle, short depth, Sprite sprite)
    {
        if (slot == 0 && hero.Riding == RIDING_SWAN)
        {
            sprite.YStop -= SWAN_LIFT;
        }
        else if (depth == SHALLOW_DEPTH)
        {
            sprite.YStop -= SHALLOW_HIDDEN_LINES;
        }
        else if (depth > UNDER_SURFACE_DEPTH)
        {
            if (actor.State == ActorState.DEAD)
            {
                return false;
            }

            // Under the surface: only the bubbles show.
            sprite.YStart += BUBBLES_DROP;
            sprite.YStop = (short)(sprite.YStart + BUBBLES_HEIGHT);
            sprite.Atype = OBJECTS;
            sprite.Inum = (short)(OBJ_SUNK + ((cycle + slot) & 1));
        }
        else if (depth > SHALLOW_DEPTH)
        {
            sprite.YStart += depth;
        }

        return true;
    }

    // A weapon or object in water goes down with the water; under the surface it is not drawn at all.
    private static bool SinkItem(short depth, Sprite sprite)
    {
        if (depth > UNDER_SURFACE_DEPTH)
        {
            return false; // no weapon
        }

        if (depth > SHALLOW_DEPTH)
        {
            sprite.YStart += depth;
            sprite.YStop += depth;

            if (sprite.YStop > sprite.Ground)
            {
                sprite.YStop = sprite.Ground;
            }

            if (sprite.YStart >= sprite.Ground)
            {
                return false;
            }

            sprite.Ground += depth;
        }

        return true;
    }
}
