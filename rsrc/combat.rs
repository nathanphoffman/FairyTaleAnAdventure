//! Fighting each frame (from the main loop in csrc/fmain.c): swings and bites, arrows and
//! fireballs in flight, and putting the visible missiles in the actor list so they get drawn.

use super::actors::hit_actor;
use super::asm_map::{step_x, step_y, terrain_at};
use super::globals::{
    active_actor_count, actor_list, brave, drawn_actor_count, freeze_timer, map_x, map_y,
    missile_slots,
};
use super::loading::play_effect;
use super::random::{bitrand, rand4, rand8, rand256};
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
        while i < active_actor_count as usize {
            if i > 0 && freeze_timer != 0 {
                break;
            }
            let attacker = &raw const actor_list[i];
            if i != RAFT && (*attacker).state < WALKING {
                swing(i);
            }
            i += 1;
        }
    }
}

/// One actor's swing.
unsafe fn swing(i: usize) {
    unsafe {
        let attacker = &raw const actor_list[i];
        let mut wt = (*attacker).weapon as i16;
        let fc = (*attacker).facing as i32;
        if wt & BOW_BIT != 0 {
            return;
        }
        if wt >= 8 {
            wt = TOUCH_WEAPON;
        }
        wt += bitrand(2) as i16;
        let reach = (wt + wt) as i32;
        let xs = (step_x((*attacker).abs_x as i32, fc, reach) + rand8() - TIP_NOISE) as i16;
        let ys = (step_y((*attacker).abs_y as i32, fc, reach) + rand8() - TIP_NOISE) as i16;
        let mut bv = if i == 0 {
            brave / BRAVE_PER_REACH + HERO_BASE_REACH
        } else {
            ENEMY_BASE_REACH + rand4() as i16
        };
        if bv > MAX_REACH {
            bv = REACH_LIMIT;
        }
        let mut j = 0;
        while j < active_actor_count as usize {
            let target = &raw const actor_list[j];
            if j == RAFT || j == i || (*target).state == DEAD || (*attacker).kind == CARRIER {
                j += 1;
                continue;
            }
            let xd = ((*target).abs_x as i32 - xs as i32) as i16;
            let yd = ((*target).abs_y as i32 - ys as i32) as i16;
            let dist = chebyshev(xd, yd);
            if (i == 0 || rand256() > brave as i32) && dist < bv && freeze_timer == 0 {
                hit_actor(i as i32, j as i32, fc, wt);
                break;
            } else if dist < bv + WHOOSH_MARGIN && wt != TOUCH_WEAPON {
                play_effect(SWING_SOUND, SWING_SOUND_PITCH + rand256());
            }
            j += 1;
        }
    }
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
        if freeze_timer != 0 {
            return;
        }
        for i in 0..MISSILES {
            let ms = &raw mut missile_slots[i];
            fly(i, ms);
        }
    }
}

/// One missile's flight for this frame.
unsafe fn fly(i: usize, ms: *mut Missile) {
    unsafe {
        let mut s = (*ms).speed as i16 * 2;
        let flight = (*ms).time_of_flight;
        let expired = if (*ms).missile_type == MISSILE_NONE
            || (*ms).missile_type == MISSILE_SPENT
            || s == 0
        {
            true
        } else {
            (*ms).time_of_flight = flight.wrapping_add(1);
            flight > MAX_FLIGHT
        };
        if expired {
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
        let range = if (*ms).missile_type == MISSILE_ROCK {
            FIREBALL_HIT_RANGE
        } else {
            ARROW_HIT_RANGE
        };

        let mut j = 0;
        while j < actor_count() {
            let bv = if j == 0 { brave } else { OTHER_DODGE };
            let target = &raw const actor_list[j];
            if j == RAFT
                || (*ms).archer as i16 == j as i16
                || (*target).state == DEAD
                || (*target).kind == CARRIER
            {
                j += 1;
                continue;
            }
            let xd = ((*target).abs_x as i32 - xs as i32) as i16;
            let yd = ((*target).abs_y as i32 - ys as i32) as i16;
            // the original tests the missile's own index here, not the target's
            if (i != 0 || bitrand(DODGE_RANGE) > bv as i32) && chebyshev(xd, yd) < range {
                let by = if (*ms).missile_type == MISSILE_ROCK {
                    BY_FIREBALL
                } else {
                    BY_ARROW
                };
                hit_actor(by, j as i32, fc, (rand8() + MISSILE_DAMAGE_BASE) as i16);
                (*ms).speed = 0;
                if (*ms).missile_type == MISSILE_ROCK {
                    (*ms).missile_type = MISSILE_SPENT;
                }
                break;
            }
            j += 1;
        }
        (*ms).abs_x = step_x((*ms).abs_x as i32, fc, s as i32) as u16;
        (*ms).abs_y = step_y((*ms).abs_y as i32, fc, s as i32) as u16;
    }
}

/// The current number of actors (the loop bound is read again each time, as in the original).
pub fn actor_count() -> usize {
    unsafe { active_actor_count as usize }
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
            let ms = &raw const missile_slots[i];
            let t = (*ms).missile_type;
            if t == MISSILE_NONE {
                continue;
            }
            let xstart = ((*ms).abs_x as i32 - map_x as i32 - SPRITE_OFFSET.0) as i16;
            let ystart = ((*ms).abs_y as i32 - map_y as i32 - SPRITE_OFFSET.1) as i16;
            if xstart > SCREEN_LEFT
                && xstart < SCREEN_RIGHT
                && ystart > SCREEN_TOP
                && ystart < SCREEN_BOTTOM
            {
                let an = &raw mut actor_list[drawn_actor_count as usize];
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
                drawn_actor_count += 1;
                if drawn_actor_count >= ACTOR_SLOTS {
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
}
