//! Actor behaviour helpers (from csrc/fmain2.c): hits, collision against other actors, distances,
//! enemy tactics, and a few story events.
//!
//! Actors live in `actor_list`: slot 0 is the hero, slot 1 the raft (or other carrier), slots 3 and
//! up the monsters, and people and objects after them. Positions are in world pixels. Directions
//! are numbered 0-7 clockwise from north-west (0 NW, 1 N, 2 NE, 3 E, 4 SE, 5 S, 6 SW, 7 W).

use crate::amiga::{RastPort, TextFont};
use crate::game::actors::encounters::{check_death, load_carrier};
use crate::game::actors::steering::steer_actor;
use crate::game::data::globals::{princesses_rescued, story_font};
use crate::game::data::object_tables::ob_list8;
use crate::game::data::random::{bitrand, fta_rand, rand8, rand64, rand256};
use crate::game::data::state::{the_actors, the_encounter, the_hero, the_screen, the_world};
use crate::game::data::tables::extent_list;
use crate::game::data::types::{Extent, Shape, WorldObject};
use crate::game::files::loading::play_effect;
use crate::game::render::screen_fx::{map_message, message_off};
use crate::game::ui::messages::{name, print, print_cont};
use crate::game::ui::text::{draw_placard_border, placard_text, print_number, show_event, speak};
use crate::game::world::map::{probe_collision, step_x, step_y, terrain_at};
use crate::game::world::places::transfer_hero;
use std::ffi::c_char;

unsafe extern "C" {
    fn SetFont(current_rastport: *mut RastPort, font: *mut TextFont);
    fn SetAPen(current_rastport: *mut RastPort, pen: u32);
    fn RectFill(current_rastport: *mut RastPort, x0: i32, y0: i32, x1: i32, y1: i32);
    fn Delay(ticks: i32);
}

// Actor kinds (csrc/ftale.h `enum sequences`, used for the `type` field).
const OBJECTS: i8 = 1;
const ENEMY: i8 = 2;
const SETFIG: i8 = 4;
/// Rafts and other carriers do not block movement.
const RAFT_TYPE: i8 = 5;
const DRAGON: i8 = 6;

// Actor states.
const WALKING: i8 = 12;
const DEAD: i8 = 15;
const SHOOT1: i8 = 24;

// Goals.
const ATTACK2: i8 = 2;
const FLEE: i8 = 5;

// Tactics.
const PURSUE: i32 = 1;
const FOLLOW: i32 = 2;
const BUMBLE_SEEK: i32 = 3;
const RANDOM: i32 = 4;
const BACKUP: i32 = 5;
const EVADE: i32 = 6;
const SHOOT: i32 = 8;
const EGG_SEEK: i32 = 10;

// Races with special rules when hit.
const RACE_WRAITH: u8 = 2;
const RACE_SPIRIT: u8 = 9;
const RACE_GHOST_KEEPER: u8 = 0x89;
const RACE_UNHITTABLE: [u8; 2] = [0x8a, 0x8b];
/// The weapon level needed to hurt spirits, and the item that also does.
const MIN_HURT_WEAPON: i8 = 4;
const SPIRIT_HURT_ITEM: usize = 7;
const MESSAGE_NEEDS_WEAPON: i32 = 58;

/// How far hit actors are pushed back.
const KNOCKBACK: i16 = 2;
const WORLD_DEAD_CHECK: i32 = 5;

// Sound effects and their speeds.
const EFFECT_ARROW_HIT: i16 = 2;
const EFFECT_ROCK_HIT: i16 = 5;
const EFFECT_HERO_HURT: i16 = 0;
const EFFECT_FOE_HURT: i16 = 3;

/// Message count shown after a fight only while the area is this peaceful (`extent_type` below).
const PEACEFUL_EXTENT: u16 = 50;
const BRAVE_LOW_VITALITY: i16 = 5;

