//! Placing monsters and carriers, and what happens when an actor dies (from csrc/fmain.c).

use super::actors::proxcheck;
use super::asm_map::px_to_im;
use super::asm_text::{event, prq, speak};
use super::hdrive::motor_off;
use super::loading::{prep, read_shapes};
use super::mood::setmood;
use super::random::{bitrand, rand2, rand4};
use super::tables::weapon_probs;
use super::types::{Extent, SeqInfo, Shape};

unsafe extern "C" {
    static mut anim_list: [Shape; 20];
    static mut extent_list: [Extent; 0];
    static extn: *const Extent;
    static mut seq_list: [SeqInfo; 0];
    static mut nextshape: *mut u8;
    static mut anix: i16;
    static mut actor_file: i16;
    static mut active_carrier: i16;
    static encounter_x: u16;
    static encounter_y: u16;
    static encounter_type: u16;
    static xtype: u16;
    static map_x: u16;
    static map_y: u16;
    static mixflag: i16;
    static mut wt: i16;
    static encounter_chart: [Encounter; 5];
    #[link_name = "kind"]
    static mut hero_kind: i16;
    static mut brave: i16;
    static mut luck: i16;
}

/// One kind of monster (`struct encounter`): hit points, how aggressive, which weapon row it uses,
/// how clever, what treasure it drops, and which sprite file it needs.
#[repr(C)]
struct Encounter {
    hitpoints: i8,
    agressive: i8,
    arms: i8,
    cleverness: i8,
    treasure: i8,
    file_id: i8,
}

// Actor kinds (`type`), states and goals (see csrc/ftale.h).
const ENEMY: i8 = 2;
const SETFIG: i8 = 4;
const CARRIER: i8 = 5;
const DRAGON: i8 = 6;
const STILL: i8 = 13;
const DYING: i8 = 14;
const DEAD: i8 = 15;
const ATTACK1: i8 = 1;
const ARCHER1: i8 = 3;
const DEATH: i8 = 7;
/// A weapon with this bit set is a bow, so the monster is an archer.
const BOW_BIT: i8 = 4;

/// Tries this many random spots before giving up.
const MAX_TRY: i32 = 15;
/// The extent value (`v3`) of the special encounter that always appears at a fixed place.
const FIXED_PLACE_ENCOUNTER: u8 = 7;
const FIXED_PLACE: (u16, u16) = (21635, 25762);
/// The spider pit extent type, where monsters may stand on terrain 7.
const SPIDER_PIT: u16 = 52;
const TERRAIN_SWAMP: i32 = 7;
/// The monster that is the dark knight and the kinds of people that matter in `checkdead`.
const RACE_DARK_KNIGHT: u8 = 7;
const RACE_LASSO_KEEPER: u8 = 0x89;
const SPEAK_DARK_KNIGHT_DIES: i32 = 42;
/// Penalties and rewards for a death.
const KINDNESS_LOSS: i16 = 3;
const LUCK_LOSS: i16 = 5;
const DYING_COUNTDOWN: i8 = 7;
/// Print queue entries: vitality and stats.
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;
/// Carrier numbers.
const CARRIER_DRAGON: i16 = 10;
const CARRIER_TURTLE: i16 = 5;
const VITALITY_CARRIER: i16 = 50;
const SEQ_ENEMY: usize = 2;
const FIRST_CARRIER_SLOT: usize = 3;

