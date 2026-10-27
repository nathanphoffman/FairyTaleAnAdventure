//! Fading the page colours: with the time of day, to and from black, and the teleport flicker.

use super::{COLOR_COUNT, Delay, LoadRGB4};
use crate::game::data::object_tables::{fader, page_colors};
use crate::game::data::random::bitrand;
use crate::game::data::state::{the_screen, the_world};

// Colour 31 of the page changes with where the hero is.
const COLOR_31: usize = 31;
const REGION_SWAMP: u16 = 4;
const REGION_SECRET: u16 = 9;
const SWAMP_COLOR: u16 = 0x0980;
const SECRET_OPEN_COLOR: u16 = 0x00f0;
const SECRET_SHUT_COLOR: u16 = 0x0445;
const DEFAULT_COLOR_31: u16 = 0x0bdf;

// Fade limits: nothing goes below these at night.
const NIGHT_MIN: (i32, i32, i32) = (10, 25, 60);
const FULL_LEVEL: i32 = 100;
const MAX_COMPONENT: i32 = 15;

/// Night blue lift: one third of the green that is missing from full light.
const NIGHT_BLUE_SHARE: i32 = 3;

/// A component in its nibble position (up to 0xf0) times a level in percent: dividing by 1600 gives
/// back a 4 bit component.
const SCALE_DIVISOR: i32 = 1600;
const PERCENT: i32 = 100;

/// The lake colours get a moonlight tint at night while green stays above these levels.
const LAKE_COLORS: std::ops::RangeInclusive<usize> = 16..=24;
const MOONLIGHT_MIN_GREEN: i32 = 20;
const MOONLIGHT_STRONG_BELOW: i32 = 50;
const MOONLIGHT_WEAK_BELOW: i32 = 75;
const MOONLIGHT_STRONG_BOOST: i32 = 2;
const MOONLIGHT_WEAK_BOOST: i32 = 1;
const RED_MASK: i32 = 0x0f00;
const GREEN_MASK: i32 = 0x00f0;
const BLUE_MASK: i32 = 0x000f;
const NIBBLE_BITS: u32 = 4;
const RANDOM_COLOR_MASK: i32 = 0xfff;

/// Fades the page colours to the levels `r`, `g`, `b` (percent). With `limit` the night limits
/// apply (red never below 10%, green 25%, blue 60%) and some blue is mixed in from green, which gives
/// the bluish moonlight. The light spell (`light_timer`) makes reds as bright as greens.
/// Colour 31 is changed first because it depends on where the hero is (swamp, secret room).
pub fn fade_page(r: i16, g: i16, b: i16, limit: i16, colors: *const u16) {
    unsafe {
        page_colors[COLOR_31] = color_31_here();

        let ((r, g, b), g2) = clamp_levels((r as i32, g as i32, b as i32), limit != 0);

        for (i, slot) in (*(&raw mut fader)).iter_mut().take(COLOR_COUNT).enumerate() {
            *slot = faded_color(
                i,
                *colors.add(i),
                (r, g, b),
                g2,
                limit != 0,
                the_world.light_timer != 0,
            );
        }

        LoadRGB4(
            &raw mut the_screen.vp_page,
            (&raw const fader).cast(),
            COLOR_COUNT as i32,
        );
    }
}

/// Colour 31 depends on where the hero is: the swamp, the secret room (open or shut), or neither.
unsafe fn color_31_here() -> u16 {
    unsafe {
        match the_world.region {
            REGION_SWAMP => SWAMP_COLOR,
            REGION_SECRET if the_world.secret_timer != 0 => SECRET_OPEN_COLOR,
            REGION_SECRET => SECRET_SHUT_COLOR,
            _ => DEFAULT_COLOR_31,
        }
    }
}

/// Keeps the levels between 0 and 100 percent, or at night between the night limits, and works out
/// how much blue is mixed in from green (`g2`, only at night).
fn clamp_levels((r, g, b): (i32, i32, i32), night: bool) -> ((i32, i32, i32), i32) {
    let (r, g, b) = (r.min(FULL_LEVEL), g.min(FULL_LEVEL), b.min(FULL_LEVEL));

    if night {
        let (r, g, b) = (r.max(NIGHT_MIN.0), g.max(NIGHT_MIN.1), b.max(NIGHT_MIN.2));

        ((r, g, b), (FULL_LEVEL - g) / NIGHT_BLUE_SHARE)
    } else {
        ((r.max(0), g.max(0), b.max(0)), 0)
    }
}

