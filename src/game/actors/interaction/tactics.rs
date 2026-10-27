//! Carrying out an enemy's tactic: pursuing, following, evading, wandering, shooting or backing up.

use super::*;

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