/// Collision box around an actor.
const BLOCK_HALF_WIDTH: i32 = 11;
const BLOCK_HALF_HEIGHT: i32 = 9;
/// Value `blocked_at` returns when another actor is in the way.
const BLOCKED_BY_ACTOR: i32 = 16;
/// Terrain types that never stop the hero.
const SOFT_TERRAIN: [i32; 2] = [8, 9];
const HERO: usize = 0;
const RAFT_SLOT: usize = 1;

/// Set when turtle eggs are in view; the slot of the eggs' actor (see `report_battle_result`).
pub static mut turtle_eggs: i8 = 0;

/// Actor `i`. A raw pointer, because the functions called while it is in use change actors too.
fn actor(i: i32) -> *mut Shape {
    unsafe {
        (&raw mut the_actors.list)
            .cast::<Shape>()
            .wrapping_add(i as usize)
    }
}

/// A hit from actor `i` on actor `j`: `wt` damage, then the target is pushed back in direction `fc`.
/// `i` is -1 or -2 for hits by missiles (which have different sounds).
///
/// Special cases, as in the original: creatures of race 9 and the witch (race 0x89, a person of
/// kind 9) cannot be hurt by weak weapons (weapon below 4); the witch can be hurt anyway if the hero
/// holds the item in inventory slot 7 (the sun stone). Races 0x8a and 0x8b (the spectre and the
/// ghost brothers) are never hurt. Dragons and people are not pushed back.
pub fn hit_actor(i: i32, j: i32, fc: i32, wt: i16) {
    unsafe {
        let target = actor(j);
        let hero_weapon = (*actor(HERO as i32)).weapon;
        let has_spirit_item = *the_hero.stuff.add(SPIRIT_HURT_ITEM) != 0;
        if too_weak_to_hurt((*target).race, hero_weapon, has_spirit_item) {
            speak(MESSAGE_NEEDS_WEAPON);
            return;
        }
        if RACE_UNHITTABLE.contains(&(*target).race) {
            return;
        }
        (*target).vitality = vitality_after_hit((*target).vitality, wt);
        match hit_sound(i, j) {
            HitSound::Arrow => play_effect(EFFECT_ARROW_HIT, 500 + rand64()),
            HitSound::Rock => play_effect(EFFECT_ROCK_HIT, 3200 + bitrand(511)),
            HitSound::HeroHurt => play_effect(EFFECT_HERO_HURT, 800 + bitrand(511)),
            HitSound::FoeHurt => play_effect(EFFECT_FOE_HURT, 400 + rand256()),
        }
        // push back the foe
        if can_be_pushed((*target).kind)
            && step_figure(j as i16, fc as i16, KNOCKBACK) != 0
            && i >= 0
        {
            step_figure(i as i16, fc as i16, KNOCKBACK);
        }
        check_death(j, WORLD_DEAD_CHECK);
    }
}

/// Which sound a hit makes: `i` is the attacker (-1 an arrow, -2 a fireball), `j` the target.
#[derive(Debug, PartialEq, Eq)]
enum HitSound {
    Arrow,
    Rock,
    HeroHurt,
    FoeHurt,
}

fn hit_sound(attacker: i32, target: i32) -> HitSound {
    match (attacker, target) {
        (-1, _) => HitSound::Arrow,
        (-2, _) => HitSound::Rock,
        (_, 0) => HitSound::HeroHurt,
        _ => HitSound::FoeHurt,
    }
}

/// Dragons and people are not pushed back by a hit.
fn can_be_pushed(kind: i8) -> bool {
    kind != DRAGON && kind != SETFIG
}

/// What to say after a fight.
#[derive(Debug, PartialEq, Eq)]
enum BattleReport {
    /// The hero is dead, or nothing worth saying.
    Silent,
    Bravely,
    Foes {
        dead: i32,
        fled: i32,
    },
}

