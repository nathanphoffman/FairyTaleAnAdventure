//! Fighting each frame (from the main loop in csrc/fmain.c): swings and bites, arrows and
//! fireballs in flight, and putting the visible missiles in the actor list so they get drawn.

use super::actors::hit_actor;
use super::asm_map::{step_x, step_y, terrain_at};
use super::loading::play_effect;
use super::random::{bitrand, rand4, rand8, rand256};
use super::state::{the_actors, the_hero, the_world};
use super::types::Missile;

// Actor kinds and states (csrc/ftale.h).
const OBJECTS: i8 = 1;
const CARRIER: i8 = 5;
const WALKING: i8 = 12;
const DEAD: i8 = 15;
const RAFT: usize = 1;
/// Slot count of the actor list, and of the missile list.
const ACTOR_SLOTS: i16 = 20;
const MISSILES: usize = 6;

// ---- melee_hits ----
/// A weapon with this bit is a bow (it is not swung).
const BOW_BIT: i16 = 4;
/// Touch attacks (monsters' weapon codes 8 and up) are as strong as this weapon.
const TOUCH_WEAPON: i16 = 5;
const BRAVE_PER_REACH: i16 = 20;
const HERO_BASE_REACH: i16 = 5;
const ENEMY_BASE_REACH: i16 = 2;
const MAX_REACH: i16 = 14;
const REACH_LIMIT: i16 = 15;
/// The tip of the weapon is placed this far (plus a little noise) in front of the actor.
const TIP_NOISE: i32 = 3;
const SWING_SOUND: i16 = 1;
const SWING_SOUND_PITCH: i32 = 150;
/// How much further than `reach` a swing still makes the whoosh sound.
const WHOOSH_MARGIN: i16 = 2;

/// Distance between two points as the larger of the x and y differences.
fn chebyshev(dx: i16, dy: i16) -> i16 {
    dx.wrapping_abs().max(dy.wrapping_abs())
}

/// Everything not walking swings its weapon (or bites) at whatever is within reach of the tip:
/// `hit_actor` if it lands, a whoosh if it nearly does. Only the hero acts while time is frozen.
pub fn resolve_melee() {
    unsafe {
        let mut i = 0;
        while i < the_actors.active as usize {
            if i > 0 && the_world.freeze_timer != 0 {
                break;
            }
            let attacker = &raw const the_actors.list[i];
            if i != RAFT && (*attacker).state < WALKING {
                swing(i);
            }
            i += 1;
        }
    }
}

/// The weapon an actor swings in melee, or `None` for a bow (archers shoot instead). Anything
/// past the hand weapons (8 and up: bites, claws) counts as a touch.
fn melee_weapon(weapon: i16) -> Option<i16> {
    if weapon & BOW_BIT != 0 {
        None
    } else if weapon >= 8 {
        Some(TOUCH_WEAPON)
    } else {
        Some(weapon)
    }
}

/// How close the weapon tip has to get to hit. Reach beyond the maximum drops to the limit.
fn limited_reach(reach: i16) -> i16 {
    if reach > MAX_REACH {
        REACH_LIMIT
    } else {
        reach
    }
}

/// The hero's reach grows with bravery.
fn hero_reach(brave: i16) -> i16 {
    limited_reach(brave / BRAVE_PER_REACH + HERO_BASE_REACH)
}

/// An enemy's reach is a fixed base plus a random `bonus`.
fn enemy_reach(bonus: i16) -> i16 {
    limited_reach(ENEMY_BASE_REACH + bonus)
}

/// One actor's swing.
unsafe fn swing(i: usize) {
    unsafe {
        let attacker = &raw const the_actors.list[i];
        let fc = (*attacker).facing as i32;
        let Some(base) = melee_weapon((*attacker).weapon as i16) else {
            return;
        };
        let wt = base + bitrand(2) as i16;
        let reach = (wt + wt) as i32;
        let xs = (step_x((*attacker).abs_x as i32, fc, reach) + rand8() - TIP_NOISE) as i16;
        let ys = (step_y((*attacker).abs_y as i32, fc, reach) + rand8() - TIP_NOISE) as i16;
        let bv = if i == 0 {
            hero_reach(the_hero.brave)
        } else {
            enemy_reach(rand4() as i16)
        };
        let mut j = 0;
        while j < the_actors.active as usize {
            let target = &raw const the_actors.list[j];
            if j == RAFT || j == i || (*target).state == DEAD || (*attacker).kind == CARRIER {
                j += 1;
                continue;
            }
            let xd = ((*target).abs_x as i32 - xs as i32) as i16;
            let yd = ((*target).abs_y as i32 - ys as i32) as i16;
            let dist = chebyshev(xd, yd);
            match swing_result(
                i == 0,
                dist,
                bv,
                the_world.freeze_timer != 0,
                wt,
                the_hero.brave,
                rand256,
            ) {
                SwingResult::Hit => {
                    hit_actor(i as i32, j as i32, fc, wt);
                    break;
                }
                SwingResult::Whoosh => play_effect(SWING_SOUND, SWING_SOUND_PITCH + rand256()),
                SwingResult::Miss => {}
            }
            j += 1;
        }
    }
}