/// Places a new monster in slot `i` of `anim_list` somewhere within `spread` pixels of the
/// encounter spot (`encounter_x`, `encounter_y`), on a free square. Its race is `encounter_type`
/// (mixed with the one next to it when `mixflag` bit 1 is set). Returns 1 if it was placed and
/// 0 if no free spot was found in 15 tries.
#[unsafe(no_mangle)]
pub extern "C" fn set_encounter(i: u16, spread: u16) -> i32 {
    unsafe {
        let an = (&raw mut anim_list).cast::<Shape>().add(i as usize);
        let mut xtest: u16 = 0;
        let mut ytest: u16 = 0;
        let mut tries = 0;
        if (*extn).v3 == FIXED_PLACE_ENCOUNTER {
            (xtest, ytest) = FIXED_PLACE;
        } else {
            while tries < MAX_TRY {
                xtest = (encounter_x as i32 + bitrand(spread as i32) - (spread as i32 / 2)) as u16;
                ytest = (encounter_y as i32 + bitrand(spread as i32) - (spread as i32 / 2)) as u16;
                if proxcheck(xtest as i16, ytest as i16, i as i16) == 0 {
                    break;
                }
                if xtype == SPIDER_PIT && px_to_im(xtest as i32, ytest as i32) == TERRAIN_SWAMP {
                    break;
                }
                tries += 1;
            }
        }
        if tries == MAX_TRY {
            return 0;
        }
        (*an).abs_x = xtest;
        (*an).abs_y = ytest;
        (*an).kind = ENEMY;
        let race = if (mixflag & 2) != 0 && encounter_type != 4 {
            (encounter_type & 0xfffe) as i32 + rand2()
        } else {
            encounter_type as i32
        };
        (*an).race = race as u8;
        if mixflag & 4 != 0 {
            wt = rand4() as i16;
        }
        let chart = &encounter_chart[race as usize];
        let w = (chart.arms as i32 * 4) + wt as i32;
        (*an).weapon = weapon_probs[w as usize];
        (*an).state = STILL;
        (*an).environ = 0;
        (*an).facing = 0;
        if (*an).weapon & BOW_BIT != 0 {
            (*an).goal = ARCHER1 + chart.cleverness;
        } else {
            (*an).goal = ATTACK1 + chart.cleverness;
        }
        (*an).vitality = chart.hitpoints as i16;
        (*an).rel_x = (*an).abs_x.wrapping_sub(map_x).wrapping_sub(8);
        (*an).rel_y = (*an).abs_y.wrapping_sub(map_y).wrapping_sub(26);
        1
    }
}

/// Starts dying if actor `i` is out of vitality (and is not already dying or dead): it becomes
/// DYING, the hero gains bravery for a kill, and killing a person (not the lasso keeper) costs
/// 3 kindness. If it is the hero, `dtype` is the death message, luck drops by 5 and the music
/// changes. The hero's stats are always refreshed on screen.
#[unsafe(no_mangle)]
pub extern "C" fn checkdead(i: i64, dtype: i64) {
    unsafe {
        let an = (&raw mut anim_list).cast::<Shape>().add(i as usize);
        if (*an).vitality < 1 && (*an).state != DYING && (*an).state != DEAD {
            (*an).vitality = 0;
            (*an).tactic = DYING_COUNTDOWN;
            (*an).goal = DEATH;
            (*an).state = DYING;
            if (*an).race == RACE_DARK_KNIGHT {
                speak(SPEAK_DARK_KNIGHT_DIES);
            } else if (*an).kind == SETFIG && (*an).race != RACE_LASSO_KEEPER {
                hero_kind -= KINDNESS_LOSS;
            }
            if i != 0 {
                brave += 1;
            } else {
                event(dtype as i32);
                luck -= LUCK_LOSS;
                setmood(1);
            }
            if hero_kind < 0 {
                hero_kind = 0;
            }
            prq(PRQ_VITALITY);
        }
        if i == 0 {
            prq(PRQ_STATS);
        }
    }
}

/// Puts carrier `n` (5 turtle, 10 dragon, otherwise the swan) in actor slot 3, loading its sprites
/// if they are not the ones already loaded, in the middle of its extent.
#[unsafe(no_mangle)]
pub extern "C" fn load_carrier(n: i16) {
    unsafe {
        let an = (&raw mut anim_list).cast::<Shape>().add(FIRST_CARRIER_SLOT);
        (*an).kind = if n == CARRIER_DRAGON { DRAGON } else { CARRIER };
        let extent = if n == CARRIER_DRAGON {
            2
        } else if n == CARRIER_TURTLE {
            1
        } else {
            0
        };
        if actor_file != n {
            nextshape = (*(&raw mut seq_list).cast::<SeqInfo>().add(SEQ_ENEMY)).location;
            read_shapes(n as i32);
            prep((*an).kind as i16);
            motor_off();
        }
        let e = &*(&raw mut extent_list).cast::<Extent>().add(extent);
        (*an).abs_x = e.x1.wrapping_add(250);
        (*an).abs_y = e.y1.wrapping_add(200);
        (*an).index = 0;
        (*an).weapon = 0;
        (*an).environ = 0;
        (*an).state = STILL;
        (*an).vitality = VITALITY_CARRIER;
        anix = 4;
        active_carrier = n;
        actor_file = n;
        (*an).race = n as u8;
    }
}
