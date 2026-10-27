//! The win sequence: a sunrise over the winning picture.

use super::COLOR_COUNT;
use super::zoom::set_intro_zoom;
use crate::game::data::object_tables::sun_colors;
use crate::game::data::tables::black_colors;
use crate::game::files::iff_picture::load_picture;
use crate::game::render::display::delay;
use crate::game::render::screen::{HIRES, SPRITES, VP_HIDE, ViewPortId, screen};
use crate::game::ui::messages::name;
use crate::game::ui::text::{draw_placard_border, placard_text};

// The win sequence: a sunrise over the winning picture.
const WIN_MESSAGE_FIRST: i32 = 6;
const WIN_MESSAGE_SECOND: i32 = 7;
const SUNRISE_START: i32 = 25;
const SUNRISE_END: i32 = -30;
const SUNRISE_COLORS: std::ops::Range<usize> = 2..28;
const SUN_GLOW_UNTIL: i32 = -14;
const WIN_SCREEN_SIZE: i32 = 156;

// Ticks to wait in the win sequence.
const WIN_MESSAGE_PAUSE: i32 = 80;
const SUNRISE_FIRST_PAUSE: i32 = 60;
const SUNRISE_STEP_PAUSE: i32 = 9;
const SUNRISE_END_PAUSE: i32 = 30;

// Sunrise palette colours: the white ones and the orange glow.
const WHITE: u16 = 0xfff;
const GLOW_FIRST: usize = 29;
const GLOW_SECOND: usize = 30;
const WHITE_COLORS: [usize; 2] = [1, 28];
const GLOW_BRIGHT: (u16, u16) = (0x800, 0x400);
const GLOW_FADE_OFFSET: i32 = 30;
const RED_STEP: i32 = 0x100;

/// The win screen: shows the picture and brings the sun up behind it by cycling the sky colours
/// through `sun_colors`, with two glowing colours that fade out near the end.
pub fn win_colors() {
    placard_text(WIN_MESSAGE_FIRST);
    name();
    placard_text(WIN_MESSAGE_SECOND);
    draw_placard_border();
    delay(WIN_MESSAGE_PAUSE);
    show_win_picture_in_black();
    rise_sun();

    delay(SUNRISE_END_PAUSE);
    screen().load_colors(ViewPortId::Page, &black_colors);
}

/// Loads the winning picture with every colour black, so the sunrise can fade it in.
fn show_win_picture_in_black() {
    let screen = screen();
    let id = screen.drawing_bitmap_id();
    screen.draw_target = id;
    load_picture("winpic", screen.bitmap_mut(id), 0, 0);
    screen.load_colors(ViewPortId::Page, &black_colors);
    screen.load_colors(ViewPortId::Text, &black_colors);
    screen.vp_text.modes = HIRES | SPRITES | VP_HIDE;
    set_intro_zoom(WIN_SCREEN_SIZE);
}

/// Steps through the sunrise palettes, one pause each.
fn rise_sun() {
    for i in (SUNRISE_END + 1..=SUNRISE_START).rev() {
        screen().load_colors(ViewPortId::Page, &sunrise_palette(i));

        if i == SUNRISE_START {
            delay(SUNRISE_FIRST_PAUSE);
        }

        delay(SUNRISE_STEP_PAUSE);
    }
}

/// The palette for step `i` of the sunrise over the ending picture. Colours 1 and 28 are white,
/// 0 and 31 black; the sky and sun colours come from `sun_colors` offset by `i`, and the glow
/// (29, 30) is a fixed orange at first and then fades out.
fn sunrise_palette(i: i32) -> [u16; COLOR_COUNT] {
    let mut colors = [0u16; COLOR_COUNT];

    for white in WHITE_COLORS {
        colors[white] = WHITE;
    }

    for j in SUNRISE_COLORS {
        let index = i + j as i32;
        colors[j] = if index > 0 {
            sun_colors[index as usize] as u16
        } else {
            0
        };
    }

    if i > SUN_GLOW_UNTIL {
        colors[GLOW_FIRST] = GLOW_BRIGHT.0;
        colors[GLOW_SECOND] = GLOW_BRIGHT.1;
    } else {
        let j = (i + GLOW_FADE_OFFSET) / 2;
        colors[GLOW_FIRST] = (RED_STEP * j) as u16;
        colors[GLOW_SECOND] = (RED_STEP * (j / 2)) as u16;
    }

    colors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sunrise_palette_keeps_fixed_colours_and_fades_the_glow() {
        let early = sunrise_palette(SUNRISE_START);
        assert_eq!(
            (early[0], early[1], early[28], early[31]),
            (0, 0xfff, 0xfff, 0)
        );

        assert_eq!((early[29], early[30]), (0x800, 0x400));
        let late = sunrise_palette(SUN_GLOW_UNTIL);
        assert_eq!((late[29], late[30]), (0x100 * 8, 0x100 * 4));

        // the sky colours at the very end (index <= 0) are black
        assert!(sunrise_palette(-30)[2..28].iter().all(|&c| c == 0));
    }
}
