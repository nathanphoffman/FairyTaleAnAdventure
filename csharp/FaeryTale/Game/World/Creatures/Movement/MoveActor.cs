using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Creatures.Movement;

/// Moving and animating one actor per frame (MoveActor): the hero, the raft, the swan and turtle,
/// the dragon, people, and enemies. It picks the sprite image, walks the actor (sliding on ice,
/// bumping into walls, sinking in water, falling in pits), starts arrows and fireballs, plays the
/// death animation, and works out where the actor is on screen.
///
/// The original used goto to jump between the actor kinds. Each label became a Pc value and each
/// stretch of code between labels a method that returns the next Pc, so the order of the original
/// is kept. The methods are in the other files of this partial class.
internal sealed partial class MoveContext
{
    /// Where the next stretch of code continues; one value per goto label of the original.
    internal enum Pc
    {
        Dispatch,
        CheckDev1,
        CheckDev2,
        NewLoc,
        Blocked,
        Still,
        DragShoot,
        Cpx,
        Tail,
        Raise,
        Statc,
    }

    // Actor states that have no shared name yet.
    private const short OSCIL = 17;
    private const short FROZEN = 20;

    /// Not a region number: nothing is waiting to load, so the terrain state may be stored.
    private const ushort NO_REGION = 10;

    // Regions below this wrap around at the world's edges.
    private const ushort OUTDOOR_REGIONS = 8;
    private const ushort WORLD_MIN = 300;
    private const ushort WORLD_MAX = 32565;

    /// The swan's sprite file; the turtle's is any other.
    private const short SWAN_FILE = 11;

    // Riding values: on the raft, on the turtle, on the swan.
    private const short RIDING_RAFT = 1;
    private const short RIDING_TURTLE = 5;
    private const short RIDING_SWAN = 11;

    /// The carrier's slot in the actor list when a turtle or swan is out.
    private const int CARRIER_SLOT = 3;

    // Terrain types (the value TerrainAt returns) and what the actor's environ becomes for them.
    private const short TERRAIN_SHALLOW = 2;
    private const short TERRAIN_WADE = 3;
    private const short TERRAIN_SLOW_SINK = 4;
    private const short TERRAIN_DEEP_WATER = 5;
    private const short TERRAIN_SLOW = 6;
    private const short TERRAIN_ICE = 7;
    private const short TERRAIN_BACKWARDS = 8;
    private const short TERRAIN_PIT = 9;
    private const sbyte ENVIRON_ICE = -2;
    private const sbyte ENVIRON_BACKWARDS = -3;
    private const sbyte ENVIRON_SLOW = -1;
    private const sbyte ENVIRON_SHALLOW = 2;
    private const sbyte ENVIRON_WADE = 5;

    // Environ where an actor is in over its head, and the depth at which it starts to sink.
    private const sbyte ENVIRON_DROWNED = 30;
    private const sbyte ENVIRON_SINKING = 15;

    /// The sector with the pit that drops the hero into the dungeon region.
    private const short PIT_SECTOR = 181;

    /// The extent type of the room with the pit floor.
    private const ushort PIT_XTYPE = 52;

    /// Arrows are not fired in extents with an extent type above this ("pax" areas).
    private const ushort NO_SHOOT_XTYPE = 80;

    // Sprite frames.
    private const short FRAME_SINK = 83;
    private const short FRAME_FLAT = 82;
    private const short FRAME_SLEEP = 86;
    private const short FRAME_OSCIL = 84;
    private const short FRAME_FRUSTRATED = 40;

    // Fixed inventory slots: arrows, the fire-proof item, the gold key, and the swan item.
    private const int STUFF_ARROWS = 8;
    private const int STUFF_FIREPROOF = 23;
    private const int STUFF_GOLD_KEY_SLOT = 30;
    private const int STUFF_SWAN_ITEM = 5;

    // Objects the dead leave behind: the talisman and the lasso.
    private const short TALISMAN = 139;
    private const short LASSO = 27;

    // Races with special handling.
    private const byte RACE_WRAITH = 2;
    private const byte RACE_SNAKE = 4;
    private const byte RACE_WITCH = 0x09;
    private const byte RACE_WITCH_BODY = 10;
    private const byte RACE_LASSO_CARRIER = 0x89;

    /// The setfig id of the witch.
    private const short SETFIG_WITCH = 9;

    // Terrain codes BlockedAt returns for doors.
    private const short BLOCK_DOOR = 15;
    private const short BLOCK_PASSABLE_WITH_KEY = 12;

    // Dragon and fireball constants.
    private const sbyte MISSILE_SPEED_FIREBALL = 5;
    private const sbyte MISSILE_SPEED_ARROW = 3;
    private const sbyte MISSILE_ROCK_TYPE = 2;
    private const sbyte WEAPON_BOW = 4;
    private const sbyte WEAPON_FIREBALL = 5;
    private const short MAX_MISSILE = 5;

    /// Facing a dragon takes when it breathes (it does not turn to the hero).
    private const sbyte DRAGON_BREATH_FACING = 5;

