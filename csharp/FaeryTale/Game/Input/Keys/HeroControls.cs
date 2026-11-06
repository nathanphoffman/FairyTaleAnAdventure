using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Creatures.Revive;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Input.Keys;

/// What the hero is doing each frame: walking, fighting or standing, and the timers that run
/// down, and the good fairy that revives the hero.
public static class HeroControls
{
    /// What the good fairy does this frame.
    internal enum FairyAction
    {
        /// The hero gets up where they are.
        Revive,

        /// No luck is left: the next brother takes over.
        NextBrother,

        /// The fairy flies in toward the hero.
        FlyIn,

        /// Nothing yet (or the resurrection glow).
        Wait,
    }

    // Mouse button bits in the input qualifier.
    private const short LEFT_BUTTON = 0x4000;
    private const short RIGHT_BUTTON = 0x2000;

    // Timers and the bounds of the fiery region (the lava).
    private const ushort LAVA_X_MIN = 8802;
    private const ushort LAVA_X_MAX = 13562;
    private const ushort LAVA_Y_MIN = 24744;
    private const ushort LAVA_Y_MAX = 29544;

    // The good fairy countdown: she appears below this, and the hero revives when it runs out.
    internal const byte FAIRY_ARRIVES = 120;
    internal const byte FAIRY_RESURRECT = 20;
    internal const byte FAIRY_GIVES_UP = 200;
    private const int FAIRY_SLOT = 3;
    private const sbyte FAIRY_IMAGE = 100;
    private const ushort FAIRY_SPEED = 2;
    private const ushort FAIRY_X_OFFSET = 20;
    private const byte FAIRY_RACE = 0xff;
    private const short NO_ACTORS_LOADED = 4;

    // Riding and shooting.
    private const ushort SHOOTING_ZONE = 80;
    private const ushort ZONE_ARROW_SLOW = 81;
    private const ushort ZONE_ARROW_FAST = 82;
    private const int EVENT_ZONE_81 = 15;
    private const int EVENT_ZONE_82 = 16;
    private const short BIRD = 11;
    private const int EVENT_LAVA = 32;
    private const int EVENT_TOO_FAST = 33;
    private const sbyte DISMOUNT_SPEED = 15;
    private const ushort DISMOUNT_STEP = 14;
    private const ushort DISMOUNT_CHECK = 10;
    internal const sbyte WEAPON_BOW = 4;
    internal const sbyte WEAPON_WAND = 5;

    /// Hunger above this makes the hero stumble sideways now and then.
    internal const short STUMBLE_HUNGER = 120;
    private const short NOT_WALKING = 9;
    private const short FACING_MASK = 7;

    /// Counts the timers down and sets the hero's state from the controls: the good fairy when
    /// dead or falling, shooting, riding, fighting or walking. cycle is the main loop's frame counter.
    public static void UpdateHero(Data.Game game, IOutput output, short cycle)
    {
        game.World.LightTimer = TimerStep(game.World.LightTimer);
        game.World.SecretTimer = TimerStep(game.World.SecretTimer);
        game.World.FreezeTimer = TimerStep(game.World.FreezeTimer);

        game.Actors.FieryDeath = (sbyte)(game.World.MapX > LAVA_X_MIN && game.World.MapX < LAVA_X_MAX && game.World.MapY > LAVA_Y_MIN && game.World.MapY < LAVA_Y_MAX ? 1 : 0);

        sbyte state = game.Actors.List[0].State;

        if (state == ActorState.DEAD || state == ActorState.FALL)
        {
            state = FairyRevival(game, output, state, cycle);
        }
        else if (state == ActorState.DYING || state == ActorState.SINK || state == ActorState.SLEEP)
        {
            // Nothing to do.
        }
        else if ((InputHandler.MouseButtons & RIGHT_BUTTON) != 0 || game.Ui.FightKeyHeld != 0 || InputHandler.JoystickFire || game.Ui.PathFighting)
        {
            state = FightingState(game, output, state);
        }
        else if (state == ActorState.SHOOT1)
        {
            state = ActorState.SHOOT3; // release the arrow
        }
        else
        {
            state = WalkingState(game, state);
        }

        game.Actors.List[0].State = state;
    }

    /// A light, secret or freeze timer after one frame: it counts down to 0 and stays there.
    internal static short TimerStep(short timer) => timer != 0 ? (short)(timer - 1) : timer;

    /// The state when the fight button is down and the hero is not already fighting: a bow
    /// shoots, a wand keeps shooting, anything else fights.
    internal static sbyte FightingStateFor(sbyte state, sbyte weapon) => weapon switch
    {
        WEAPON_BOW => ActorState.SHOOT1,
        WEAPON_WAND => Math.Max(state, ActorState.SHOOT1),
        _ => ActorState.FIGHTING,
    };

    /// A starving hero sometimes stumbles one step off the direction walked. dice(n) is a random
    /// number from 0 to n - 1: one draw of 4 says whether the hero trips (0), and only then a draw
    /// of 2 says which way (1 is right).
    internal static short Stumble(short direction, short hunger, Func<int, int> dice)
    {
        if (hunger <= STUMBLE_HUNGER || dice(4) != 0)
        {
            return direction;
        }

        return dice(2) != 0 ? (short)((direction + 1) & FACING_MASK) : (short)((direction - 1) & FACING_MASK);
    }