/// `dead` and `fled` count the enemies that fell and that ran away.
fn battle_report(hero_vitality: i16, dead: i32, fled: i32, extent_type: u16) -> BattleReport {
    if hero_vitality < 1 {
        BattleReport::Silent
    } else if hero_vitality < BRAVE_LOW_VITALITY && dead != 0 {
        BattleReport::Bravely
    } else if extent_type < PEACEFUL_EXTENT {
        BattleReport::Foes { dead, fled }
    } else {
        BattleReport::Silent
    }
}

/// Spirits can only be hurt by a real weapon (`MIN_HURT_WEAPON` or better); the ghost keeper too,
/// unless the hero carries the item that makes him vulnerable.
fn too_weak_to_hurt(race: u8, hero_weapon: i8, has_spirit_item: bool) -> bool {
    hero_weapon < MIN_HURT_WEAPON
        && (race == RACE_SPIRIT || (race == RACE_GHOST_KEEPER && !has_spirit_item))
}

/// A hit of strength `wt` takes that much vitality, down to a minimum of 0.
fn vitality_after_hit(vitality: i16, wt: i16) -> i16 {
    (vitality - wt).max(0)
}

/// After a fight: reports how many foes were defeated or fled, or "Bravely done!" if the hero is
/// nearly dead. Then, if turtle eggs are around, the turtle story event is checked.
pub fn report_battle_result() {
    unsafe {
        let (mut dead, mut fled) = (0, 0);
        for i in 3..the_actors.active as i32 {
            let a = actor(i);
            if (*a).kind != ENEMY {
                continue;
            }
            if (*a).state == DEAD {
                dead += 1;
            } else if (*a).goal == FLEE {
                fled += 1;
            }
        }
        let vitality = (*actor(HERO as i32)).vitality;
        match battle_report(vitality, dead, fled, the_world.extent_type) {
            BattleReport::Silent => {}
            BattleReport::Bravely => print(c"Bravely done!".as_ptr()),
            BattleReport::Foes { dead, fled } => {
                if dead != 0 {
                    print(c"".as_ptr());
                    print_number(dead, 1);
                    print_cont(c"foes were defeated in battle.".as_ptr());
                }
                if fled != 0 {
                    print(c"".as_ptr());
                    print_number(fled, 1);
                    print_cont(c"foes fled in retreat.".as_ptr());
                }
            }
        }
        // check for presence of turtle eggs
        if turtle_eggs != 0 {
            summon_turtle();
        }
    }
}

/// What blocks actor `i` at (x, y): a terrain type, 16 for another actor, or 0 if clear.
///
/// Terrain is checked first (wraiths float, so they ignore it; the hero ignores terrain types 8
/// and 9). Then every other actor is checked: it blocks if it is within 10 pixels sideways and
/// 8 pixels up or down.
/// Dead actors and the raft can be walked over.
pub fn blocked_at(x: i16, y: i16, i: i16) -> i32 {
    unsafe {
        let me = actor(i as i32);
        if (*me).kind != ENEMY || (*me).race != RACE_WRAITH {
            let mut terrain = probe_collision(x as i32, y as i32);
            if i == 0 && SOFT_TERRAIN.contains(&terrain) {
                terrain = 0;
            }
            if terrain != 0 {
                return terrain;
            }
        }
        for j in 0..the_actors.active as i32 {
            let other = actor(j);
            // dead actors and rafts can be walked over
            if i as i32 != j
                && j != RAFT_SLOT as i32
                && (*other).kind != RAFT_TYPE
                && (*other).state != DEAD
            {
                let dx = x as i32 - (*other).abs_x as i32;
                let dy = y as i32 - (*other).abs_y as i32;
                if dx < BLOCK_HALF_WIDTH
                    && dx > -BLOCK_HALF_WIDTH
                    && dy < BLOCK_HALF_HEIGHT
                    && dy > -BLOCK_HALF_HEIGHT
                {
                    return BLOCKED_BY_ACTOR;
                }
            }
        }
        0
    }
}

