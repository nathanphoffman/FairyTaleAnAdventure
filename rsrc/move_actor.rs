//! Moving and animating one actor per frame (`move_actor`, from the main loop in csrc/fmain.c): the
//! hero, the raft, the swan and turtle, the dragon, people, and enemies. It picks the sprite image,
//! walks the actor (sliding on ice, bumping into walls, sinking in water, falling in pits), starts
//! arrows and fireballs, plays the death animation, and works out where the actor is on screen.
//!
//! The C used `goto` to jump between the actor kinds. Each label became a `Pc` value and each stretch
//! of code between labels a method that returns the next `Pc`, so the order of the original is kept.

use super::actors::{blocked_at, center_extent_on};
use super::asm_map::{move_camera_to, step_x, step_y, terrain_at, wrap_coordinate};
use super::asm_text::steer_actor;
use super::doors::{bumped_locked_door, open_door_near};
use super::encounters::check_death;
use super::game_tables::{facing_frame_offsets, setfig_table, swing_transitions};
use super::loading::play_effect;
use super::messages::print;
use super::mood::set_mood;
use super::objects::{SetFigEntry, leave_item};
use super::places::{find_place, transfer_hero};
use super::random::{rand2, rand4, rand256};
use super::state::{the_actors, the_hero, the_witch, the_world};
use super::tables::{bowshotx, bowshoty, fallstates, gunshoty};
use super::types::{Missile, Shape};

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

/// The swan carries the hero once he is near, with the swan on its slot and the lasso in the pack.
fn swan_carries(nearness: i16, carrier_slot: i16, has_lasso: bool) -> bool {
    nearness != 0 && carrier_slot == CARRIER_SLOT as i16 && has_lasso
}

/// The turtle carries the hero whenever he is near and on its slot.
fn turtle_carries(nearness: i16, carrier_slot: i16) -> bool {
    nearness != 0 && carrier_slot == CARRIER_SLOT as i16
}

/// The turtle's direction of swimming: straight on while there is deep water ahead, otherwise one
/// step right, then two to the left of that, then one more to the left; the last is used whether
/// or not it is water. `water_ahead(dir)` says if the turtle could swim that way.
fn turtle_heading(dir: i16, water_ahead: impl Fn(i16) -> bool) -> i16 {
    if water_ahead(dir) {
        return dir;
    }
    let right = (dir + 1) & 7;
    if water_ahead(right) {
        return right;
    }
    let left = (right - 2) & 7;
    if water_ahead(left) {
        return left;
    }
    (left - 1) & 7
}

/// Standing in water (depth `k` above 2) on ground that is dry (`j` 0), or wading with a depth over
/// the limit of the terrain (3: 5, 4: 10): the actor climbs out one step a frame.
fn climbs_out_of_water(k: i8, j: i16) -> bool {
    k > 2 && (j == 0 || (j == 3 && k > 5) || (j == 4 && k > 10))
}

/// The hero's frame once the way has been blocked for a while: oscillating, then a frustrated one.
fn frustrated_frame(frustration: i8, cycle: i16) -> Option<i16> {
    if frustration > 40 {
        Some(FRAME_FRUSTRATED)
    } else if frustration > 20 {
        Some(FRAME_OSCIL + ((cycle >> 1) & 1))
    } else {
        None
    }
}

/// What wading does to an actor this frame.
#[derive(Debug, PartialEq, Eq)]
enum Wading {
    Nothing,
    /// Fully under: the actor stops moving (or falls through the pit).
    Drowned,
    /// Deep enough to show the sinking frame.
    Sinking,
}

