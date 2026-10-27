//! Placing monsters and carriers, and what happens when an actor dies (from csrc/fmain.c).

use super::actors::blocked_at;
use super::asm_map::terrain_at;
use super::asm_text::{queue_print, show_event, speak};
use super::game_tables::{encounter_chart, extent_list};
use super::globals::{
    active_actor_count, active_carrier, actor_list, actors_loading, brave, current_extent,
    encounter_number, encounter_type, encounter_x, encounter_y, extent_type, kind as hero_kind,
    loaded_actor_file, luck, map_x, map_y, mixflag, nextshape, seq_list, wt,
};
use super::hdrive::motor_off;
use super::loading::{finish_sprite_load, load_sprite_file};
use super::mood::set_mood;
use super::random::{bitrand, rand2, rand4, rnd};
use super::tables::weapon_probs;
use super::types::{Extent, SeqInfo, Shape};

/// One kind of monster (`struct encounter`): hit points, how aggressive, which weapon row it uses,
/// how clever, what treasure it drops, and which sprite file it needs.
#[repr(C)]
pub struct Encounter {
    pub hitpoints: i8,
    pub agressive: i8,
    pub arms: i8,
    pub cleverness: i8,
    pub treasure: i8,
    pub file_id: i8,
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
/// The monster that is the dark knight and the kinds of people that matter in `check_death`.
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

/// Places a new monster in slot `i` of `actor_list` somewhere within `spread` pixels of the
/// encounter spot (`encounter_x`, `encounter_y`), on a free square. Its race is `encounter_type`
/// (mixed with the one next to it when `mixflag` bit 1 is set). Returns 1 if it was placed and
/// 0 if no free spot was found in 15 tries.
pub fn place_encounter(i: u16, spread: u16) -> i32 {
    unsafe {
        let an = (&raw mut actor_list).cast::<Shape>().add(i as usize);
        let Some((xtest, ytest)) = find_open_spot(i, spread) else {
            return 0;
        };
        (*an).abs_x = xtest;
        (*an).abs_y = ytest;
        (*an).kind = ENEMY;
        let race = encounter_race();
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

/// Finds a place near the encounter point for actor `i`, trying up to `MAX_TRY` random spots
/// within `spread` pixels. Some encounters always use one fixed place. Returns `None` if every
/// spot was blocked.
unsafe fn find_open_spot(i: u16, spread: u16) -> Option<(u16, u16)> {
    unsafe {
        if (*current_extent).v3 == FIXED_PLACE_ENCOUNTER {
            return Some(FIXED_PLACE);
        }
        for _ in 0..MAX_TRY {
            let xtest = (encounter_x as i32 + bitrand(spread as i32) - (spread as i32 / 2)) as u16;
            let ytest = (encounter_y as i32 + bitrand(spread as i32) - (spread as i32 / 2)) as u16;
            if blocked_at(xtest as i16, ytest as i16, i as i16) == 0 {
                return Some((xtest, ytest));
            }
            if extent_type == SPIDER_PIT && terrain_at(xtest as i32, ytest as i32) == TERRAIN_SWAMP
            {
                return Some((xtest, ytest));
            }
        }
        None
    }
}

/// The race of the next enemy: for mixed groups the two races of the pair are picked at random
/// (except encounter type 4).
unsafe fn encounter_race() -> i32 {
    unsafe {
        if (mixflag & 2) != 0 && encounter_type != 4 {
            (encounter_type & 0xfffe) as i32 + rand2()
        } else {
            encounter_type as i32
        }
    }
}

/// Starts dying if actor `i` is out of vitality (and is not already dying or dead): it becomes
/// DYING, the hero gains bravery for a kill, and killing a person (not the lasso keeper) costs
/// 3 kindness. If it is the hero, `dtype` is the death message, luck drops by 5 and the music
/// changes. The hero's stats are always refreshed on screen.
#[unsafe(export_name = "checkdead")]
pub extern "C" fn check_death(i: i32, dtype: i32) {
    unsafe {
        let an = (&raw mut actor_list).cast::<Shape>().add(i as usize);
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
                show_event(dtype);
                luck -= LUCK_LOSS;
                set_mood(1);
            }
            if hero_kind < 0 {
                hero_kind = 0;
            }
            queue_print(PRQ_VITALITY);
        }
        if i == 0 {
            queue_print(PRQ_STATS);
        }
    }
}

/// Puts carrier `n` (5 turtle, 10 dragon, otherwise the swan) in actor slot 3, loading its sprites
/// if they are not the ones already loaded, in the middle of its extent.
pub fn load_carrier(n: i16) {
    unsafe {
        let an = (&raw mut actor_list)
            .cast::<Shape>()
            .add(FIRST_CARRIER_SLOT);
        (*an).kind = if n == CARRIER_DRAGON { DRAGON } else { CARRIER };
        let extent = if n == CARRIER_DRAGON {
            2
        } else if n == CARRIER_TURTLE {
            1
        } else {
            0
        };
        if loaded_actor_file != n {
            nextshape = (*(&raw mut seq_list).cast::<SeqInfo>().add(SEQ_ENEMY)).location;
            load_sprite_file(n as i32);
            finish_sprite_load((*an).kind as i16);
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
        active_actor_count = 4;
        active_carrier = n;
        loaded_actor_file = n;
        (*an).race = n as u8;
    }
}

/// The actors' sprite file the monsters of this encounter need: sets how many appear and, if that
/// file is not the one loaded, loads it (the carrier is dropped then).
pub fn load_enemy_sprites() {
    const FIRST_ENEMY_SLOT: i16 = 3;
    unsafe {
        encounter_number = ((*current_extent).v1 as i32 + rnd((*current_extent).v2 as i32)) as u8;
        let file = encounter_chart[encounter_type as usize].file_id as i16;
        if loaded_actor_file != file {
            loaded_actor_file = file;
            active_actor_count = FIRST_ENEMY_SLOT;
            nextshape = (*(&raw const seq_list).cast::<SeqInfo>().add(ENEMY as usize)).location;
            load_sprite_file(loaded_actor_file as i32);
            actors_loading = 1;
            active_carrier = 0;
        }
    }
}
