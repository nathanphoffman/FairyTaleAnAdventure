//! The game's own random number generator (from fsubs.asm via csrc/asm_port.c).
//! It must match the original exactly, and it replaces libc `rand()`.

unsafe extern "C" {
    /// The generator's state, defined in csrc/fmain.c.
    static mut seed1: i32;
}

const SEED_MULTIPLIER: u32 = 45821;
const RESULT_MASK: u32 = 0x7fff_ffff;
const ROTATE_BITS: u32 = 6;
const LOW_WORD_MASK: i32 = 0xffff;

/// Steps the generator: returns (new seed, random value).
fn step(seed: i32) -> (i32, i32) {
    // mulu.w multiplies only the low 16 bits of the seed
    let product = (seed as u16 as u32).wrapping_mul(SEED_MULTIPLIER);
    let next = (product as i32).wrapping_add(1);
    // ror.l #6
    let rotated = (next as u32).rotate_right(ROTATE_BITS);
    (next, (rotated & RESULT_MASK) as i32)
}

/// Value in 0..n for n up to 16 bits, as the 68000 divu did it.
fn bounded(value: i32, n: i32) -> i32 {
    (value as u16 as u32 % (n & LOW_WORD_MASK) as u32) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn fta_rand() -> i32 {
    let (next, value) = step(unsafe { seed1 });
    unsafe { seed1 = next };
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn bitrand(mask: i32) -> i32 {
    fta_rand() & mask
}

#[unsafe(no_mangle)]
pub extern "C" fn rand2() -> i32 {
    fta_rand() & 1
}

#[unsafe(no_mangle)]
pub extern "C" fn rand4() -> i32 {
    fta_rand() & 3
}

#[unsafe(no_mangle)]
pub extern "C" fn rand8() -> i32 {
    fta_rand() & 7
}

#[unsafe(no_mangle)]
pub extern "C" fn rand64() -> i32 {
    fta_rand() & 63
}

#[unsafe(no_mangle)]
pub extern "C" fn rand256() -> i32 {
    fta_rand() & 255
}

#[unsafe(no_mangle)]
pub extern "C" fn rnd(n: i32) -> i32 {
    bounded(fta_rand(), n)
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
        assert_eq!(values, [1777372681, 1298959771, 555739856, 636999534, 442890465]);
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
