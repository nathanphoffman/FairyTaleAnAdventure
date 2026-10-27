//! Moving and animating one actor per frame (`move_actor`, from the main loop in csrc/fmain.c): the
//! hero, the raft, the swan and turtle, the dragon, people, and enemies. It picks the sprite image,
//! walks the actor (sliding on ice, bumping into walls, sinking in water, falling in pits), starts
//! arrows and fireballs, plays the death animation, and works out where the actor is on screen.
//!
//! The C used `goto` to jump between the actor kinds. Each label became a `Pc` value and each stretch
//! of code between labels a method that returns the next `Pc`, so the order of the original is kept.

mod after_move;
mod carriers;
mod dragon;
mod people;
mod shooting;
mod walking;
mod water;

use self::water::*;

use crate::game::actors::encounters::check_death;
use crate::game::actors::interaction::{blocked_at, center_extent_on};
use crate::game::actors::steering::steer_actor;
use crate::game::data::object_tables::{bowshotx, bowshoty, fallstates, gunshoty};
use crate::game::data::random::{rand2, rand4, rand256};
use crate::game::data::state::{the_actors, the_hero, the_witch, the_world};
use crate::game::data::tables::{facing_frame_offsets, setfig_table, swing_transitions};
use crate::game::data::types::{Missile, Shape};
use crate::game::files::loading::play_effect;
use crate::game::ui::messages::print;
use crate::game::ui::music::set_mood;
use crate::game::world::key_doors::{bumped_locked_door, open_door_near};
use crate::game::world::map::{move_camera_to, step_x, step_y, terrain_at, wrap_coordinate};
use crate::game::world::objects::{SetFigEntry, leave_item};
use crate::game::world::places::{find_place, transfer_hero};

// Actor kinds and states (csrc/ftale.h).
const OBJECTS: i8 = 1;
const ENEMY: i8 = 2;
const RAFT: i8 = 3;
const SETFIG: i8 = 4;
const CARRIER: i8 = 5;
const DRAGON: i8 = 6;
const WALKING: i16 = 12;
const STILL: i8 = 13;
const DYING: i16 = 14;
const DEAD: i16 = 15;
const SINK: i16 = 16;
const OSCIL: i16 = 17;
const TALKING: i16 = 19;
const FROZEN: i16 = 20;
const FALL: i16 = 22;
const SLEEP: i16 = 23;
const SHOOT1: i16 = 24;
const SHOOT3: i16 = 25;
/// The tactic that means "frustrated, try something else".
const FRUST: i8 = 0;

/// Not a region number: nothing is waiting to load, so the terrain state may be stored.
const NO_REGION: u16 = 10;
/// Regions below this wrap around at the world's edges.
const OUTDOOR_REGIONS: u16 = 8;
const WORLD_MIN: u16 = 300;
const WORLD_MAX: u16 = 32565;

/// The swan's sprite file; the turtle's is any other.
const SWAN_FILE: i16 = 11;
/// `riding` values: on the raft, on the turtle, on the swan.
const RIDING_RAFT: i16 = 1;
const RIDING_TURTLE: i16 = 5;
const RIDING_SWAN: i16 = 11;
/// The carrier's slot in the actor list when a turtle or swan is out.
const CARRIER_SLOT: usize = 3;
/// Terrain type the turtle swims in.
const TURTLE_WATER: i16 = 5;

// Terrain types (the value `terrain_at` returns) and what the actor's `environ` becomes for them.
const TERRAIN_SLOW_SINK: i16 = 4;
const TERRAIN_DEEP_WATER: i16 = 5;
const ENVIRON_ICE: i8 = -2;
const ENVIRON_BACKWARDS: i8 = -3;
const ENVIRON_SLOW: i8 = -1;
/// `environ` where an actor is in over its head, and the depth at which it starts to sink.
const ENVIRON_DROWNED: i8 = 30;
const ENVIRON_SINKING: i8 = 15;
/// The sector with the pit that drops the hero into the dungeon region.
const PIT_SECTOR: i16 = 181;
/// The `extent_type` of the room with the pit floor.
const PIT_XTYPE: u16 = 52;
/// Arrows are not fired in extents with `extent_type` above this ("pax" areas).
const NO_SHOOT_XTYPE: u16 = 80;
/// Sprite frames.
const FRAME_SINK: i16 = 83;
const FRAME_FLAT: i16 = 82;
const FRAME_SLEEP: i16 = 86;
const FRAME_OSCIL: i16 = 84;
const FRAME_FRUSTRATED: i16 = 40;
/// Fixed stuff slots (inventory): arrows, and the fire-proof item.
const STUFF_ARROWS: usize = 8;
const STUFF_FIREPROOF: usize = 23;
const STUFF_GOLD_KEY_SLOT: usize = 30;
const STUFF_SWAN_ITEM: usize = 5;
/// Objects the dead leave behind: the talisman and the lasso.
const TALISMAN: i16 = 139;
const LASSO: i16 = 27;
/// Races with special handling.
const RACE_WRAITH: u8 = 2;
const RACE_SNAKE: u8 = 4;
const RACE_WITCH: u8 = 0x09;
const RACE_WITCH_BODY: u8 = 10;
const RACE_LASSO_CARRIER: u8 = 0x89;
/// The setfig id of the witch.
const SETFIG_WITCH: i16 = 9;
/// Door-find terrain codes `blocked_at` returns.
const BLOCK_DOOR: i16 = 15;
const BLOCK_PASSABLE_WITH_KEY: i16 = 12;
/// Dragon and fireball constants.
const MISSILE_SPEED_FIREBALL: i8 = 5;
const MISSILE_SPEED_ARROW: i8 = 3;
const MISSILE_ROCK_TYPE: i8 = 2;
const WEAPON_BOW: i8 = 4;
const WEAPON_FIREBALL: i8 = 5;
const MAX_MISSILE: i16 = 5;
/// Facing a dragon takes when it breathes (it does not turn to the hero).
const DRAGON_BREATH_FACING: i8 = 5;
/// Hitting the pit: the luck it costs.
const PIT_LUCK_COST: i16 = 2;
/// Burning in lava, and running out of breath: dtype for `check_death`.
const DEATH_LAVA: i32 = 27;
const DEATH_DROWN: i32 = 6;

