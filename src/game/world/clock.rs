//! The day and night clock (from the main loop in csrc/fmain.c): sleeping, the time of day, the
//! light level, the messages when a new part of the day starts, and slow healing.

use crate::game::data::object_tables::ob_listg;
use crate::game::data::random::rand64;
use crate::game::data::state::{the_actors, the_hero, the_world};
use crate::game::data::types::Shape;
use crate::game::render::screen_fx::day_fade;
use crate::game::ui::text::{queue_print, show_event};

// Actor kinds and states (csrc/ftale.h).
const STILL: i8 = 13;
const DEAD: i8 = 15;
const SLEEP: i8 = 23;

// Print queue entries.
const PRQ_STATS: i32 = 4;

const DAY_LENGTH: u16 = 24000;

/// Sleeping makes time pass this much faster.
const SLEEP_TIME_STEP: u16 = 63;

/// The hero wakes at dawn (this window) if only a little tired.
const DAWN: (u16, u16) = (9000, 10000);
const LITTLE_TIRED: i16 = 30;
const LIGHT_PEAK: u16 = 300;
const DARK: u16 = 40;

/// The night-time object: the lamp post at this slot of `ob_listg` is on in the dark.
const LAMP_SLOT: usize = 5;
const LAMP_ON: i8 = 3;
const LAMP_OFF: i8 = 2;
const PERIOD_LENGTH: u16 = 2000;

/// Events said when a new part of the day starts.
const PERIOD_EVENTS: [(i16, i32); 4] = [(0, 28), (4, 29), (6, 30), (9, 31)];

/// The hero heals a little every time the clock passes a multiple of this (minus one).
const HEAL_MASK: u16 = 0x3ff;
const HEAL_BASE: i16 = 15;

/// A quarter of the bravery is added to the healing limit.
const BRAVERY_DIVISOR: i16 = 4;
const WAKE_Y_MASK: u16 = 0xffe0;

/// The day clock after one frame: it stands still while time is frozen and wraps to 0 once it
/// is past `DAY_LENGTH`.
fn clock_step(clock: u16, frozen: bool) -> u16 {
    if frozen {
        return clock;
    }

    if clock >= DAY_LENGTH {
        0
    } else {
        clock.wrapping_add(1)
    }
}

/// The light level for a clock value: it rises to `LIGHT_PEAK` at midday and falls again.
fn light_level(clock: u16) -> u16 {
    let level = clock / DARK;

    if level >= LIGHT_PEAK {
        2 * LIGHT_PEAK - level
    } else {
        level
    }
}

/// The lamp post is lit when it is dark.
fn lamp_is_on(level: u16) -> bool {
    level < DARK
}

/// Which part of the day (0..=11) the clock is in.
fn day_period(clock: u16) -> i16 {
    (clock / PERIOD_LENGTH) as i16
}

/// The message said when `period` starts, if it has one.
fn period_event(period: i16) -> Option<i32> {
    PERIOD_EVENTS
        .iter()
        .find(|(p, _)| *p == period)
        .map(|(_, e)| *e)
}

/// Slow healing: once per `HEAL_MASK + 1` clock ticks, if hurt and alive.
fn heal_due(clock: u16, vitality: i16, bravery: i16, state: i8) -> bool {
    clock & HEAL_MASK == 0 && vitality < HEAL_BASE + bravery / BRAVERY_DIVISOR && state != DEAD
}

/// Sleeping, the day clock, the light level, the messages for the time of day and slow healing.
pub fn day_night_tick() {
    unsafe {
        let hero = &raw mut the_actors.list[0];

        if (*hero).state == SLEEP {
            sleep_tick(hero);
        }

        let tick = day_step(
            the_world.day_clock,
            the_world.freeze_timer != 0,
            the_world.day_period,
            (*hero).vitality,
            the_hero.brave,
            (*hero).state,
        );

        the_world.day_clock = tick.clock;
        the_world.light_level = tick.light;

        ob_listg[LAMP_SLOT].ob_stat = if tick.lamp_on { LAMP_ON } else { LAMP_OFF };
        the_world.day_period = tick.period;

        if let Some(e) = tick.period_event {
            show_event(e);
        }

        day_fade();

        if tick.heal {
            (*hero).vitality += 1;
            queue_print(PRQ_STATS);
        }
    }
}

/// What one frame of the day clock changes.
#[derive(Debug, PartialEq, Eq)]
struct DayTick {
    clock: u16,
    light: u16,
    lamp_on: bool,
    period: i16,
    /// The message for a new part of the day, if one started this frame and has one.
    period_event: Option<i32>,
    /// The hero heals one point.
    heal: bool,
}

/// The day clock, light, lamp, part of the day and slow healing after one frame.
fn day_step(
    clock: u16,
    frozen: bool,
    old_period: i16,
    vitality: i16,
    bravery: i16,
    state: i8,
) -> DayTick {
    let clock = clock_step(clock, frozen);
    let light = light_level(clock);
    let period = day_period(clock);

    DayTick {
        clock,
        light,
        lamp_on: lamp_is_on(light),
        period,
        period_event: if period != old_period {
            period_event(period)
        } else {
            None
        },
        heal: heal_due(clock, vitality, bravery, state),
    }
}