/// How a swing at one target turned out.
#[derive(Debug, PartialEq, Eq)]
enum SwingResult {
    Hit,
    /// Near enough to make the whoosh sound, but no hit.
    Whoosh,
    Miss,
}

/// The tip of the weapon is `dist` from the target and the reach is `reach`. The hero always hits
/// within reach; an enemy also has to get past the hero's bravery (`dodge_roll` is a 0-255 draw,
/// taken before anything else is checked, only for enemies). Nothing hits while time is frozen,
/// and touch attacks make no sound.
#[allow(clippy::too_many_arguments)] // the whole situation of one swing, from plain values
fn swing_result(
    by_hero: bool,
    dist: i16,
    reach: i16,
    frozen: bool,
    strength: i16,
    hero_bravery: i16,
    dodge_roll: impl FnOnce() -> i32,
) -> SwingResult {
    if (by_hero || dodge_roll() > hero_bravery as i32) && dist < reach && !frozen {
        SwingResult::Hit
    } else if dist < reach + WHOOSH_MARGIN && strength != TOUCH_WEAPON {
        SwingResult::Whoosh
    } else {
        SwingResult::Miss
    }
}

/// Whether a missile at `dist` from a target hits it: within `range`, and for the hero (only for the
/// first missile slot, as in the original) a roll above the hero's bravery is needed. The roll is
/// only drawn when it is needed.
fn missile_connects(
    slot: usize,
    dist: i16,
    range: i16,
    dodge: i16,
    roll: impl FnOnce() -> i32,
) -> bool {
    (slot != 0 || roll() > dodge as i32) && dist < range
}

// ---- move_missiles ----
const MISSILE_NONE: i8 = 0;
const MISSILE_ROCK: i8 = 2;
const MISSILE_SPENT: i8 = 3;
/// A missile gives up after this many frames in flight.
const MAX_FLIGHT: i8 = 40;
/// Terrain a missile stops at: solid wall and the map edge.
const WALL_TERRAIN: [i32; 2] = [1, 15];
/// How close a missile has to be to hit (arrows and rocks, fireballs).
const ARROW_HIT_RANGE: i16 = 6;
const FIREBALL_HIT_RANGE: i16 = 9;
/// Other actors are treated as having this much bravery when it comes to dodging.
const OTHER_DODGE: i16 = 20;
const DODGE_RANGE: i32 = 512;
const MISSILE_DAMAGE_BASE: i32 = 4;
/// `hit_actor` attacker codes for an arrow and for a fireball.
const BY_ARROW: i32 = -1;
const BY_FIREBALL: i32 = -2;

/// Moves the six arrows and fireballs; they hit actors in the way and stop at walls. Nothing
/// moves while time is frozen.
pub fn advance_missiles() {
    unsafe {
        if the_world.freeze_timer != 0 {
            return;
        }
        for i in 0..MISSILES {
            let ms = &raw mut the_actors.missiles[i];
            fly(i, ms);
        }
    }
}

/// One missile's flight for this frame.
unsafe fn fly(i: usize, ms: *mut Missile) {
    unsafe {
        let mut s = (*ms).speed as i16 * 2;
        if missile_expired(ms, s) {
            (*ms).missile_type = MISSILE_NONE;
            return;
        }

        let xs = (*ms).abs_x as i16;
        let ys = (*ms).abs_y as i16;
        if WALL_TERRAIN.contains(&terrain_at(xs as i32, ys as i32)) {
            (*ms).missile_type = MISSILE_NONE;
            s = 0;
        }
        let fc = (*ms).direction as i32;
        hit_first_actor_in_path(i, ms, xs, ys);
        (*ms).abs_x = step_x((*ms).abs_x as i32, fc, s as i32) as u16;
        (*ms).abs_y = step_y((*ms).abs_y as i32, fc, s as i32) as u16;
    }
}