/// Where the next stretch of code continues; one value per `goto` label of the C.
#[derive(Clone, Copy)]
enum Pc {
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

/// `xtest`/`ytest` stay static: a dead actor reads them without setting them, so they carry over from
/// the previous actor.
static mut XTEST: u16 = 0;
static mut YTEST: u16 = 0;

/// The working variables of one `move_actor` call (the C locals).
struct Ctx {
    pub(super) an: *mut Shape,
    pub(super) ms: *mut Missile,
    pub(super) i: i32,
    pub(super) cycle: i16,
    /// The actor's environ (terrain state).
    pub(super) k: i8,
    /// Facing.
    pub(super) d: i16,
    /// State.
    pub(super) s: i16,
    /// Sprite image index.
    pub(super) dex: i16,
    /// Start of the facing's frames in the sprite sheet.
    pub(super) inum: i16,
    /// Terrain type under the actor.
    pub(super) j: i16,
    pub(super) e: i16,
}

fn diroff(d: i16) -> i16 {
    // Like the C this does not check the range; facings are 0-7 (+8 for shooting).
    unsafe { *(&raw const facing_frame_offsets as *const i8).offset(d as isize) as i16 }
}

fn pos_x(x: i32, dir: i16, speed: i16) -> u16 {
    step_x(x, dir as i32, speed as i32) as u16
}

fn pos_y(y: i32, dir: i16, speed: i16) -> u16 {
    step_y(y, dir as i32, speed as i32) as u16
}

fn terrain(x: u16, y: u16) -> i16 {
    terrain_at(x as i32, y as i32) as i16
}

/// Moves and animates actor `i`; `cycle` is the frame counter. `*jp` is the main loop's `j`, which
/// carries the last terrain value from one actor to the next (a dead actor reads it without setting
/// it), so it is passed in and written back.
pub fn move_actor(i: i32, cycle: i16, jp: *mut i16) {
    unsafe {
        let an = &raw mut the_actors.list[i as usize];
        let mut c = Ctx {
            an,
            ms: (&raw mut the_actors.missiles as *mut Missile).offset(0),
            i,
            cycle,
            k: 0,
            d: 0,
            s: 0,
            dex: 0,
            inum: 0,
            j: *jp,
            e: 0,
        };
        let mut pc = if the_world.freeze_timer != 0 && i > 0 {
            Pc::Statc
        } else {
            c.k = (*an).environ;
            c.d = (*an).facing as i16;
            c.s = (*an).state as i16;
            c.dex = (*an).index as i16;
            c.inum = diroff(c.d);
            c.ms = (&raw mut the_actors.missiles as *mut Missile)
                .offset(the_actors.next_missile as isize);
            Pc::Dispatch
        };
        loop {
            pc = match pc {
                Pc::Dispatch => c.dispatch(),
                Pc::CheckDev1 => c.check_dev1(),
                Pc::CheckDev2 => c.check_dev2(),
                Pc::NewLoc => c.new_loc(),
                Pc::Blocked => c.blocked(),
                Pc::Still => c.still(),
                Pc::DragShoot => c.drag_shoot(),
                Pc::Cpx => c.cpx(),
                Pc::Tail => c.tail(),
                Pc::Raise => c.raise(),
                Pc::Statc => break,
            };
        }
        c.statc();
        *jp = c.j;
    }
}

impl Ctx {
    /// Picks the code for this kind of actor and state (the `if` chain at the top of the C).
    unsafe fn dispatch(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let kind = (*an).kind;
            if kind == OBJECTS {
                self.j = 0;
                Pc::Raise
            } else if kind == DRAGON {
                self.dragon()
            } else if kind == CARRIER {
                self.carrier()
            } else if kind == SETFIG {
                self.setfig()
            } else if kind == RAFT {
                self.raft()
            } else if self.s == SINK {
                if (*an).vitality > 0 {
                    self.dex = FRAME_SINK;
                }
                the_actors.frustration = 0;
                Pc::Cpx
            } else if self.s == WALKING {
                self.walking()
            } else if self.s == STILL as i16 {
                Pc::Still
            } else if self.s >= SHOOT1 {
                self.shooting()
            } else if self.s < 9 {
                self.fighting()
            } else {
                self.other_states()
            }
        }
    }
}
