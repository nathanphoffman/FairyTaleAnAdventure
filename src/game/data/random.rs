//! The game's own random number generator (from fsubs.asm via csrc/asm_port.c).
//! It must match the original exactly, and it replaces libc `rand()`.
//!
//! Why it must match: the game uses random numbers for combat, monster behaviour, loot and where
//! things appear. The tests replay recorded sessions and compare the game state, which only works
//! if the same seed gives the same numbers. The generator is a small linear congruential one:
//! multiply the seed by 45821, add 1, then rotate the bits to get the result.

const SEED_MULTIPLIER: u32 = 45821;
const RESULT_MASK: u32 = 0x7fff_ffff;
const ROTATE_BITS: u32 = 6;
const LOW_WORD_MASK: i32 = 0xffff;

/// The generator's two seeds. (Only `seed1` is used; `seed2` is saved with the game.)
pub struct Rng {
    pub seed1: i32,
    pub seed2: i32,
}

impl Rng {
    pub const fn new() -> Rng {
        Rng {
            seed1: 19837325,
            seed2: 23098324,
        }
    }
}

/// Steps the generator once: returns (new seed, random value).
///
/// The 68000 `mulu.w` instruction multiplies only the low 16 bits of the seed, which is why the
/// seed is cut to 16 bits before multiplying. The result is the new seed rotated right by 6 bits
/// with the top bit cleared, so it is always a positive 31-bit number.
fn step(seed: i32) -> (i32, i32) {
    // mulu.w multiplies only the low 16 bits of the seed
    let product = (seed as u16 as u32).wrapping_mul(SEED_MULTIPLIER);
    let next = (product as i32).wrapping_add(1);

    // ror.l #6
    let rotated = (next as u32).rotate_right(ROTATE_BITS);

    (next, (rotated & RESULT_MASK) as i32)
}

/// A value in 0..n for `n` up to 16 bits, as the 68000 `divu` instruction did it.
///
/// `divu` divides a 32-bit number by a 16-bit one and keeps the remainder. Only the low 16 bits of
/// the value and of `n` are used.
fn bounded(value: i32, n: i32) -> i32 {
    (value as u16 as u32 % (n & LOW_WORD_MASK) as u32) as i32
}

/// The next random number (0 to 2^31 - 1). The C code calls this `rand()`; the game's own version
/// replaces the libc one. The seed lives in `seed1`, defined in csrc/fmain.c.
pub fn fta_rand(rng: &mut Rng) -> i32 {
    let (next, value) = step(rng.seed1);

    rng.seed1 = next;

    value
}

/// A random number with only the bits in `mask` kept, e.g. `bitrand(rng, 0x3fff)` is 0 to 16383.
pub fn bitrand(rng: &mut Rng, mask: i32) -> i32 {
    fta_rand(rng) & mask
}

/// Random 0 or 1.
pub fn rand2(rng: &mut Rng) -> i32 {
    fta_rand(rng) & 1
}

/// Random 0 to 3.
pub fn rand4(rng: &mut Rng) -> i32 {
    fta_rand(rng) & 3
}

/// Random 0 to 7 (also used as a random compass direction).
pub fn rand8(rng: &mut Rng) -> i32 {
    fta_rand(rng) & 7
}

/// Random 0 to 63.
pub fn rand64(rng: &mut Rng) -> i32 {
    fta_rand(rng) & 63
}

/// Random 0 to 255.
pub fn rand256(rng: &mut Rng) -> i32 {
    fta_rand(rng) & 255
}

/// A random number from 0 to `n - 1`, for any `n` up to 16 bits.
pub fn rnd(rng: &mut Rng, n: i32) -> i32 {
    bounded(fta_rand(rng), n)
}

#[cfg(test)]
mod tests {
    use super::*;

    const START_SEED: i32 = 19_837_325;

    /// Values produced by the C original with the game's starting seed.
    #[test]
    fn matches_the_c_generator() {
        let mut seed = START_SEED;
        let mut values = Vec::new();

        for _ in 0..5 {
            let (next, value) = step(seed);
            seed = next;
            values.push(value);
        }

        assert_eq!(
            values,
            [1777372681, 1298959771, 555739856, 636999534, 442890465]
        );

        assert_eq!(seed, -1719781274);
    }

    #[test]
    fn bounded_values_match_the_c_original() {
        let mut seed = START_SEED;
        let mut values = Vec::new();

        for _ in 0..5 {
            let (next, value) = step(seed);
            seed = next;
            values.push(bounded(value, 100));
        }

        assert_eq!(values, [61, 51, 12, 50, 13]);
    }
}