/// Colour `i` of the palette (`color`, 4 bits per channel) scaled to the light levels `r`, `g`, `b`
/// (percent). At night (`night`) blue is lifted by `g2` times green, the lake colours (16-24) get a
/// moonlight tint, and blue is capped. With a magic light (`lit`) red is brought up to the green.
fn faded_color(
    i: usize,
    color: u16,
    (r, g, b): (i32, i32, i32),
    g2: i32,
    night: bool,
    lit: bool,
) -> u16 {
    let color = color as i32;

    // Colours are 12 bit 0RGB. Red is moved down a nibble so all three work on the same scale.
    let mut r1 = (color & RED_MASK) >> NIBBLE_BITS;

    // Masks with GREEN_MASK, keeping only the bits that matter here.
    let mut g1 = color & GREEN_MASK;

    // Masks with BLUE_MASK, keeping only the bits that matter here.
    let mut b1 = color & BLUE_MASK;

    if lit && r1 < g1 {
        r1 = g1;
    }

    r1 = (r * r1) / SCALE_DIVISOR;
    g1 = (g * g1) / SCALE_DIVISOR;
    b1 = (b * b1 + g2 * g1) / PERCENT;

    if night {
        if LAKE_COLORS.contains(&i) && g > MOONLIGHT_MIN_GREEN {
            if g < MOONLIGHT_STRONG_BELOW {
                b1 += MOONLIGHT_STRONG_BOOST;
            } else if g < MOONLIGHT_WEAK_BELOW {
                b1 += MOONLIGHT_WEAK_BOOST;
            }
        }

        b1 = b1.min(MAX_COMPONENT);
    }

    // Back to 0RGB.
    ((r1 << (2 * NIBBLE_BITS)) + (g1 << NIBBLE_BITS) + b1) as u16
}

/// The teleport effect: random colours flicker for a moment (32 flashes).
pub fn flicker_colors() {
    unsafe {
        for _ in 0..COLOR_COUNT {
            for slot in (*(&raw mut fader)).iter_mut().take(COLOR_COUNT).skip(1) {
                *slot = bitrand(RANDOM_COLOR_MASK) as u16;
            }

            LoadRGB4(
                &raw mut the_screen.vp_page,
                (&raw const fader).cast(),
                COLOR_COUNT as i32,
            );

            Delay(1);
        }
    }
}

/// Fading goes in steps of 5 percent, one tick each.
const FADE_STEP: usize = 5;

/// Fades the page out to black.
pub fn fade_down() {
    for level in (0..=FULL_LEVEL).rev().step_by(FADE_STEP) {
        fade_page(
            level as i16,
            level as i16,
            level as i16,
            0,
            (&raw const page_colors).cast(),
        );

        unsafe { Delay(1) };
    }
}

/// Fades the page back in from black.
pub fn fade_normal() {
    for level in (0..=FULL_LEVEL).step_by(FADE_STEP) {
        fade_page(
            level as i16,
            level as i16,
            level as i16,
            0,
            (&raw const page_colors).cast(),
        );

        unsafe { Delay(1) };
    }
}

// Daylight: the page colours follow the time of day.
const DAY_DARKEN: (i32, i32, i32) = (80, 61, 62);
const LIGHT_SPELL_BONUS: i32 = 200;

/// The daylight fade runs every fourth tick.
const FADE_TICK_MASK: u16 = 3;

/// Inside buildings (regions from here on) it is never night.
const FIRST_INDOOR_REGION: u16 = 8;

/// `view_status` above this means the screen has to be rebuilt.
const VIEW_REBUILD: i8 = 97;

/// Updates the page colours for the time of day, every fourth tick. `light_level` follows the sun;
/// inside buildings (regions 8 and 9) it is always bright.
pub fn day_fade() {
    unsafe {
        let bonus = if the_world.light_timer != 0 {
            LIGHT_SPELL_BONUS
        } else {
            0
        };

        // Masks with FADE_TICK_MASK, keeping only the bits that matter here.
        if the_world.day_clock & FADE_TICK_MASK == 0 || the_screen.view_status > VIEW_REBUILD {
            let colors = (&raw const page_colors).cast();

            if the_world.region < FIRST_INDOOR_REGION {
                let level = the_world.light_level as i32;
                fade_page(
                    (level - DAY_DARKEN.0 + bonus) as i16,
                    (level - DAY_DARKEN.1) as i16,
                    (level - DAY_DARKEN.2) as i16,
                    1,
                    colors,
                );
            } else {
                fade_page(
                    FULL_LEVEL as i16,
                    FULL_LEVEL as i16,
                    FULL_LEVEL as i16,
                    1,
                    colors,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_steps_run_from_full_to_none() {
        let down: Vec<i32> = (0..=FULL_LEVEL).rev().step_by(FADE_STEP).collect();
        assert_eq!(down.first(), Some(&100));
        assert_eq!(down.last(), Some(&0));
        assert_eq!(down.len(), 21);
    }

    #[test]
    fn full_light_leaves_colours_alone() {
        assert_eq!(
            faded_color(0, 0x0a5c, (100, 100, 100), 0, false, false),
            0x0a5c
        );
    }

    #[test]
    fn no_light_is_black() {
        assert_eq!(faded_color(0, 0x0fff, (0, 0, 0), 0, false, false), 0);
    }

    #[test]
    fn magic_light_raises_red_to_green() {
        assert_eq!(
            faded_color(0, 0x00f0, (100, 100, 100), 0, false, true),
            0x0ff0
        );
    }

    #[test]
    fn night_tints_the_lake_colours_but_caps_blue() {
        let lake = faded_color(20, 0x0000, (100, 40, 100), 0, true, false);
        assert_eq!(lake & 0xf, 2);
        let capped = faded_color(0, 0x000f, (100, 100, 100), 100, true, false);
        assert_eq!(capped & 0xf, MAX_COMPONENT as u16);
    }
}