    /// Hitting the pit: the luck it costs.
    private const short PIT_LUCK_COST = 2;

    // The sound of a fireball and its base pitch.
    private const short EFFECT_FIREBALL = 5;
    private const int EFFECT_FIREBALL_PITCH = 1800;

    /// A person's race is the low 7 bits of the race.
    private const byte PERSON_MASK = 0x7f;

    // A person's dying and dead frames come this far after the standing one (the witch's, 2 further).
    private const short PERSON_DYING_OFFSET = 2;
    private const short PERSON_DEAD_OFFSET = 3;
    private const short WITCH_EXTRA_OFFSET = 2;

    // The dragon's frames when dying and when dead.
    private const short DRAGON_DYING_FRAME = 3;
    private const short DRAGON_DEAD_FRAME = 4;

    /// States 0 to 8 are the fighting swings; after them come the other states.
    private const short LAST_FIGHT_STATE = 8;

    /// Facings are 0-7 (clockwise from north-west).
    private const short FACING_MASK = 7;

    /// How far the turtle swims in a frame.
    private const short TURTLE_STEP = 3;

    private readonly Data.Game game;
    private readonly IOutput output;
    private readonly Shape actor;
    private readonly int slot;
    private readonly short cycle;
    private readonly int missileIndex;

    private sbyte environ;
    private short facing;
    private short state;
    private short image;
    private short frameBase;
    private short terrainHere;
    private short step;

    private MoveContext(Data.Game game, IOutput output, int slot, short cycle, short terrainHere)
    {
        this.game = game;
        this.output = output;
        this.slot = slot;
        this.cycle = cycle;
        this.terrainHere = terrainHere;
        actor = game.Actors.List[slot];
        missileIndex = game.Actors.NextMissile;
    }

    private Missile CurrentMissile => game.Actors.Missiles[missileIndex];

    /// Moves and animates actor i; cycle is the frame counter. jp is the main loop's j, which
    /// carries the last terrain value from one actor to the next (a dead actor reads it without
    /// setting it), so it is passed in and written back.
    public static void MoveActor(Data.Game game, IOutput output, int i, short cycle, ref short jp)
    {
        MoveContext context = new(game, output, i, cycle, jp);
        Pc pc = context.Start();

        while (pc != Pc.Statc)
        {
            pc = context.Run(pc);
        }

        context.Statc();
        jp = context.terrainHere;
    }

    // Sets up the working variables, and where to start: frozen enemies skip straight to the end,
    // everyone else is dispatched on their kind and state.
    private Pc Start()
    {
        if (game.World.FreezeTimer != 0 && slot > 0)
        {
            return Pc.Statc;
        }

        environ = actor.Environ;
        facing = actor.Facing;
        state = actor.State;
        image = actor.Index;
        frameBase = DirectionOffset(facing);

        return Pc.Dispatch;
    }

    private Pc Run(Pc pc) => pc switch
    {
        Pc.Dispatch => Dispatch(),
        Pc.CheckDev1 => CheckDev1(),
        Pc.CheckDev2 => CheckDev2(),
        Pc.NewLoc => NewLoc(),
        Pc.Blocked => Blocked(),
        Pc.Still => Still(),
        Pc.DragShoot => DragShoot(),
        Pc.Cpx => Cpx(),
        Pc.Tail => Tail(),
        _ => Raise(),
    };

    // Picks the code for this kind of actor and state (the if chain at the top of the original).
    private Pc Dispatch()
    {
        sbyte kind = actor.Kind;

        if (kind == ActorKind.OBJECTS)
        {
            terrainHere = 0;

            return Pc.Raise;
        }

        if (kind == ActorKind.DRAGON)
        {
            return Dragon();
        }

        if (kind == ActorKind.CARRIER)
        {
            return Carrier();
        }

        if (kind == ActorKind.SETFIG)
        {
            return SetFig();
        }

        if (kind == ActorKind.RAFT)
        {
            return Raft();
        }

        return DispatchByState();
    }

    private Pc DispatchByState()
    {
        if (state == ActorState.SINK)
        {
            if (actor.Vitality > 0)
            {
                image = FRAME_SINK;
            }

            game.Actors.Frustration = 0;

            return Pc.Cpx;
        }

        if (state == ActorState.WALKING)
        {
            return Walking();
        }

        if (state == ActorState.STILL)
        {
            return Pc.Still;
        }

        if (state >= ActorState.SHOOT1)
        {
            return Shooting();
        }

        return state <= LAST_FIGHT_STATE ? Fighting() : OtherStates();
    }

    // The start of the facing's frames in the sprite sheet. Like the original this does not check
    // the range; facings are 0-7 (+8 for shooting).
    private static short DirectionOffset(int direction) => Tables.Entry(Tables.FacingFrameOffsets, direction);

    private static ushort PosX(int x, int direction, int speed) => (ushort)MapSteps.StepX(x, direction, speed);

    private static ushort PosY(int y, int direction, int speed) => (ushort)MapSteps.StepY(y, direction, speed);

    private short TerrainAtPoint(ushort x, ushort y) => (short)MapLookup.TerrainAt(game, x, y);
}