    /// One step of the fairy countdown for a dead or fallen hero: returns the new timer value and
    /// what to do. At timer 1 the hero is revived; below FAIRY_RESURRECT it is just the glow;
    /// below FAIRY_GIVES_UP the hero is revived (a fallen one) or replaced (no luck); below
    /// FAIRY_ARRIVES the fairy flies in.
    internal static (byte Timer, FairyAction Action) FairyStep(byte timer, short luck, bool falling)
    {
        if (timer == 1)
        {
            return (timer, FairyAction.Revive);
        }

        byte next = unchecked((byte)(timer - 1));
        FairyAction action;

        if (next < FAIRY_RESURRECT)
        {
            action = FairyAction.Wait;
        }
        else if (luck < 1 && next < FAIRY_GIVES_UP)
        {
            action = FairyAction.NextBrother;
        }
        else if (falling && next < FAIRY_GIVES_UP)
        {
            action = FairyAction.Revive;
        }
        else
        {
            action = next < FAIRY_ARRIVES ? FairyAction.FlyIn : FairyAction.Wait;
        }

        return (next, action);
    }

    // The fight button is down: shoot in a shooting zone, dismount, or fight with the weapon held.
    private static sbyte FightingState(Data.Game game, IOutput output, sbyte state)
    {
        Shape hero = game.Actors.List[0];

        if (game.World.ExtentType > SHOOTING_ZONE)
        {
            return EnterShootingZone(game, output, state);
        }

        if (game.Hero.Riding == BIRD)
        {
            TryToDismount(game, output, hero);

            return state;
        }

        if (game.Ui.CompassLitDirection < NOT_WALKING)
        {
            hero.Facing = (sbyte)game.Ui.CompassLitDirection;
        }

        return state >= ActorState.WALKING ? FightingStateFor(state, hero.Weapon) : state;
    }

    // In a shooting zone the hero shoots; the two special zones say so the first time.
    private static sbyte EnterShootingZone(Data.Game game, IOutput output, sbyte state)
    {
        if (state != ActorState.SHOOT1 && game.World.ExtentType == ZONE_ARROW_SLOW)
        {
            TextRoutines.ShowEvent(game, output, EVENT_ZONE_81);
        }

        if (state != ActorState.SHOOT1 && game.World.ExtentType == ZONE_ARROW_FAST)
        {
            TextRoutines.ShowEvent(game, output, EVENT_ZONE_82);
        }

        return ActorState.SHOOT1;
    }

    // On the bird, the fight button tries to land: not over lava, not when moving too fast, and
    // only where there is room to stand.
    private static void TryToDismount(Data.Game game, IOutput output, Shape hero)
    {
        if (game.Actors.FieryDeath != 0)
        {
            TextRoutines.ShowEvent(game, output, EVENT_LAVA);

            return;
        }

        bool slowEnough = hero.VelX > -DISMOUNT_SPEED && hero.VelX < DISMOUNT_SPEED && hero.VelY > -DISMOUNT_SPEED && hero.VelY < DISMOUNT_SPEED;

        if (!slowEnough)
        {
            TextRoutines.ShowEvent(game, output, EVENT_TOO_FAST);

            return;
        }

        ushort y = unchecked((ushort)(hero.AbsY - DISMOUNT_STEP));

        if (Blocking.BlockedAt(game, (short)hero.AbsX, (short)y, 0) == 0
            && Blocking.BlockedAt(game, (short)hero.AbsX, (short)(ushort)(y + DISMOUNT_CHECK), 0) == 0)
        {
            game.Hero.Riding = 0;
            hero.AbsY = y;
        }
    }

    // No fight button: walk the way the controls point (stumbling when starving), or stand still.
    private static sbyte WalkingState(Data.Game game, sbyte state)
    {
        if (game.Ui.CompassLitDirection >= NOT_WALKING)
        {
            return ActorState.STILL;
        }

        game.Ui.CompassLitDirection = Stumble(game.Ui.CompassLitDirection, game.Hero.Hunger, n => game.Rng.Next() & (n - 1));
        game.Actors.List[0].Facing = (sbyte)game.Ui.CompassLitDirection;

        return (InputHandler.MouseButtons & LEFT_BUTTON) != 0 || game.Ui.HeldDirection != 0 || game.Ui.PathWalking ? ActorState.WALKING : state;
    }

    // The good fairy: after the hero dies she flies in and revives him, or the next brother takes
    // over. Returns the hero's new state.
    private static sbyte FairyRevival(Data.Game game, IOutput output, sbyte state, short cycle)
    {
        (byte timer, FairyAction action) = FairyStep(game.Hero.GoodFairyTimer, game.Hero.Luck, game.Actors.List[0].State == ActorState.FALL);

        game.Hero.GoodFairyTimer = timer;

        switch (action)
        {
            case FairyAction.Revive:
                Revival.Revive(game, output, 0);

                return ActorState.STILL;

            case FairyAction.NextBrother:
                Revival.Revive(game, output, 1);

                return ActorState.STILL;

            case FairyAction.FlyIn:
                PlaceFairy(game, cycle);
                break;
        }

        return state;
    }

    // Puts the fairy near the hero, closer each frame, and clears the other actors away.
    private static void PlaceFairy(Data.Game game, short cycle)
    {
        Shape fairy = game.Actors.List[FAIRY_SLOT];

        game.Actors.Active = NO_ACTORS_LOADED;
        fairy.AbsX = unchecked((ushort)(game.Hero.X + game.Hero.GoodFairyTimer * FAIRY_SPEED - FAIRY_X_OFFSET));
        fairy.AbsY = game.Hero.Y;
        fairy.Kind = ActorKind.OBJECTS;
        fairy.Index = (sbyte)(FAIRY_IMAGE + (cycle & 1));
        fairy.State = ActorState.STILL;
        fairy.Weapon = 0;
        fairy.Environ = 0;
        fairy.Race = FAIRY_RACE;
        game.Actors.OnScreen = 1;
        game.Actors.BattleFlag = 0;
    }
}
