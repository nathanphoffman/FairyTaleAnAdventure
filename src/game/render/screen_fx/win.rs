//! The win sequence: a sunrise over the winning picture.

use super::zoom::set_intro_zoom;
use super::{COLOR_COUNT, Delay, HIRES, LoadRGB4, SPRITES, VP_HIDE};
use crate::game::data::object_tables::{fader, sun_colors};
use crate::game::data::state::the_screen;
use crate::game::data::tables::black_colors;
use crate::game::files::iff_picture::load_picture;
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
    unsafe {
        placard_text(WIN_MESSAGE_FIRST);
        name();
        placard_text(WIN_MESSAGE_SECOND);
        draw_placard_border();
        Delay(WIN_MESSAGE_PAUSE);
        the_screen.bm_draw = (*(*the_screen.drawing_page).ri_page).BitMap;
        load_picture(c"winpic".as_ptr(), the_screen.bm_draw, 0, 0);
        LoadRGB4(
            &raw mut the_screen.vp_page,
            (&raw const black_colors).cast::<u16>(),
            COLOR_COUNT as i32,
        );

        LoadRGB4(
            &raw mut the_screen.vp_text,
            (&raw const black_colors).cast::<u16>(),
            COLOR_COUNT as i32,
        );

        the_screen.vp_text.Modes = HIRES | SPRITES | VP_HIDE;
        set_intro_zoom(WIN_SCREEN_SIZE);

        for i in (SUNRISE_END + 1..=SUNRISE_START).rev() {
            fader = sunrise_palette(i);
            LoadRGB4(
                &raw mut the_screen.vp_page,
                (&raw const fader).cast(),
                COLOR_COUNT as i32,
            );

            if i == SUNRISE_START {
                Delay(SUNRISE_FIRST_PAUSE);
            }

            Delay(SUNRISE_STEP_PAUSE);
        }

        Delay(SUNRISE_END_PAUSE);
        LoadRGB4(
            &raw mut the_screen.vp_page,
            (&raw const black_colors).cast::<u16>(),
            COLOR_COUNT as i32,
        );
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
