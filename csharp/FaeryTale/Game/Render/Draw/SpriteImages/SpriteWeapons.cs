using FaeryTale.Game.Data;

namespace FaeryTale.Game.Render;

/// SpriteImages: the weapon images.
internal static partial class SpriteImages
{
    /// Says whether this pass draws the weapon (true) or the body. An actor with a weapon is drawn
    /// in two passes, the weapon first or second depending on which way it faces.
    internal static bool WeaponPassmode(Shape actor, World world, ref byte pass)
    {
        bool passmode = false;
        sbyte state = actor.State;
        bool resting = state >= ActorState.DEAD && state < ActorState.SHOOT1;

        if (actor.Weapon > 0 && actor.Weapon <= LAST_HELD_WEAPON && (!resting || world.ExtentType > PEACEFUL_XTYPE))
        {
            int facing = actor.Facing;

            // The other of the two passes (0 becomes 1, 1 becomes 0).
            byte flipped = (byte)(pass ^ 1);
            byte pick = ((facing - FACING_PICK_OFFSET) & FACING_LEFT_BIT) != 0 ? flipped : pass;

            if (actor.Weapon == WEAPON_BOW && state < ActorState.SHOOT1)
            {
                pick = (facing & FACING_LEFT_BIT) == 0 ? flipped : pass;
            }

            passmode = pick != 0;
            pass++;
        }

        return passmode;
    }

    // The picture of the bow for an animation frame: it is drawn up, middle or down as the frames go.
    private static short BowImage(short frameNumber)
    {
        int phase = frameNumber / BOW_PHASE_FRAMES;

        if ((phase & 1) != 0)
        {
            return OBJ_BOW_UP;
        }

        return (phase & 2) != 0 ? OBJ_BOW_MID : OBJ_BOW_DOWN;
    }

    // The weapon's sprite: where it is held for this animation frame, and which image it is.
    private static Sprite WeaponImage(Shape actor, Sprite sprite)
    {
        short kind = actor.Weapon;
        short frameNumber = sprite.Inum;

        if (kind == WEAPON_BOW && frameNumber < BOW_FRAMES)
        {
            sprite.XStart += ByteOf(ObjectTables.BowX, frameNumber);
            sprite.YStart += ByteOf(ObjectTables.BowY, frameNumber);
            sprite.Inum = BowImage(frameNumber);
        }
        else
        {
            StateEntry entry = StateEntryFor(frameNumber);

            sprite.XStart += entry.WpnX;
            sprite.YStart += entry.WpnY;

            if (kind == WEAPON_FIREBALL)
            {
                sprite.Inum = (short)(actor.Facing + OBJ_ARROW_BASE);

                if (actor.Facing == FIREBALL_FACING_EAST)
                {
                    sprite.YStart -= FIREBALL_EAST_LIFT;
                }
            }
            else
            {
                sprite.Inum = (short)(entry.WpnNo + HandWeaponImages(kind));
            }
        }

        sprite.Atype = OBJECTS;

        return sprite;
    }

    // Hand weapons each have their own block of images; the first image of the block for kind.
    private static short HandWeaponImages(short kind)
    {
        foreach ((sbyte weapon, short imageBase) in WEAPON_SPRITE_BASE)
        {
            if (weapon == kind)
            {
                return imageBase;
            }
        }

        return kind;
    }
}