/// Whether the missile is unused, spent, standing still (`s` is its doubled speed) or has been in
/// the air too long. Counts one more tick of flight when it has not expired.
unsafe fn missile_expired(ms: *mut Missile, s: i16) -> bool {
    unsafe {
        let flight = (*ms).time_of_flight;
        if (*ms).missile_type == MISSILE_NONE || (*ms).missile_type == MISSILE_SPENT || s == 0 {
            return true;
        }
        (*ms).time_of_flight = flight.wrapping_add(1);
        flight > MAX_FLIGHT
    }
}

/// Hits the first actor close enough to the missile at (`xs`, `ys`) and stops the missile.
/// `i` is the missile's own index; the hero (actor 0) gets a dodge chance from `brave`, others a
/// fixed one.
unsafe fn hit_first_actor_in_path(i: usize, ms: *mut Missile, xs: i16, ys: i16) {
    unsafe {
        let fc = (*ms).direction as i32;
        let is_rock = (*ms).missile_type == MISSILE_ROCK;
        let range = if is_rock {
            FIREBALL_HIT_RANGE
        } else {
            ARROW_HIT_RANGE
        };
        let mut j = 0;
        while j < actor_count() {
            let dodge = if j == 0 { the_hero.brave } else { OTHER_DODGE };
            let target = &raw const the_actors.list[j];
            let untouchable = j == RAFT
                || (*ms).archer as i16 == j as i16
                || (*target).state == DEAD
                || (*target).kind == CARRIER;
            j += 1;
            if untouchable {
                continue;
            }
            let xd = ((*target).abs_x as i32 - xs as i32) as i16;
            let yd = ((*target).abs_y as i32 - ys as i32) as i16;
            // the original tests the missile's own index here, not the target's
            if missile_connects(i, chebyshev(xd, yd), range, dodge, || bitrand(DODGE_RANGE)) {
                let by = if is_rock { BY_FIREBALL } else { BY_ARROW };
                hit_actor(
                    by,
                    (j - 1) as i32,
                    fc,
                    (rand8() + MISSILE_DAMAGE_BASE) as i16,
                );
                (*ms).speed = 0;
                if is_rock {
                    (*ms).missile_type = MISSILE_SPENT;
                }
                return;
            }
        }
    }
}

/// The current number of actors (the loop bound is read again each time, as in the original).
pub fn actor_count() -> usize {
    unsafe { the_actors.active as usize }
}

// ---- queue_missiles ----
/// A missile is added if its sprite would be inside this box on the screen.
const SCREEN_LEFT: i16 = -20;
const SCREEN_RIGHT: i16 = 340;
const SCREEN_TOP: i16 = -20;
const SCREEN_BOTTOM: i16 = 190;
/// The sprite's centre is this far from its map position.
const SPRITE_OFFSET: (i32, i32) = (9, 26);
const MISSILE_ROCK_IMAGE: i8 = 0x59;
const MISSILE_SPENT_IMAGE: i8 = 0x58;
const NO_RACE: u8 = 0xff;

