using FaeryTale.Game.Data;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Movement;

// Shooting arrows and fireballs: starting a missile when an actor lets go, and the dragon's fire.
internal sealed partial class MoveContext
{
    // The facing frames for shooting come after the 8 walking ones (facing + 8); the bow-drawn and
    // released frames sit 10 or 11 on from the shooting frame (11 for enemies).
    private const short SHOOTING_FACING_OFFSET = 8;
    private const short BOW_RELEASED_FRAME = 11;
    private const short HERO_BOW_DRAWN_FRAME = 10;
    private const short ENEMY_BOW_DRAWN_FRAME = 11;

    /// A weapon number of 4 or more is a missile weapon; the missile type is the weapon minus 3.
    private const sbyte MISSILE_WEAPON_BASE = 3;

    // Sound effects for the bow, and its base pitch.
    private const short EFFECT_ARROW = 4;
    private const int EFFECT_ARROW_PITCH = 400;

    // Aiming and firing a bow or a fireball. No shooting under water or in peaceful places.
    private Pc Shooting()
    {
        if (environ > ENVIRON_SINKING || game.World.ExtentType > NO_SHOOT_XTYPE)
        {
            return Pc.Cpx;
        }

        Pc? earlyExit = null;

        if (state == ActorState.SHOOT3)
        {
            earlyExit = ReleaseShot();
        }
        else if (state == ActorState.SHOOT1)
        {
            earlyExit = StartShot();
        }

        if (earlyExit is { } exit)
        {
            return exit;
        }

        CurrentMissile.MissileType = (sbyte)(actor.Weapon - MISSILE_WEAPON_BASE);

        return Pc.DragShoot;
    }

    // The shot is let go: a fireball just stays drawn, an arrow leaves the bow (and one is used up
    // for the hero). A result means nothing is fired.
    private Pc? ReleaseShot()
    {
        short shooting = (short)(facing + SHOOTING_FACING_OFFSET);

        if (actor.Weapon == WEAPON_FIREBALL)
        {
            image = DirectionOffset(shooting);

            return Pc.Cpx;
        }

        image = (short)(DirectionOffset(shooting) + BOW_RELEASED_FRAME);
        actor.State = ActorState.STILL;

        if (slot == 0)
        {
            if (game.Hero.Stuff[STUFF_ARROWS] == 0)
            {
                return Pc.Cpx;
            }

            game.Hero.Stuff[STUFF_ARROWS]--;
        }

        CurrentMissile.Speed = MISSILE_SPEED_ARROW;
        game.Actors.NextMissile++;
        Music.PlayEffect(game, EFFECT_ARROW, EFFECT_ARROW_PITCH + game.Rng.Rand256());

        return null;
    }

    // The actor starts to shoot: a fireball is fired at once, a bow needs arrows (for the hero). A
    // result means nothing is fired.
    private Pc? StartShot()
    {
        short shooting = (short)(facing + SHOOTING_FACING_OFFSET);

        image = (short)(DirectionOffset(shooting) + (actor.Kind == ActorKind.ENEMY ? ENEMY_BOW_DRAWN_FRAME : HERO_BOW_DRAWN_FRAME));

        if (actor.Weapon == WEAPON_FIREBALL)
        {
            CurrentMissile.Speed = MISSILE_SPEED_FIREBALL;
            game.Actors.NextMissile++;
            actor.State = ActorState.SHOOT3;
            image = DirectionOffset(shooting);
            Music.PlayEffect(game, EFFECT_FIREBALL, EFFECT_FIREBALL_PITCH + game.Rng.Rand256());
        }
        else if (slot == 0 && game.Hero.Stuff[STUFF_ARROWS] == 0)
        {
            output.Print("No Arrows!");

            return Pc.Cpx;
        }
        else
        {
            CurrentMissile.Speed = 0;
        }

        return null;
    }

    // The dragon: it always shows image 0 (the original sets the dying and dead images and then
    // overwrites them), and now and then breathes a fireball.
    private Pc Dragon()
    {
        image = 0;

        if (state == ActorState.DYING)
        {
            image = DRAGON_DYING_FRAME;
        }
        else if (state == ActorState.DEAD)
        {
            image = DRAGON_DEAD_FRAME;
        }
        else if (game.Rng.Rand4() == 0)
        {
            CurrentMissile.Speed = MISSILE_SPEED_FIREBALL;
            game.Actors.NextMissile++;
            image = (short)(game.Rng.Rand2() + 1);
            Music.PlayEffect(game, EFFECT_FIREBALL, EFFECT_FIREBALL_PITCH + game.Rng.Rand256());
            actor.Facing = DRAGON_BREATH_FACING;
            CurrentMissile.MissileType = MISSILE_ROCK_TYPE;

            return Pc.DragShoot;
        }

        image = 0;

        return Pc.Tail;
    }

    // Sets up the missile just started (also used by the dragon): where it starts, which way it
    // goes, and who shot it.
    private Pc DragShoot()
    {
        Missile missile = CurrentMissile;
        int d = facing;
        int dx = d >= 0 && d < ObjectTables.Bowshotx.Length ? ObjectTables.Bowshotx[d] : 0;

        missile.AbsX = (ushort)(actor.AbsX + dx);
        missile.AbsY = actor.AbsY;

        sbyte[] table = actor.Weapon == WEAPON_BOW ? ObjectTables.Bowshoty : ObjectTables.Gunshoty;
        int dy = d >= 0 && d < table.Length ? table[d] : 0;

        missile.AbsY = (ushort)(missile.AbsY + dy);
        missile.TimeOfFlight = 0;
        missile.Direction = actor.Facing;
        missile.Archer = (sbyte)slot;

        if (game.Actors.NextMissile > MAX_MISSILE)
        {
            game.Actors.NextMissile = 0;
        }

        game.Actors.Frustration = 0;

        return Pc.Cpx;
    }
}