/// Finds the nearest actor to the hero closer than `dist`, setting `nearest_object`; returns the
/// distance found. With `constraint` set, objects are ignored too.
pub fn find_nearest_figure(constraint: c_char, mut dist: i16) -> i32 {
    /// Index of the "still" object type that always counts.
    const ALWAYS_COUNTED_OBJECT: i8 = 0x1d;
    unsafe {
        the_actors.nearest_object = 0;
        for i in 1..the_actors.drawn as i32 {
            let a = actor(i);
            if (*a).kind == OBJECTS && (constraint != 0 || (*a).index == ALWAYS_COUNTED_OBJECT) {
                continue;
            }
            let d = actor_distance(i, 0);
            if d < dist as i32 {
                the_actors.nearest_object = i as i16;
                dist = d as i16;
            }
        }
        dist as i32
    }
}

/// Distance between two actors. The game avoids a square root: if one axis is more than twice
/// the other it is used alone, otherwise (x + y) * 5 / 7 approximates the diagonal.
pub fn actor_distance(a: i32, b: i32) -> i32 {
    let (a, b) = unsafe { (&*actor(a), &*actor(b)) };
    let x = (a.abs_x as i32 - b.abs_x as i32).abs();
    let y = (a.abs_y as i32 - b.abs_y as i32).abs();
    if x > y + y {
        x
    } else if y > x + x {
        y
    } else {
        (x + y) * 5 / 7
    }
}

/// Moves an actor `dist` in direction `dir` if nothing blocks it. Returns TRUE if it moved.
pub fn step_figure(fig: i16, dir: i16, dist: i16) -> i32 {
    unsafe {
        let a = actor(fig as i32);
        let xtest = step_x((*a).abs_x as i32, dir as i32, dist as i32) as u16;
        let ytest = step_y((*a).abs_y as i32, dir as i32, dist as i32) as u16;
        if blocked_at(xtest as i16, ytest as i16, fig) != 0 {
            return 0;
        }
        (*a).abs_x = xtest;
        (*a).abs_y = ytest;
        1
    }
}

// How an enemy shooting at the hero lines up.
const SHOOT_CLOSE: i16 = 8;
const SHOOT_DIAGONAL_BELOW: i16 = 5;
const SHOOT_DIAGONAL_ABOVE: i16 = 7;
// `steer_actor` modes.
const COURSE_TOWARD: i32 = 0;
const COURSE_AWAY: i32 = 3;
const COURSE_STRAIGHT: i32 = 4;
const COURSE_TURN_ONLY: i32 = 5;
const COURSE_WOBBLE_CLOSE: i32 = 2;
/// The followers aim this far below the one they follow.
const FOLLOW_OFFSET: i32 = 20;
/// Where snakes go to find the eggs.
const EGG_X: i32 = 23087;
const EGG_Y: i32 = 5667;

/// Runs one step of enemy `i`'s tactic. Most tactics only act on some ticks (1 in 8, or 1 in 4
/// for clever enemies), which makes them look less robotic.
///
/// - PURSUE: head toward the hero.  - SHOOT: line up with the hero (same row, column or
///   diagonal) and aim, else approach.  - RANDOM: wander.  - BUMBLE_SEEK: head toward the hero
///   without smart steering.  - BACKUP: head away from the hero.  - FOLLOW: stay near the leader.
/// - EVADE: keep away from another enemy.  - EGG_SEEK: go to the turtle eggs.
pub fn run_tactic(i: i32, tactic: i32) {
    unsafe {
        let mut acts_now = fta_rand() & 7 == 0;
        let an = actor(i);
        (*an).tactic = tactic as i8;
        if (*an).goal == ATTACK2 {
            acts_now = fta_rand() & 3 == 0;
        }
        let (hx, hy) = (the_hero.x as i32, the_hero.y as i32);
        match tactic {
            PURSUE => steer_if(acts_now, i, hx, hy, COURSE_TOWARD),
            SHOOT => shoot_or_approach(i, an, hx, hy),
            RANDOM => wander(an, acts_now),
            BUMBLE_SEEK => steer_if(acts_now, i, hx, hy, COURSE_STRAIGHT),
            BACKUP => steer_if(acts_now, i, hx, hy, COURSE_AWAY),
            FOLLOW => follow_leader(i, an, acts_now),
            EVADE => evade_neighbour(i, acts_now),
            EGG_SEEK => {
                steer_if(acts_now, i, EGG_X, EGG_Y, COURSE_TOWARD);
                (*an).state = WALKING;
            }
            _ => {}
        }
    }
}