/// The depth after one frame in water of terrain type `j` (shallow 4 or deep 5), and what that does.
/// The depth moves one step towards its limit; the dead are not affected.
fn wade_step(depth: i8, j: i16, dead_or_dying: bool) -> (i8, Wading) {
    let limit: i8 = if j == TERRAIN_SLOW_SINK { 10 } else { ENVIRON_DROWNED };
    if depth > limit {
        (depth - 1, Wading::Nothing)
    } else if depth < limit {
        let depth = depth + 1;
        let outcome = if dead_or_dying {
            Wading::Nothing
        } else if depth == ENVIRON_DROWNED {
            Wading::Drowned
        } else if depth > ENVIRON_SINKING {
            Wading::Sinking
        } else {
            Wading::Nothing
        };
        (depth, outcome)
    } else {
        (depth, Wading::Nothing)
    }
}

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
    an: *mut Shape,
    ms: *mut Missile,
    i: i32,
    cycle: i16,
    /// The actor's environ (terrain state).
    k: i8,
    /// Facing.
    d: i16,
    /// State.
    s: i16,
    /// Sprite image index.
    dex: i16,
    /// Start of the facing's frames in the sprite sheet.
    inum: i16,
    /// Terrain type under the actor.
    j: i16,
    e: i16,
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

    /// The dragon: it always shows image 0 (the C sets the dying and dead images and then overwrites
    /// them), and now and then breathes a fireball.
    unsafe fn dragon(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.dex = 0;
            if self.s == DYING {
                self.dex = 3;
            } else if self.s == DEAD {
                self.dex = 4;
            } else if rand4() == 0 {
                (*self.ms).speed = MISSILE_SPEED_FIREBALL;
                the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                self.dex = (rand2() + 1) as i16;
                play_effect(5, 1800 + rand256());
                (*an).facing = DRAGON_BREATH_FACING;
                (*self.ms).missile_type = MISSILE_ROCK_TYPE;
                return Pc::DragShoot;
            }
            self.dex = 0;
            Pc::Tail
        }
    }

    /// The swan (sprite file 11) or the turtle: carries the hero when the hero is close enough.
    unsafe fn carrier(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.k = 0;
            self.j = 0;
            let hero = &raw const the_actors.list[0];
            if the_actors.loaded_actor_file == SWAN_FILE {
                if swan_carries(
                    the_actors.raft_nearness,
                    the_actors.carrier_slot,
                    *the_hero.stuff.add(STUFF_SWAN_ITEM) != 0,
                ) {
                    self.d = (*hero).facing as i16;
                    (*an).abs_x = (*hero).abs_x;
                    XTEST = (*hero).abs_x;
                    (*an).abs_y = (*hero).abs_y;
                    YTEST = (*hero).abs_y;
                    the_hero.riding = RIDING_SWAN;
                } else {
                    XTEST = (*an).abs_x;
                    YTEST = (*an).abs_y;
                }
                self.dex = self.d;
                self.e = 0;
            } else {
                if turtle_carries(the_actors.raft_nearness, the_actors.carrier_slot) {
                    self.d = (*hero).facing as i16;
                    XTEST = (*hero).abs_x;
                    YTEST = (*hero).abs_y;
                    the_hero.riding = RIDING_TURTLE;
                    self.dex = self.d + self.d;
                    if (*hero).state as i16 == WALKING {
                        self.dex += self.cycle & 1;
                    }
                } else {
                    self.turtle_swims();
                }
                self.j = terrain(XTEST, YTEST);
                if self.j == TURTLE_WATER {
                    (*an).abs_x = XTEST;
                    (*an).abs_y = YTEST;
                }
                self.e = 1;
            }
            center_extent_on(self.e, XTEST as i16, YTEST as i16);
            Pc::Raise
        }
    }

    /// The turtle on its own: keeps going the same way while there is deep water ahead, otherwise
    /// tries turning right, then left, then back to straight on the left again.
    unsafe fn turtle_swims(&mut self) {
        unsafe {
            let an = self.an;
            let (ax, ay) = ((*an).abs_x as i32, (*an).abs_y as i32);
            self.d = turtle_heading(self.d, |dir| {
                terrain(pos_x(ax, dir, 3), pos_y(ay, dir, 3)) == TURTLE_WATER
            });
            XTEST = pos_x(ax, self.d, 3);
            YTEST = pos_y(ay, self.d, 3);
            the_hero.riding = 0;
            self.dex = self.d + self.d + (self.cycle & 1);
        }
    }

    /// People standing about: images from the setfig table, talking, and the witch watching the hero.
    unsafe fn setfig(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.j = ((*an).race & 0x7f) as i16;
            let entry = &*(&raw const setfig_table)
                .cast::<SetFigEntry>()
                .add(self.j as usize);
            self.dex = entry.image_base as i16;
            let witch = self.j == SETFIG_WITCH;
            if self.s == DYING {
                self.dex += 2;
                if witch {
                    self.dex += 2;
                }
            } else if self.s == DEAD {
                self.dex += 3;
                if witch {
                    self.dex += 2;
                }
            } else if witch {
                steer_actor(self.i as i32, the_hero.x as i32, the_hero.y as i32, 0);
                self.dex = ((*an).facing / 2) as i16;
                the_witch.on_screen = 1;
            } else if self.s == TALKING {
                self.dex += rand2() as i16;
                (*an).tactic = (*an).tactic.wrapping_sub(1);
                if (*an).tactic == 0 {
                    (*an).state = STILL;
                }
            } else {
                return Pc::Statc;
            }
            self.k = 0;
            self.j = 0;
            Pc::Tail
        }
    }

    /// The raft: follows the hero when the hero is very near and the terrain is water.
    unsafe fn raft(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            the_hero.riding = 0;
            if the_actors.carrier_slot != RIDING_RAFT || the_actors.raft_nearness != 2 {
                return Pc::Statc;
            }
            XTEST = the_actors.list[0].abs_x;
            YTEST = the_actors.list[0].abs_y;
            self.j = terrain(XTEST, YTEST);
            if self.j < 3 || self.j > 5 {
                return Pc::Statc;
            }
            (*an).abs_x = XTEST;
            (*an).abs_y = YTEST;
            the_hero.riding = RIDING_RAFT;
            Pc::Statc
        }
    }

    /// Walking: sliding on ice, picking a step, and trying to go around what is in the way.
    unsafe fn walking(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            if self.k == ENVIRON_ICE {
                if let Some(next) = self.slide_on_ice() {
                    return next;
                }
                self.k = 0;
            }
            let k = self.k;
            self.e = if self.i == 0 && the_hero.riding == RIDING_TURTLE {
                3
            } else if k == ENVIRON_BACKWARDS {
                -2
            } else if k == ENVIRON_SLOW {
                4
            } else if k == 2 || k > 6 {
                1
            } else {
                2
            };
            XTEST = pos_x((*an).abs_x as i32, self.d, self.e);
            YTEST = pos_y((*an).abs_y as i32, self.d, self.e);
            self.j = blocked_at(XTEST as i16, YTEST as i16, self.i as i16) as i16;
            if self.i == 0 {
                if self.j == BLOCK_DOOR {
                    open_door_near(XTEST, YTEST, 0);
                } else {
                    bumped_locked_door = 0;
                }
                if *the_hero.stuff.add(STUFF_GOLD_KEY_SLOT) != 0
                    && self.j == BLOCK_PASSABLE_WITH_KEY
                {
                    return Pc::NewLoc;
                }
            }
            if self.j != 0 {
                Pc::CheckDev1
            } else {
                Pc::NewLoc
            }
        }
    }

    /// On ice the actor keeps its momentum: pushes the velocity toward the wanted direction up to a
    /// limit and moves by it. Returns where to go next, or `None` when it hit something and should
    /// walk normally this tick.
    unsafe fn slide_on_ice(&mut self) -> Option<Pc> {
        unsafe {
            let an = self.an;
            self.e = if the_hero.riding == RIDING_SWAN {
                40
            } else {
                42
            };
            let nvx = ((*an).vel_x as i32 + step_x(20, self.d as i32, 2) - 20) as i16;
            let nvy = ((*an).vel_y as i32 + step_y(20, self.d as i32, 2) - 20) as i16;
            if nvx.abs() < self.e - 8 {
                (*an).vel_x = nvx as i8;
            }
            if nvy.abs() < self.e {
                (*an).vel_y = nvy as i8;
            }
            XTEST = ((*an).abs_x as i32 + (*an).vel_x as i32 / 4) as u16;
            YTEST = ((*an).abs_y as i32 + (*an).vel_y as i32 / 4) as u16;
            if the_hero.riding == RIDING_SWAN {
                steer_actor(0, -(nvx as i32), -(nvy as i32), 6);
                self.d = (*an).facing as i16;
                return Some(Pc::NewLoc);
            }
            if blocked_at(XTEST as i16, YTEST as i16, self.i as i16) == 0 {
                return Some(Pc::NewLoc);
            }
            None
        }
    }

    /// Blocked: try one direction to the right.
    unsafe fn check_dev1(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.d = (self.d + 1) & 7;
            XTEST = pos_x((*an).abs_x as i32, self.d, self.e);
            YTEST = pos_y((*an).abs_y as i32, self.d, self.e);
            if blocked_at(XTEST as i16, YTEST as i16, self.i as i16) != 0 {
                Pc::CheckDev2
            } else {
                Pc::NewLoc
            }
        }
    }

    /// Still blocked: try one direction to the left of the original.
    unsafe fn check_dev2(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.d = (self.d - 2) & 7;
            XTEST = pos_x((*an).abs_x as i32, self.d, self.e);
            YTEST = pos_y((*an).abs_y as i32, self.d, self.e);
            if blocked_at(XTEST as i16, YTEST as i16, self.i as i16) != 0 {
                Pc::Blocked
            } else {
                Pc::NewLoc
            }
        }
    }

    /// A free step was found: face that way, pick the walking frame, and move (unless the terrain
    /// slows the actor to a halt).
    unsafe fn new_loc(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            (*an).facing = self.d as i8;
            self.inum = diroff(self.d);
            self.dex = self.inum;
            if !(the_hero.riding != 0 && self.i == 0) && (*an).race != RACE_WRAITH {
                self.dex = (self.dex as i32 + ((self.cycle as i32 + self.i) & 7)) as i16;
            }
            if self.k == ENVIRON_ICE {
                return Pc::Cpx;
            }
            if self.k == ENVIRON_BACKWARDS {
                self.dex ^= 7;
            }
            self.j = terrain(XTEST, YTEST);
            // wraiths and snakes are immune to water
            if (*an).race == RACE_WRAITH || (*an).race == RACE_SNAKE {
                self.j = 0;
            }
            if (*an).race == RACE_SNAKE {
                self.dex = (self.cycle & 2) + ((self.d & 0xfe) << 2) + 0x24;
            }
            if climbs_out_of_water(self.k, self.j) {
                if the_hero.sector != PIT_SECTOR {
                    self.k -= 1;
                }
                return Pc::Raise;
            }
            (*an).vel_x = ((XTEST.wrapping_sub((*an).abs_x) as i16 as i32) * 4) as i8;
            (*an).vel_y = ((YTEST.wrapping_sub((*an).abs_y) as i16 as i32) * 4) as i8;
            (*an).abs_x = XTEST;
            (*an).abs_y = YTEST;
            the_actors.frustration = 0;
            Pc::Tail
        }
    }

    /// Walking found no way through. The hero shows a frustrated frame after a while; others change
    /// tactic. (The C placed this label inside the `STILL` branch.)
    unsafe fn blocked(&mut self) -> Pc {
        unsafe {
            if self.i == 0 {
                the_actors.frustration = the_actors.frustration.wrapping_add(1);
                if let Some(frame) = frustrated_frame(the_actors.frustration, self.cycle) {
                    self.dex = frame;
                }
                return Pc::Cpx;
            }
            (*self.an).tactic = FRUST;
            Pc::Still
        }
    }

    /// Standing still: the frame after the first of the facing's walking frames.
    unsafe fn still(&mut self) -> Pc {
        unsafe {
            self.dex = self.inum + 1;
            if self.k == ENVIRON_ICE {
                return Pc::Cpx;
            }
            self.j = terrain((*self.an).abs_x, (*self.an).abs_y);
            Pc::Tail
        }
    }

    /// Aiming and firing a bow or a fireball. No shooting under water or in peaceful places.
    unsafe fn shooting(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let ms = self.ms;
            let d8 = self.d + 8;
            if self.k > ENVIRON_SINKING || the_world.extent_type > NO_SHOOT_XTYPE {
                return Pc::Cpx;
            }
            if self.s == SHOOT3 {
                if (*an).weapon == WEAPON_FIREBALL {
                    self.dex = diroff(d8);
                    return Pc::Cpx;
                }
                self.dex = diroff(d8) + 11;
                (*an).state = STILL;
                if self.i == 0 {
                    if *the_hero.stuff.add(STUFF_ARROWS) == 0 {
                        return Pc::Cpx;
                    }
                    *the_hero.stuff.add(STUFF_ARROWS) -= 1;
                }
                (*ms).speed = MISSILE_SPEED_ARROW;
                the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                play_effect(4, 400 + rand256());
            } else if self.s == SHOOT1 {
                self.dex = diroff(d8) + if (*an).kind == ENEMY { 11 } else { 10 };
                if (*an).weapon == WEAPON_FIREBALL {
                    (*ms).speed = MISSILE_SPEED_FIREBALL;
                    the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                    (*an).state = SHOOT3 as i8;
                    self.dex = diroff(d8);
                    play_effect(5, 1800 + rand256());
                } else if self.i == 0 && *the_hero.stuff.add(STUFF_ARROWS) == 0 {
                    print(c"No Arrows!".as_ptr());
                    return Pc::Cpx;
                } else {
                    (*ms).speed = 0;
                }
            }
            (*ms).missile_type = (*an).weapon - 3;
            Pc::DragShoot
        }
    }

    /// Sets up the missile just started (also used by the dragon): where it starts, which way it
    /// goes, and who shot it.
    unsafe fn drag_shoot(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let ms = self.ms;
            let d = self.d as usize;
            let dx = bowshotx.get(d).copied().unwrap_or(0) as i32;
            (*ms).abs_x = ((*an).abs_x as i32 + dx) as u16;
            (*ms).abs_y = (*an).abs_y;
            let dy = if (*an).weapon == WEAPON_BOW {
                bowshoty.get(d).copied().unwrap_or(0)
            } else {
                gunshoty.get(d).copied().unwrap_or(0)
            };
            (*ms).abs_y = ((*ms).abs_y as i32 + dy as i32) as u16;
            (*ms).time_of_flight = 0;
            (*ms).direction = (*an).facing;
            (*ms).archer = self.i as i8;
            if the_actors.next_missile > MAX_MISSILE {
                the_actors.next_missile = 0;
            }
            the_actors.frustration = 0;
            Pc::Cpx
        }
    }

    /// Swinging a weapon (states 0-8): steps through the swing table.
    unsafe fn fighting(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.inum = diroff(self.d + 8);
            let next = swing_transitions[self.s as usize][rand4() as usize] as i16;
            (*an).state = next as i8;
            self.s = next;
            if (self.i > 2 && self.s == 6) || self.s == 7 {
                self.s = 8;
            }
            self.dex = self.s + self.inum;
            the_actors.frustration = 0;
            Pc::Cpx
        }
    }

    /// The remaining states: dying, dead, frozen, oscillating, asleep and falling.
    unsafe fn other_states(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let s = self.s;
            if s == DYING {
                let d = self.d;
                if (*an).tactic > 4 {
                    self.dex = if d == 0 || d > 4 { 80 } else { 81 };
                } else if (*an).tactic > 0 {
                    self.dex = if d == 0 || d > 4 { 81 } else { 80 };
                } else {
                    (*an).state = DEAD as i8;
                    self.dex = FRAME_FLAT;
                }
                the_actors.frustration = 0;
            } else if s == DEAD || s == FROZEN {
                self.dex = FRAME_FLAT;
            } else if s == OSCIL {
                self.dex = FRAME_OSCIL + (self.cycle & 1);
            } else if s == OSCIL + 1 {
                self.dex = FRAME_OSCIL;
            } else if s == SLEEP {
                self.dex = FRAME_SLEEP;
            } else if s == FALL && (*an).tactic < 30 {
                self.j = (*an).tactic as i16 / 5 + the_hero.brother * 6;
                self.dex = fallstates.get(self.j as usize).copied().unwrap_or(0) as i16;
                (*an).tactic += 1;
                (*an).vel_x = ((*an).vel_x as i32 * 3 / 4) as i8;
                (*an).vel_y = ((*an).vel_y as i32 * 3 / 4) as i8;
            }
            Pc::Cpx
        }
    }

    /// Works out the terrain under the actor, and carries on sliding if it is on ice.
    unsafe fn cpx(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.j = terrain((*an).abs_x, (*an).abs_y);
            if self.k == ENVIRON_ICE {
                (*an).abs_x = ((*an).abs_x as i32 + (*an).vel_x as i32 / 4) as u16;
                (*an).abs_y = ((*an).abs_y as i32 + (*an).vel_y as i32 / 4) as u16;
            }
            Pc::Tail
        }
    }

    /// After the movement: finishes a death, then turns the terrain under the actor into its `environ`
    /// (water depth, ice, the pit).
    unsafe fn tail(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let i = self.i;
            if self.s == DYING {
                (*an).tactic = (*an).tactic.wrapping_sub(1);
                if (*an).tactic == 0 {
                    (*an).state = DEAD as i8;
                    if (*an).race == RACE_WITCH {
                        (*an).race = RACE_WITCH_BODY;
                        (*an).vitality = 10;
                        (*an).state = STILL;
                        (*an).weapon = 0;
                        leave_item(i as i16, TALISMAN);
                    }
                    if (*an).race == RACE_LASSO_CARRIER {
                        leave_item(i as i16, LASSO);
                    }
                }
            }
            let j = self.j;
            if i == 0 && the_actors.raft_nearness != 0 {
                self.k = 0; // can't drown on the raft
            } else if j == 0 {
                self.k = 0;
            } else if j == 6 {
                self.k = ENVIRON_SLOW;
            } else if j == 7 {
                self.k = ENVIRON_ICE;
            } else if j == 8 {
                self.k = ENVIRON_BACKWARDS;
            } else if j == 9 && i == 0 && the_world.extent_type == PIT_XTYPE {
                if (*an).state as i16 != FALL {
                    self.dex = fallstates[(the_hero.brother * 6) as usize] as i16;
                    (*an).state = FALL as i8;
                    (*an).tactic = 0;
                    the_hero.luck = the_hero.luck.wrapping_sub(PIT_LUCK_COST);
                    set_mood(1);
                }
                self.k = ENVIRON_ICE;
            } else if j == 2 {
                self.k = 2;
            } else if j == 3 {
                self.k = 5;
            } else if j == TERRAIN_SLOW_SINK || j == TERRAIN_DEEP_WATER {
                self.wade(j);
            }
            if self.k == 0 && self.s == SINK {
                (*an).state = STILL;
            }
            Pc::Raise
        }
    }

    /// In water (terrain 4 is shallow, 5 deep): the depth moves one step towards its limit, and the
    /// actor sinks, or at the pit sector falls through to the dungeon.
    unsafe fn wade(&mut self, j: i16) {
        unsafe {
            let an = self.an;
            let (depth, outcome) = wade_step(self.k, j, self.s == DYING || self.s == DEAD);
            self.k = depth;
            match outcome {
                Wading::Nothing => {}
                Wading::Drowned => {
                    if the_hero.sector == PIT_SECTOR {
                        if self.i == 0 {
                            self.k = 0;
                            the_world.new_region = 9;
                            transfer_hero(0x1080, 34950, 0);
                            find_place(1);
                        } else {
                            (*an).vitality = 0;
                        }
                    }
                    (*an).state = STILL;
                }
                Wading::Sinking => (*an).state = SINK as i8,
            }
        }
    }

    /// Stores the environ, adjusts the sprite for some enemies, keeps the hero inside the world and
    /// scrolls the map to it, and applies lava and drowning damage.
    unsafe fn raise(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let i = self.i;
            let cycle = self.cycle as i32;
            if the_world.new_region >= NO_REGION {
                (*an).environ = self.k;
            }
            let race = (*an).race as i8;
            if (*an).kind == ENEMY {
                self.enemy_frame(race);
            }
            (*an).index = self.dex as i8;
            if i == 0 {
                self.hero_wrap();
                the_hero.x = (*an).abs_x;
                the_hero.y = (*an).abs_y;
                move_camera_to(the_hero.x as i32, the_hero.y as i32);
                the_hero.safe_flag = self.j as i8;
            }
            if (*an).vitality == 0 {
                return Pc::Statc;
            }
            if the_actors.fiery_death != 0 {
                if i == 0 && *the_hero.stuff.add(STUFF_FIREPROOF) != 0 {
                    (*an).environ = 0;
                } else if (*an).environ > ENVIRON_SINKING {
                    (*an).vitality = 0;
                } else if (*an).environ > 2 {
                    (*an).vitality = (*an).vitality.wrapping_sub(1);
                }
                check_death(i, DEATH_LAVA);
            }
            if (*an).environ == ENVIRON_DROWNED && (cycle & 7) == 0 && race != 2 && race != 3 {
                (*an).vitality = (*an).vitality.wrapping_sub(1);
                check_death(i, DEATH_DROWN);
            }
            Pc::Statc
        }
    }

    /// The sprite frames of snakes (race 4), the spectre (8) and the race-7 enemy at zero vitality.
    unsafe fn enemy_frame(&mut self, race: i8) {
        unsafe {
            let an = self.an;
            let cycle = self.cycle as i32;
            let state = (*an).state as i16;
            if race == 4 && state < WALKING {
                self.dex = ((cycle & 1) + diroff(self.d) as i32) as i16;
            } else if race == 4 && state < DYING {
                self.dex = (((cycle / 2) & 1) + diroff(self.d) as i32) as i16;
            } else if race == 8 {
                if state == DEAD {
                    (*an).abs_x = 0;
                } else if state == DYING {
                    self.dex = 0x3f;
                } else {
                    self.dex = ((cycle & 3) * 2) as i16;
                    if self.dex > 4 {
                        self.dex -= 1;
                    }
                    match self.i % 3 {
                        0 => self.dex = 0x25,
                        1 => self.dex += 0x28,
                        _ => self.dex += 0x30,
                    }
                }
            } else if race == 7 && (*an).vitality == 0 {
                // The original has `an->state == STILL` here: a comparison that does nothing (an
                // assignment may have been meant). Kept as it was.
                self.dex = 1;
            }
        }
    }

    /// The hero leaving the edge of the world comes back on the other side (outdoor regions only);
    /// whatever the hero rides is moved along.
    unsafe fn hero_wrap(&mut self) {
        unsafe {
            let an = self.an;
            if the_world.region >= OUTDOOR_REGIONS {
                return;
            }
            if (*an).abs_x < WORLD_MIN {
                (*an).abs_x = WORLD_MAX;
            } else if (*an).abs_x > WORLD_MAX {
                (*an).abs_x = WORLD_MIN;
            } else if (*an).abs_y < WORLD_MIN {
                (*an).abs_y = WORLD_MAX;
            } else if (*an).abs_y > WORLD_MAX {
                (*an).abs_y = WORLD_MIN;
            } else {
                return;
            }
            if the_hero.riding > 1 {
                the_actors.list[CARRIER_SLOT].abs_x = (*an).abs_x;
                the_actors.list[CARRIER_SLOT].abs_y = (*an).abs_y;
            }
        }
    }

    /// Where the actor is on screen: relative to the map, with the sprite's anchor offset.
    unsafe fn statc(&mut self) {
        unsafe {
            let an = self.an;
            let (ox, oy) = if (*an).kind == CARRIER && the_hero.riding == RIDING_SWAN {
                (32, 40)
            } else if (*an).kind == RAFT || (*an).kind == CARRIER || (*an).kind == DRAGON {
                (16, 16)
            } else {
                (8, 26)
            };
            (*an).rel_x = wrap_coordinate((*an).abs_x as i32 - the_world.map_x as i32 - ox) as u16;
            (*an).rel_y = wrap_coordinate((*an).abs_y as i32 - the_world.map_y as i32 - oy) as u16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_swan_needs_the_lasso_and_the_turtle_does_not() {
        assert!(swan_carries(1, CARRIER_SLOT as i16, true));
        assert!(!swan_carries(1, CARRIER_SLOT as i16, false));
        assert!(!swan_carries(0, CARRIER_SLOT as i16, true));
        assert!(!swan_carries(1, CARRIER_SLOT as i16 + 1, true));
        assert!(turtle_carries(1, CARRIER_SLOT as i16));
        assert!(!turtle_carries(0, CARRIER_SLOT as i16));
        assert!(!turtle_carries(1, 0));
    }

    #[test]
    fn the_turtle_keeps_to_the_water() {
        // straight on, then right, then two left of that, then one more left
        assert_eq!(turtle_heading(2, |_| true), 2);
        assert_eq!(turtle_heading(2, |d| d == 3), 3);
        assert_eq!(turtle_heading(2, |d| d == 1), 1);
        assert_eq!(turtle_heading(2, |_| false), 0);
        // the compass wraps
        assert_eq!(turtle_heading(7, |d| d == 0), 0);
        assert_eq!(turtle_heading(0, |d| d == 7), 7);
    }

    #[test]
    fn actors_climb_out_of_shallow_water() {
        assert!(climbs_out_of_water(3, 0));
        assert!(!climbs_out_of_water(2, 0));
        assert!(climbs_out_of_water(6, 3));
        assert!(!climbs_out_of_water(5, 3));
        assert!(climbs_out_of_water(11, 4));
        assert!(!climbs_out_of_water(10, 4));
        assert!(!climbs_out_of_water(20, 5));
    }

    #[test]
    fn being_stuck_shows_impatience() {
        assert_eq!(frustrated_frame(20, 0), None);
        assert_eq!(frustrated_frame(21, 0), Some(FRAME_OSCIL));
        assert_eq!(frustrated_frame(21, 2), Some(FRAME_OSCIL + 1));
        assert_eq!(frustrated_frame(41, 0), Some(FRAME_FRUSTRATED));
    }

    #[test]
    fn wading_deeper_sinks_then_drowns() {
        // deep water (5): wade towards ENVIRON_DROWNED
        assert_eq!(wade_step(0, 5, false), (1, Wading::Nothing));
        assert_eq!(wade_step(ENVIRON_SINKING, 5, false), (ENVIRON_SINKING + 1, Wading::Sinking));
        assert_eq!(wade_step(ENVIRON_DROWNED - 1, 5, false), (ENVIRON_DROWNED, Wading::Drowned));
        assert_eq!(wade_step(ENVIRON_DROWNED, 5, false), (ENVIRON_DROWNED, Wading::Nothing));
        // shallow water (4) stops at 10 and comes back up from deeper
        assert_eq!(wade_step(9, 4, false), (10, Wading::Nothing));
        assert_eq!(wade_step(10, 4, false), (10, Wading::Nothing));
        assert_eq!(wade_step(12, 4, false), (11, Wading::Nothing));
        // the dead are not affected
        assert_eq!(wade_step(ENVIRON_SINKING, 5, true), (ENVIRON_SINKING + 1, Wading::Nothing));
    }
}
