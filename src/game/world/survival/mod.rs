//! Hunger and fatigue (from the main loop in csrc/fmain.c): noting a safe spot, eating fruit, and
//! the messages and damage that come with being hungry or tired.

mod hunger;

mod schedule;

pub use self::schedule::survival_tick;

/// Hunger, fatigue and the safe spot are checked when the clock is a multiple of 128; the
/// music is picked when it is a multiple of 8.
const SAFE_CHECK_MASK: u16 = 127;
const STARVING_AT: i16 = 90;

/// The music mood is picked every 8th tick, and hunger changes bite every 8 points.
const MOOD_PERIOD_MASK: u16 = 7;

fn safe_check_due(clock: u16) -> bool {
    clock & SAFE_CHECK_MASK == 0
}