/// Steers actor `i` toward a point (or however `mode` says) if this is one of its active ticks.
fn steer_if(acts_now: bool, i: i32, x: i32, y: i32, mode: i32) {
    if acts_now {
        steer_actor(i, x, y, mode);
    }
}

/// An archer lines up with the hero (same row, column or diagonal) and aims, otherwise approaches.
unsafe fn shoot_or_approach(i: i32, an: *mut Shape, hx: i32, hy: i32) {
    unsafe {
        let hero = actor(HERO as i32);
        let xd = ((*an).abs_x as i32 - (*hero).abs_x as i32) as i16;
        let yd = ((*an).abs_y as i32 - (*hero).abs_y as i32) as i16;
        let (xd, yd) = (xd.wrapping_abs(), yd.wrapping_abs());
        let lined_up = xd < SHOOT_CLOSE
            || yd < SHOOT_CLOSE
            || (xd as i32 > yd as i32 - SHOOT_DIAGONAL_BELOW as i32
                && (xd as i32) < yd as i32 + SHOOT_DIAGONAL_ABOVE as i32);
        if fta_rand() & 1 != 0 && lined_up {
            steer_actor(i, hx, hy, COURSE_TURN_ONLY);
            if (*an).state < SHOOT1 {
                (*an).state = SHOOT1;
            }
        } else {
            steer_actor(i, hx, hy, COURSE_TOWARD);
        }
    }
}

/// Wander: now and then turn a random way, and keep walking.
unsafe fn wander(an: *mut Shape, acts_now: bool) {
    unsafe {
        if acts_now {
            (*an).facing = (fta_rand() & 7) as i8;
        }
        (*an).state = WALKING;
    }
}

/// Stay near the leader (an enemy cannot follow itself, so it wanders instead).
unsafe fn follow_leader(i: i32, an: *mut Shape, acts_now: bool) {
    unsafe {
        let leader_slot = the_actors.leader as i32;
        if i == leader_slot {
            (*an).tactic = RANDOM as i8; // can't follow self
        }
        if acts_now {
            steer_toward_actor(i, leader_slot, COURSE_TOWARD);
        }
    }
}

/// Keep away from another enemy (the one before the last, or the one at twice this slot number).
unsafe fn evade_neighbour(i: i32, acts_now: bool) {
    unsafe {
        let other = if i == the_actors.active as i32 {
            i - 1
        } else {
            i + i
        };
        if acts_now {
            steer_toward_actor(i, other, COURSE_WOBBLE_CLOSE);
        }
    }
}

/// Steers actor `i` toward (a little below) actor `target`.
unsafe fn steer_toward_actor(i: i32, target: i32, mode: i32) {
    unsafe {
        let target = actor(target);
        steer_actor(
            i,
            (*target).abs_x as i32,
            (*target).abs_y as i32 + FOLLOW_OFFSET,
            mode,
        );
    }
}

