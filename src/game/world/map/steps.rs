//! Stepping a position in one of the 8 directions, with the 16-bit wrap-around of the original.

/// World coordinates are 15 bit.
pub(super) const COORD_MASK: i32 = 0x7fff;
pub(super) const COORD_SIGN: i32 = 0x8000;

// Steps for the 8 directions (and two "no move" entries) per half-unit of speed.
pub(super) const X_STEPS: [i16; 10] = [-2, 0, 2, 3, 2, 0, -2, -3, 0, 0];
pub(super) const Y_STEPS: [i16; 10] = [-2, -3, -2, 0, 2, 3, 2, 0, 0, 0];

/// Directions are the low byte of the value; a step is half the table value times the speed.
const DIRECTION_MASK: i32 = 0xff;
const WORD_MASK: u32 = 0xffff;
const HIGH_WORD_MASK: u32 = 0xffff_0000;
const SIGN_BIT_14: u32 = 0x4000;
const SIGN_BIT_15: u32 = 0x8000;
const LOW_15_BITS: u32 = 0x7fff;

/// Directions above this mean "no move".
pub(super) const MAX_DIRECTION: i32 = 7;

/// Moves `v` one step in direction `dir` at `speed`; the result stays within 15 bits.
/// The step is table value * speed / 2, so speed 2 moves the table's amount in pixels.
pub(super) fn step(v: i32, dir: i32, speed: i32, table: &[i16; 10]) -> i32 {
    let product = table[(dir & DIRECTION_MASK) as usize] as i32 * speed as i16 as i32;

    // Half the table step times the speed: speed 2 moves the table's amount.
    let half = (product as u16 >> 1) as i32;

    (v.wrapping_add(half) as u16 as i32) & COORD_MASK
}

/// The x position after one step in direction `dir` (0-7) at `speed`. Directions above 7 mean
/// "no move". Directions go clockwise from north-west.
pub fn step_x(x: i32, dir: i32, speed: i32) -> i32 {
    if dir & DIRECTION_MASK > MAX_DIRECTION {
        return x;
    }

    step(x, dir, speed, &X_STEPS)
}

/// Like `step_x`, but for y; it keeps bit 15 of y (a flag the game stores there).
pub fn step_y(y: i32, dir: i32, speed: i32) -> i32 {
    if dir & DIRECTION_MASK > MAX_DIRECTION {
        return y;
    }

    step(y, dir, speed, &Y_STEPS) | (y & COORD_SIGN)
}

/// Copies bit 14 into bit 15, giving a signed 15 bit value (the world wraps around at the edges).
pub fn wrap_coordinate(x: i32) -> i32 {
    let mut low = x as u32 & WORD_MASK;

    if low & SIGN_BIT_14 != 0 {
        low |= SIGN_BIT_15;
    } else {
        low &= LOW_15_BITS;
    }

    ((x as u32 & HIGH_WORD_MASK) | low) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_follow_the_direction_tables() {
        assert_eq!(step_x(100, 2, 4), 104); // east, speed 4: +2 * 4 / 2
        assert_eq!(step_x(100, 6, 4), 96);
        assert_eq!(step_x(100, 9, 4), 100); // no move
        assert_eq!(step_y(100, 1, 2), 97); // north: the negative step wraps in 16 bits, then stays 15 bit
    }

    #[test]
    fn newy_keeps_bit_15() {
        // Bit 15 of y is a flag, and stepping must leave it set.
        assert_eq!(step_y(0x8000 | 100, 4, 2) & 0x8000, 0x8000);
    }

    #[test]
    fn wrap_copies_bit_14_to_bit_15() {
        assert_eq!(wrap_coordinate(0x4000), 0xc000);
        assert_eq!(wrap_coordinate(0x1234), 0x1234);
        assert_eq!(wrap_coordinate(0x9234), 0x1234);
    }
}