/// One frame of sleep: returns the new clock, the new fatigue and whether the hero wakes.
/// `fight_wakes` is only called (it draws a random number) when a battle is on and no other
/// reason to wake applied.
fn sleep_step(
    clock: u16,
    tired: i16,
    battle: bool,
    fight_wakes: impl FnOnce() -> bool,
) -> (u16, i16, bool) {
    let clock = clock.wrapping_add(SLEEP_TIME_STEP);

    let tired = if tired != 0 { tired - 1 } else { tired };
    let wake = tired == 0
        || (tired < LITTLE_TIRED && clock > DAWN.0 && clock < DAWN.1)
        || (battle && fight_wakes());

    (clock, tired, wake)
}

/// While asleep time runs fast; the hero wakes when rested, at dawn, or when a fight starts.
unsafe fn sleep_tick(hero: *mut Shape) {
    unsafe {
        let (clock, tired, wake) = sleep_step(
            the_world.day_clock,
            the_hero.fatigue,
            the_actors.battle_flag != 0,
            || rand64() == 0,
        );

        the_world.day_clock = clock;
        the_hero.fatigue = tired;

        if wake {
            (*hero).state = STILL;
            (*hero).abs_y &= WAKE_Y_MASK;
            the_hero.y = (*hero).abs_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_of_the_day() {
        let t = day_step(1999, false, 0, 50, 0, STILL);
        assert_eq!(
            (
                t.clock,
                t.light,
                t.lamp_on,
                t.period,
                t.period_event,
                t.heal
            ),
            (2000, 50, false, 1, None, false)
        );

        // a new period with a message
        let t = day_step(7999, false, 3, 50, 0, STILL);
        assert_eq!((t.period, t.period_event), (4, Some(29)));

        // the same period again says nothing
        assert_eq!(day_step(8000, false, 4, 50, 0, STILL).period_event, None);

        // night: the lamp is on
        assert!(day_step(100, false, 0, 50, 0, STILL).lamp_on);

        // frozen time stands still
        assert_eq!(day_step(500, true, 0, 50, 0, STILL).clock, 500);

        // healing lands on multiples of 1024 for a hurt hero
        assert!(day_step(1023, false, 0, 10, 0, STILL).heal);
        assert!(!day_step(1023, false, 0, 10, 0, DEAD).heal);
        assert!(!day_step(1024, false, 0, 10, 0, STILL).heal);
    }

    #[test]
    fn clock_counts_wraps_and_freezes() {
        assert_eq!(clock_step(0, false), 1);
        assert_eq!(clock_step(500, true), 500);
        assert_eq!(clock_step(DAY_LENGTH - 1, false), DAY_LENGTH);
        assert_eq!(clock_step(DAY_LENGTH, false), 0);
    }

    #[test]
    fn light_follows_the_sun() {
        assert_eq!(light_level(0), 0);
        assert_eq!(light_level(39), 0);
        assert_eq!(light_level(40), 1);
        assert_eq!(light_level(300 * 40 - 1), 299);
        assert_eq!(light_level(300 * 40), 300);
        assert_eq!(light_level(301 * 40), 299);
        assert_eq!(light_level(23999), 1);
        assert!(lamp_is_on(light_level(0)) && lamp_is_on(light_level(39 * 40 + 39)));
        assert!(!lamp_is_on(light_level(40 * 40)));
    }

    #[test]
    fn day_periods_and_messages() {
        assert_eq!(day_period(0), 0);
        assert_eq!(day_period(1999), 0);
        assert_eq!(day_period(2000), 1);
        assert_eq!(day_period(23999), 11);
        let said: Vec<_> = (0..12)
            .filter_map(|p| period_event(p).map(|e| (p, e)))
            .collect();
        assert_eq!(said, vec![(0, 28), (4, 29), (6, 30), (9, 31)]);
    }

    #[test]
    fn healing_every_1024_ticks() {
        assert!(heal_due(0, 10, 0, STILL));
        assert!(heal_due(1024, 10, 0, STILL));
        assert!(!heal_due(1023, 10, 0, STILL));
        assert!(!heal_due(1024, 15, 0, STILL));
        assert!(heal_due(1024, 15, 4, STILL));
        assert!(!heal_due(1024, 10, 0, DEAD));
    }

    #[test]
    fn sleep_steps_fast_and_wakes() {
        let mut drawn = 0;
        let (clock, tired, wake) = sleep_step(100, 50, false, || {
            drawn += 1;

            true
        });

        assert_eq!((clock, tired, wake, drawn), (163, 49, false, 0));
        assert!(sleep_step(100, 1, false, || false).2);
        assert!(sleep_step(9000, 20, false, || false).2);
        assert!(!sleep_step(9000, 40, false, || false).2);
        assert!(!sleep_step(8000, 20, false, || false).2);
        let mut drawn = 0;
        assert!(
            sleep_step(100, 50, true, || {
                drawn += 1;
                true
            })
            .2
        );

        assert_eq!(drawn, 1);
        let mut drawn = 0;
        sleep_step(100, 1, true, || {
            drawn += 1;

            true
        });

        assert_eq!(drawn, 0);
        assert_eq!(sleep_step(0, 0, false, || false).1, 0);
    }
}