/// Places the turtle: tries up to 25 random spots around the hero for one on water (terrain 5),
/// then moves the turtle extent there and loads the turtle as the carrier. Called when turtle
/// eggs are in view. Leaves the loop count in C's global `i`, as the original did.
pub fn summon_turtle() {
    const TRIES: i32 = 25;
    const WATER: i32 = 5;
    const TURTLE_EXTENT: i16 = 1;
    const TURTLE_CARRIER: i16 = 5;
    unsafe {
        let mut tries_used = 0;
        while tries_used < TRIES {
            pick_encounter_spot();
            if terrain_at(the_encounter.x as i32, the_encounter.y as i32) == WATER {
                break;
            }
            tries_used += 1;
        }
        if tries_used == TRIES {
            return;
        }
        center_extent_on(
            TURTLE_EXTENT,
            the_encounter.x as i16,
            the_encounter.y as i16,
        );
        load_carrier(TURTLE_CARRIER);
    }
}

/// Centres extent `e` on a point (it covers 500 by 400 pixels around it).
pub fn center_extent_on(e: i16, x: i16, y: i16) {
    const HALF_WIDTH: i32 = 250;
    const HALF_HEIGHT: i32 = 200;
    unsafe {
        let extent = &mut *(&raw mut extent_list).cast::<Extent>().add(e as usize);
        extent.x1 = (x as i32 - HALF_WIDTH) as u16;
        extent.y1 = (y as i32 - HALF_HEIGHT) as u16;
        extent.x2 = (x as i32 + HALF_WIDTH) as u16;
        extent.y2 = (y as i32 + HALF_HEIGHT) as u16;
    }
}

// The rescue scene.
const PLACARD_PEN: u32 = 24;
const RESCUE_PAUSE: i32 = 380;
const CLEAR_PAUSE: i32 = 10;
const PLACARD_AREA: (i32, i32, i32, i32) = (13, 13, 271, 107);
const PRINCESS_HOME: (u16, u16) = (5511, 33780);
const PRINCESS_EXTENT: (i16, i16) = (22205, 21231);
/// Entries of the interior object list (`ob_list8`) that change when the princess is rescued.
const PRINCESS_OBJECT: usize = 2;
const PRINCESS_OBJECT_ID: i8 = 4;
const CAPTIVE_OBJECT: usize = 9;
const PRINCESS_STUFF: usize = 28;
/// Items 16 to 21 (the keys) get this many extra.
const KEY_ITEMS: std::ops::Range<usize> = 16..22;
const KEYS_REWARD: u8 = 3;
const REWARD_MESSAGE: i32 = 18;
const REWARD_GOLD: i16 = 100;

/// The scene when a princess is rescued, and her reward.
///
/// Shows the story pages, moves the hero to the princess's home, repositions the extent for the
/// next princess, changes the princess object, gives 100 gold and three of each key, and bumps
/// the `princess` count (each rescue shows different story pages).
pub fn rescue_princess() {
    unsafe {
        map_message();
        SetFont(the_screen.current_rastport, story_font);
        let line = princesses_rescued as i32 * 3;
        placard_text(8 + line);
        name();
        placard_text(9 + line);
        name();
        placard_text(10 + line);
        draw_placard_border();
        Delay(RESCUE_PAUSE);
        SetAPen(the_screen.current_rastport, 0);
        let (x0, y0, x1, y1) = PLACARD_AREA;
        RectFill(the_screen.current_rastport, x0, y0, x1, y1);
        Delay(CLEAR_PAUSE);
        SetAPen(the_screen.current_rastport, PLACARD_PEN);
        placard_text(17);
        name();
        placard_text(18);
        Delay(RESCUE_PAUSE);
        message_off();

        princesses_rescued += 1;
        transfer_hero(PRINCESS_HOME.0, PRINCESS_HOME.1, 0); // do princess transfer
        center_extent_on(0, PRINCESS_EXTENT.0, PRINCESS_EXTENT.1);
        let objects = (&raw mut ob_list8).cast::<WorldObject>();
        (*objects.add(PRINCESS_OBJECT)).ob_id = PRINCESS_OBJECT_ID;
        *the_hero.stuff.add(PRINCESS_STUFF) = 1;
        speak(REWARD_MESSAGE);
        the_hero.wealth = the_hero.wealth.wrapping_add(REWARD_GOLD);
        (*objects.add(CAPTIVE_OBJECT)).ob_stat = 0;
        for item in KEY_ITEMS {
            *the_hero.stuff.add(item) = (*the_hero.stuff.add(item)).wrapping_add(KEYS_REWARD);
        }
    }
}

