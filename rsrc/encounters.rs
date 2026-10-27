//! Placing monsters and carriers, and what happens when an actor dies (from csrc/fmain.c).

use super::actors::blocked_at;
use super::asm_map::terrain_at;
use super::asm_text::{queue_print, show_event, speak};
use super::game_tables::{encounter_chart, extent_list};
use super::hdrive::motor_off;
use super::loading::{finish_sprite_load, load_sprite_file};
use super::mood::set_mood;
use super::random::{bitrand, rand2, rand4, rnd};
use super::state::{Hero, the_actors, the_encounter, the_hero, the_memory, the_world};
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
        let an = (&raw mut the_actors.list).cast::<Shape>().add(i as usize);
        let Some((xtest, ytest)) = find_open_spot(i, spread) else {
            return 0;
        };
        (*an).abs_x = xtest;
        (*an).abs_y = ytest;
        (*an).kind = ENEMY;
        let race = encounter_race();
        (*an).race = race as u8;
        if the_encounter.mix_flag & 4 != 0 {
            the_encounter.weapon_type = rand4() as i16;
        }
        let chart = &encounter_chart[race as usize];
        let w = (chart.arms as i32 * 4) + the_encounter.weapon_type as i32;
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
        (*an).rel_x = (*an).abs_x.wrapping_sub(the_world.map_x).wrapping_sub(8);
        (*an).rel_y = (*an).abs_y.wrapping_sub(the_world.map_y).wrapping_sub(26);
        1
    }
}

