//! Steering an actor toward or away from a target (the enemy-steering routine of the 68000 assembly,
//! from csrc/asm_text.c): turn the actor and start it walking.

use crate::game::data::random::fta_rand;
use crate::game::data::state::the_actors;

// Actor states (csrc/ftale.h).
const WALKING: i8 = 12;
const STILL: i8 = 13;

// Steering modes (see `steer`).
pub const STEER_STRAIGHT: i32 = 4;
pub const STEER_TURN_ONLY: i32 = 5;
pub const STEER_OFFSET: i32 = 6;
const STEER_WOBBLE_FAR: i32 = 1;
const STEER_WOBBLE_NEAR: i32 = 2;
const STEER_AWAY: i32 = 3;

/// The wobble starts when the target is closer than this (in mode 1 and in mode 2).
const WOBBLE_DISTANCE_FAR: i16 = 40;
const WOBBLE_DISTANCE_NEAR: i16 = 30;

/// The steering mode is the low byte of the argument; directions are 0-7.
const MODE_MASK: i32 = 0xff;
const FACING_MASK: i32 = 7;

/// The table is indexed by 4 - 3 * (vertical sign) - (horizontal sign).
const TABLE_CENTER: i32 = 4;
const TABLE_ROW_WIDTH: i32 = 3;

/// An axis is ignored when it is less than half the other (a shift of one).
const IGNORE_SHIFT: u32 = 1;

/// Direction number for each of the 9 combinations of horizontal and vertical steering.
const DIRECTION_TABLE: [u8; 9] = [0, 1, 2, 7, 9, 3, 6, 5, 4];
const NO_DIRECTION: i32 = 9;

/// What the steering decides for an actor.
#[derive(Debug, PartialEq)]
enum Course {
    Stop,
    Face { facing: i32 },
}

/// The sign of a difference as -1, 0 or 1.
fn sign(v: i16) -> i32 {
    (v as i32).signum()
}

/// Works out the direction toward (or away from) a target. `xdif`/`ydif` are actor minus target
/// (or the offset itself in mode 6). Modes: 0 toward, 1 and 2 toward with a sideways wobble when
/// close, 3 away, 4 straight toward (no smart steering), 5 turn only, 6 the offset is given.
///
/// Method: take the sign of each axis, then ignore an axis if the other is more than twice as
/// far (so mostly-horizontal targets give a pure left/right move, not a diagonal). The two signs
/// pick one of 8 directions from a table; if both are zero the actor is on the target and stops.
/// The wobble adds or subtracts one direction step at random when the target is close.
fn steer(xdif: i16, ydif: i16, mode: i32, random_bit: impl FnOnce() -> bool) -> Course {
    let xabs = (xdif as i32).unsigned_abs() as u16;
    let yabs = (ydif as i32).unsigned_abs() as u16;
    let (mut xdir, mut ydir) = (sign(xdif), sign(ydif));

    if mode != STEER_STRAIGHT {
        (xdir, ydir) = along_the_long_axis((xdir, ydir), xabs, yabs);
    }

    let close = (xabs as i32 + yabs as i32) as i16;
    let deviation = wobble(mode, close);

    if mode == STEER_AWAY {
        xdir = -xdir;
        ydir = -ydir;
    }

    let mut facing =
        DIRECTION_TABLE[(TABLE_CENTER - ydir * TABLE_ROW_WIDTH - xdir) as usize] as i32;

    if facing == NO_DIRECTION {
        return Course::Stop;
    }

    facing += if random_bit() { deviation } else { -deviation };

    Course::Face {
        // Masks with FACING_MASK, keeping only the bits that matter here.
        facing: facing & FACING_MASK,
    }
}

/// An axis less than half as far as the other is ignored, so the actor goes straight along the
/// long one.
fn along_the_long_axis((xdir, ydir): (i32, i32), xabs: u16, yabs: u16) -> (i32, i32) {
    // Shifts by IGNORE_SHIFT, converting between pixels, tiles, sectors or bytes.
    let ydir = if (yabs as i16) < (xabs >> IGNORE_SHIFT) as i16 {
        0
    } else {
        ydir
    };

    // Shifts by IGNORE_SHIFT, converting between pixels, tiles, sectors or bytes.
    let xdir = if (xabs as i16) < (yabs >> IGNORE_SHIFT) as i16 {
        0
    } else {
        xdir
    };

    (xdir, ydir)
}

/// The sideways wobble (one direction step) of modes 1 and 2 when the target is close, else 0.
fn wobble(mode: i32, close: i16) -> i32 {
    let wobbles = (mode == STEER_WOBBLE_FAR && close < WOBBLE_DISTANCE_FAR)
        || (mode == STEER_WOBBLE_NEAR && close < WOBBLE_DISTANCE_NEAR);

    wobbles as i32
}

/// Points an actor toward (or away from) a target and sets it walking (see `steer` for modes).
/// The random number is only drawn when the actor moves, as in the original, so recorded games
/// stay in step.
pub fn steer_actor(object: i32, target_x: i32, target_y: i32, mode: i32) {
    // Masks with MODE_MASK, keeping only the bits that matter here.
    let mode = mode & MODE_MASK;
    unsafe {
        let actor = &mut (*(&raw mut the_actors.list))[object as usize];
        let (xdif, ydif) = if mode == STEER_OFFSET {
            (target_x as i16, target_y as i16)
        } else {
            (
                (actor.abs_x as i32 - target_x) as i16,
                (actor.abs_y as i32 - target_y) as i16,
            )
        };

        // Masks off the low bits (a cheap remainder or a flag test).
        match steer(xdif, ydif, mode, || fta_rand() & 2 != 0) {
            Course::Stop => actor.state = STILL,
            Course::Face { facing } => {
                actor.facing = facing as i8;

                if mode != STEER_TURN_ONLY {
                    actor.state = WALKING;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steers_toward_a_target() {
        // actor is right of and level with the target: direction 7
        assert_eq!(steer(100, 0, 0, || false), Course::Face { facing: 7 });

        // below the target: direction 1
        assert_eq!(steer(0, 100, 0, || false), Course::Face { facing: 1 });

        // exactly on the target: stop
        assert_eq!(steer(0, 0, 0, || false), Course::Stop);
    }

    #[test]
    fn steering_away_reverses_direction() {
        assert_eq!(steer(100, 0, 3, || false), Course::Face { facing: 3 });
    }

    #[test]
    fn close_targets_wobble_in_modes_1_and_2() {
        assert_eq!(steer(10, 0, 1, || true), Course::Face { facing: 0 });
        assert_eq!(steer(10, 0, 1, || false), Course::Face { facing: 6 });

        // 35 is close enough for mode 1 (under 40) but not for mode 2 (under 30)
        assert_eq!(steer(35, 0, 1, || true), Course::Face { facing: 0 });
        assert_eq!(steer(35, 0, 2, || true), Course::Face { facing: 7 });
    }

    #[test]
    fn diagonal_needs_both_axes_to_matter() {
        // a 3:1 slope is treated as straight along x
        assert_eq!(steer(90, 30, 0, || false), Course::Face { facing: 7 });
        assert_eq!(steer(60, 60, 0, || false), Course::Face { facing: 0 });
    }

    #[test]
    fn the_random_bit_is_only_used_when_the_actor_moves() {
        let mut asked = false;
        steer(0, 0, 0, || {
            asked = true;

            true
        });

        assert!(!asked);
    }
}