/// Eats: lowers hunger by `amt`. If that would go below zero the hero is simply full
/// (message 13); otherwise "Yum!".
pub fn eat_food(amt: i32) {
    const HUNGER_FULL_MESSAGE: i32 = 13;
    unsafe {
        the_hero.hunger = (the_hero.hunger as i32 - amt) as i16;
        if the_hero.hunger < 0 {
            the_hero.hunger = 0;
            show_event(HUNGER_FULL_MESSAGE);
        } else {
            print(c"Yum!".as_ptr());
        }
    }
}

// An encounter appears this far away: 150 plus up to 63.
const ENCOUNTER_MIN_DISTANCE: i32 = 150;

/// Picks where a new encounter appears: a random direction and distance from the hero
/// (150 to 213 pixels).
pub fn pick_encounter_spot() {
    unsafe {
        let direction = rand8();
        let distance = ENCOUNTER_MIN_DISTANCE + rand64();
        the_encounter.x = step_x(the_hero.x as i32, direction, distance) as u16;
        the_encounter.y = step_y(the_hero.y as i32, direction, distance) as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spirits_need_a_real_weapon() {
        assert!(too_weak_to_hurt(RACE_SPIRIT, MIN_HURT_WEAPON - 1, true));
        assert!(!too_weak_to_hurt(RACE_SPIRIT, MIN_HURT_WEAPON, false));
    }

    #[test]
    fn the_ghost_keeper_is_hurt_by_weak_weapons_only_with_the_item() {
        assert!(too_weak_to_hurt(RACE_GHOST_KEEPER, 0, false));
        assert!(!too_weak_to_hurt(RACE_GHOST_KEEPER, 0, true));
    }

    #[test]
    fn ordinary_foes_are_always_hurt() {
        assert!(!too_weak_to_hurt(0, 0, false));
    }

    #[test]
    fn a_hit_never_takes_vitality_below_zero() {
        assert_eq!(vitality_after_hit(10, 4), 6);
        assert_eq!(vitality_after_hit(3, 9), 0);
    }

    #[test]
    fn struct_sizes() {
        assert_eq!(size_of::<Extent>(), 12);
    }

    #[test]
    fn hit_sounds_by_who_hit_whom() {
        assert_eq!(hit_sound(-1, 0), HitSound::Arrow);
        assert_eq!(hit_sound(-2, 5), HitSound::Rock);
        assert_eq!(hit_sound(3, 0), HitSound::HeroHurt);
        assert_eq!(hit_sound(0, 3), HitSound::FoeHurt);
    }

    #[test]
    fn dragons_and_people_are_not_pushed_back() {
        assert!(!can_be_pushed(DRAGON));
        assert!(!can_be_pushed(SETFIG));
        assert!(can_be_pushed(2));
    }

    #[test]
    fn what_is_said_after_a_fight() {
        let calm = PEACEFUL_EXTENT - 1;
        assert_eq!(battle_report(0, 3, 0, calm), BattleReport::Silent);
        assert_eq!(
            battle_report(BRAVE_LOW_VITALITY - 1, 1, 0, calm),
            BattleReport::Bravely
        );
        // nearly dead but nobody died: the normal report
        assert_eq!(
            battle_report(BRAVE_LOW_VITALITY - 1, 0, 2, calm),
            BattleReport::Foes { dead: 0, fled: 2 }
        );
        assert_eq!(
            battle_report(50, 2, 1, calm),
            BattleReport::Foes { dead: 2, fled: 1 }
        );
        // peaceful places say nothing
        assert_eq!(
            battle_report(50, 2, 1, PEACEFUL_EXTENT),
            BattleReport::Silent
        );
    }
}