/// Finds a place near the encounter point for actor `i`, trying up to `MAX_TRY` random spots
/// within `spread` pixels. Some encounters always use one fixed place. Returns `None` if every
/// spot was blocked.
unsafe fn find_open_spot(i: u16, spread: u16) -> Option<(u16, u16)> {
    unsafe {
        if (*the_world.extent).v3 == FIXED_PLACE_ENCOUNTER {
            return Some(FIXED_PLACE);
        }
        for _ in 0..MAX_TRY {
            let xtest =
                (the_encounter.x as i32 + bitrand(spread as i32) - (spread as i32 / 2)) as u16;
            let ytest =
                (the_encounter.y as i32 + bitrand(spread as i32) - (spread as i32 / 2)) as u16;
            if blocked_at(xtest as i16, ytest as i16, i as i16) == 0 {
                return Some((xtest, ytest));
            }
            if the_world.extent_type == SPIDER_PIT
                && terrain_at(xtest as i32, ytest as i32) == TERRAIN_SWAMP
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
        if (the_encounter.mix_flag & 2) != 0 && the_encounter.kind != 4 {
            (the_encounter.kind & 0xfffe) as i32 + rand2()
        } else {
            the_encounter.kind as i32
        }
    }
}

/// Starts dying if actor `i` is out of vitality (and is not already dying or dead): it becomes
/// DYING, the hero gains bravery for a kill, and killing a person (not the lasso keeper) costs
/// 3 kindness. If it is the hero, `dtype` is the death message, luck drops by 5 and the music
/// changes. The hero's stats are always refreshed on screen.
pub extern "C" fn check_death(i: i32, dtype: i32) {
    unsafe {
        let an = (&raw mut the_actors.list).cast::<Shape>().add(i as usize);
        if let Some(death) = start_dying(i, &mut *an, &mut *(&raw mut the_hero)) {
            if death.dark_knight {
                speak(SPEAK_DARK_KNIGHT_DIES);
            }
            if i == 0 {
                show_event(dtype);
                set_mood(1);
            }
            queue_print(PRQ_VITALITY);
        }
        if i == 0 {
            queue_print(PRQ_STATS);
        }
    }
}

/// What a new death needs shown or said.
#[derive(Debug, PartialEq)]
struct Death {
    dark_knight: bool,
}

/// Starts actor `i` (`actor`) dying if it is out of vitality and not already dying or dead: it
/// goes into the dying state with a countdown. Killing a person (except the lasso keeper) costs
/// the hero kindness; killing anything else gives bravery; the hero's own death costs luck.
/// Returns `None` if nothing happened.
fn start_dying(i: i32, actor: &mut Shape, hero: &mut Hero) -> Option<Death> {
    if actor.vitality >= 1 || actor.state == DYING || actor.state == DEAD {
        return None;
    }
    actor.vitality = 0;
    actor.tactic = DYING_COUNTDOWN;
    actor.goal = DEATH;
    actor.state = DYING;
    let dark_knight = actor.race == RACE_DARK_KNIGHT;
    if !dark_knight && actor.kind == SETFIG && actor.race != RACE_LASSO_KEEPER {
        hero.kindness -= KINDNESS_LOSS;
    }
    if i != 0 {
        hero.brave += 1;
    } else {
        hero.luck -= LUCK_LOSS;
    }
    if hero.kindness < 0 {
        hero.kindness = 0;
    }
    Some(Death { dark_knight })
}

/// Puts carrier `n` (5 turtle, 10 dragon, otherwise the swan) in actor slot 3, loading its sprites
/// if they are not the ones already loaded, in the middle of its extent.
pub fn load_carrier(n: i16) {
    unsafe {
        let an = (&raw mut the_actors.list)
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
        if the_actors.loaded_actor_file != n {
            the_memory.nextshape = (*(&raw mut the_memory.seq_list)
                .cast::<SeqInfo>()
                .add(SEQ_ENEMY))
            .location;
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
        the_actors.active = 4;
        the_actors.active_carrier = n;
        the_actors.loaded_actor_file = n;
        (*an).race = n as u8;
    }
}

/// The actors' sprite file the monsters of this encounter need: sets how many appear and, if that
/// file is not the one loaded, loads it (the carrier is dropped then).
pub fn load_enemy_sprites() {
    const FIRST_ENEMY_SLOT: i16 = 3;
    unsafe {
        the_encounter.number =
            ((*the_world.extent).v1 as i32 + rnd((*the_world.extent).v2 as i32)) as u8;
        let file = encounter_chart[the_encounter.kind as usize].file_id as i16;
        if the_actors.loaded_actor_file != file {
            the_actors.loaded_actor_file = file;
            the_actors.active = FIRST_ENEMY_SLOT;
            the_memory.nextshape = (*(&raw const the_memory.seq_list)
                .cast::<SeqInfo>()
                .add(ENEMY as usize))
            .location;
            load_sprite_file(the_actors.loaded_actor_file as i32);
            the_actors.loading = 1;
            the_actors.active_carrier = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(vitality: i16, state: i8) -> Shape {
        let mut a: Shape = unsafe { std::mem::zeroed() };
        a.vitality = vitality;
        a.state = state;
        a
    }

    fn hero(brave: i16, luck: i16, kindness: i16) -> Hero {
        let mut h: Hero = unsafe { std::mem::zeroed() };
        h.brave = brave;
        h.luck = luck;
        h.kindness = kindness;
        h
    }

    #[test]
    fn a_living_actor_does_not_die() {
        let (mut a, mut h) = (actor(1, 0), hero(10, 10, 10));
        assert_eq!(start_dying(5, &mut a, &mut h), None);
        assert_eq!(a.state, 0);
    }

    #[test]
    fn dying_and_dead_actors_are_not_killed_twice() {
        let mut h = hero(10, 10, 10);
        assert_eq!(start_dying(5, &mut actor(0, DYING), &mut h), None);
        assert_eq!(start_dying(5, &mut actor(0, DEAD), &mut h), None);
        assert_eq!(h.brave, 10);
    }

    #[test]
    fn killing_an_enemy_starts_the_death_and_gives_bravery() {
        let (mut a, mut h) = (actor(-4, 0), hero(10, 10, 10));
        assert_eq!(
            start_dying(5, &mut a, &mut h),
            Some(Death { dark_knight: false })
        );
        assert_eq!(
            (a.vitality, a.state, a.goal, a.tactic),
            (0, DYING, DEATH, DYING_COUNTDOWN)
        );
        assert_eq!((h.brave, h.luck, h.kindness), (11, 10, 10));
    }

    #[test]
    fn killing_a_person_costs_kindness_but_not_below_zero() {
        let mut a = actor(0, 0);
        a.kind = SETFIG;
        let mut h = hero(10, 10, 5);
        start_dying(5, &mut a, &mut h);
        assert_eq!(h.kindness, 2);
        let mut a = actor(0, 0);
        a.kind = SETFIG;
        start_dying(5, &mut a, &mut h);
        assert_eq!(h.kindness, 0);
    }

    #[test]
    fn the_lasso_keeper_is_not_a_murder() {
        let mut a = actor(0, 0);
        a.kind = SETFIG;
        a.race = RACE_LASSO_KEEPER;
        let mut h = hero(10, 10, 5);
        start_dying(5, &mut a, &mut h);
        assert_eq!(h.kindness, 5);
    }

    #[test]
    fn the_dark_knight_speaks_and_costs_no_kindness() {
        let mut a = actor(0, 0);
        a.kind = SETFIG;
        a.race = RACE_DARK_KNIGHT;
        let mut h = hero(10, 10, 5);
        assert_eq!(
            start_dying(5, &mut a, &mut h),
            Some(Death { dark_knight: true })
        );
        assert_eq!(h.kindness, 5);
    }

    #[test]
    fn the_heros_own_death_costs_luck_not_bravery() {
        let (mut a, mut h) = (actor(0, 0), hero(10, 20, 10));
        start_dying(0, &mut a, &mut h);
        assert_eq!((h.brave, h.luck), (10, 15));
    }
}