/// Adds the visible missiles to the end of the actor list as objects so the drawing draws them.
/// Stops when the list is full.
pub fn queue_missile_sprites() {
    unsafe {
        for i in 0..MISSILES {
            let ms = &raw const the_actors.missiles[i];
            let t = (*ms).missile_type;
            if t == MISSILE_NONE {
                continue;
            }
            let xstart = ((*ms).abs_x as i32 - the_world.map_x as i32 - SPRITE_OFFSET.0) as i16;
            let ystart = ((*ms).abs_y as i32 - the_world.map_y as i32 - SPRITE_OFFSET.1) as i16;
            if xstart > SCREEN_LEFT
                && xstart < SCREEN_RIGHT
                && ystart > SCREEN_TOP
                && ystart < SCREEN_BOTTOM
            {
                let an = &raw mut the_actors.list[the_actors.drawn as usize];
                (*an).abs_x = (*ms).abs_x;
                (*an).abs_y = (*ms).abs_y;
                (*an).rel_x = xstart as u16;
                (*an).rel_y = ystart as u16;
                (*an).kind = OBJECTS;
                (*an).weapon = 0;
                (*an).index = (*ms).direction;
                (*an).race = NO_RACE;
                if t == MISSILE_SPENT {
                    (*an).index = MISSILE_SPENT_IMAGE;
                } else if t == MISSILE_ROCK {
                    (*an).index = (*an).index.wrapping_add(MISSILE_ROCK_IMAGE);
                }
                the_actors.drawn += 1;
                if the_actors.drawn >= ACTOR_SLOTS {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_the_larger_axis() {
        assert_eq!(chebyshev(3, -5), 5);
        assert_eq!(chebyshev(-7, 2), 7);
        assert_eq!(chebyshev(0, 0), 0);
    }

    #[test]
    fn bows_are_not_swung() {
        assert_eq!(melee_weapon(BOW_BIT), None);
        assert_eq!(melee_weapon(1), Some(1));
    }

    #[test]
    fn natural_weapons_all_count_as_a_touch() {
        assert_eq!(melee_weapon(8), Some(TOUCH_WEAPON));
        assert_eq!(melee_weapon(10), Some(TOUCH_WEAPON));
    }

    #[test]
    fn reach_is_limited() {
        assert_eq!(limited_reach(MAX_REACH), MAX_REACH);
        assert_eq!(limited_reach(MAX_REACH + 1), REACH_LIMIT);
    }

    #[test]
    fn braver_heroes_reach_further_up_to_the_limit() {
        assert!(hero_reach(100) >= hero_reach(0));
        assert_eq!(hero_reach(i16::MAX / 2), REACH_LIMIT);
    }

    fn missile(kind: i8, speed: i8, time_of_flight: i8) -> Missile {
        let mut ms: Missile = unsafe { std::mem::zeroed() };
        ms.missile_type = kind;
        ms.speed = speed;
        ms.time_of_flight = time_of_flight;
        ms
    }

    #[test]
    fn unused_spent_and_stopped_missiles_expire() {
        unsafe {
            assert!(missile_expired(&mut missile(MISSILE_NONE, 5, 0), 10));
            assert!(missile_expired(&mut missile(MISSILE_SPENT, 5, 0), 10));
            assert!(missile_expired(&mut missile(MISSILE_ROCK, 0, 0), 0));
        }
    }

    #[test]
    fn missile_counts_flight_time_until_it_runs_out() {
        unsafe {
            let mut ms = missile(MISSILE_ROCK, 5, 0);
            assert!(!missile_expired(&mut ms, 10));
            assert_eq!(ms.time_of_flight, 1);
            let mut old = missile(MISSILE_ROCK, 5, MAX_FLIGHT + 1);
            assert!(missile_expired(&mut old, 10));
        }
    }

    #[test]
    fn the_hero_always_hits_within_reach() {
        let never = || -> i32 { unreachable!("the hero does not roll") };
        assert_eq!(
            swing_result(true, 4, 5, false, 3, 100, never),
            SwingResult::Hit
        );
        assert_eq!(
            swing_result(true, 5, 5, false, 3, 100, never),
            SwingResult::Whoosh
        );
        assert_eq!(
            swing_result(true, 7, 5, false, 3, 100, never),
            SwingResult::Miss
        );
    }

    #[test]
    fn an_enemy_must_beat_the_heros_bravery() {
        assert_eq!(
            swing_result(false, 1, 5, false, 3, 100, || 101),
            SwingResult::Hit
        );
        assert_eq!(
            swing_result(false, 1, 5, false, 3, 100, || 100),
            SwingResult::Whoosh
        );
        assert_eq!(
            swing_result(false, 1, 5, false, 3, 100, || 0),
            SwingResult::Whoosh
        );
    }

    #[test]
    fn the_enemy_rolls_even_when_out_of_reach() {
        let mut drawn = 0;
        swing_result(false, 50, 5, false, 3, 100, || {
            drawn += 1;
            0
        });
        assert_eq!(drawn, 1);
    }

    #[test]
    fn nothing_hits_while_time_is_frozen_and_touches_are_silent() {
        assert_eq!(
            swing_result(true, 1, 5, true, 3, 0, || 0),
            SwingResult::Whoosh
        );
        assert_eq!(
            swing_result(true, 1, 5, true, TOUCH_WEAPON, 0, || 0),
            SwingResult::Miss
        );
        assert_eq!(
            swing_result(true, 5, 5, false, TOUCH_WEAPON, 0, || 0),
            SwingResult::Miss
        );
    }

    #[test]
    fn missiles_roll_only_for_the_first_slot() {
        let never = || -> i32 { unreachable!("only slot 0 rolls") };
        assert!(missile_connects(3, 5, 6, 100, never));
        assert!(!missile_connects(3, 6, 6, 100, never));
        assert!(missile_connects(0, 5, 6, 100, || 101));
        assert!(!missile_connects(0, 5, 6, 100, || 100));
    }
}
