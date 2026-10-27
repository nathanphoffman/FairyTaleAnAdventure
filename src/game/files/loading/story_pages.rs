//! The story pages of the intro: showing them and the page-turn animation between them.

use crate::amiga::BitMap;
use crate::game::data::state::the_screen;
use crate::game::files::iff_picture::load_picture;
use crate::game::input::handler::next_key;
use crate::game::render::display::flip_pages;
use crate::platform::display::delay;
use crate::platform::gfx::blt_bit_map;
use std::ffi::c_char;

const BLIT_COPY: u32 = 0xC0;
const ALL_PLANES: u32 = 0x1f;

/// Set once the player has pressed Space to skip the intro.
pub static mut intro_skip_requested: i8 = 0;

/// Whether the intro is being skipped. Once Space has been seen the skip sticks (the original
/// reset it on every poll without a key).
pub fn intro_skipped() -> i32 {
    unsafe {
        if next_key() == b' ' as i32 {
            intro_skip_requested = 1;
        }

        intro_skip_requested as i32
    }
}

const SCREEN_WIDTH: i32 = 320;
const SCREEN_HEIGHT: i32 = 200;

/// Where the first story picture goes on the page.
const PICTURE_X: i16 = 4;
const PICTURE_Y: i16 = 24;

/// Ticks to show a page before turning it.
const PAGE_DELAY: i32 = 350;

/// Shows two pictures on the story page, then turns it over.
/// The new pictures are drawn onto `pageb` while `pagea` keeps the old page for the turn.
pub fn show_story_page(br1: *const c_char, br2: *const c_char, x: i16, y: i16) {
    unsafe {
        if intro_skip_requested != 0 {
            return;
        }

        delay(PAGE_DELAY);
        blt_bit_map(
            &raw mut the_screen.pageb,
            0,
            0,
            &raw mut the_screen.pagea,
            0,
            0,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            BLIT_COPY,
            ALL_PLANES,
            std::ptr::null_mut(),
        );

        load_picture(br1, &raw mut the_screen.pageb, PICTURE_X, PICTURE_Y);
        load_picture(br2, &raw mut the_screen.pageb, x, y);

        if intro_skipped() != 0 {
            return;
        }

        turn_page();
        intro_skipped();
    }
}

// The page-turn animation, 22 frames: strip step, strip width and delay for each.
const FLIP_RATE: [i16; 22] = [
    8, 6, 5, 4, 3, 2, 3, 5, 13, 0, 0, 13, 5, 3, 2, 3, 4, 5, 6, 8, 0, 0,
];

const FLIP_WIDE: [i16; 22] = [
    7, 5, 4, 3, 2, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 2, 3, 4, 5, 7, 0, 0,
];

const FLIP_DELAY: [i32; 22] = [
    12, 9, 6, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 6, 9, 0, 0,
];

/// The first half of the frames turn the right page up, the rest lay the left one down.
const FIRST_HALF: usize = 11;
const PAGE_HALF_WIDTH: i32 = 135;
const PAGE_STRIP_END: i16 = 136;

/// The page turn is drawn around the book's spine at x = 160 (the right page starts one pixel
/// later); the left page starts at x = 24 and the outer edge of the right page is at x = 296. The
/// last strip is a pixel wide, 7 lines in from the top and bottom, which leaves 186 lines.
const SPINE: i32 = 160;
const SPINE_RIGHT: i32 = SPINE + 1;
const PAGE_LEFT_EDGE: i32 = 24;
const PAGE_RIGHT_EDGE: i32 = 296;
const EDGE_INSET: i32 = 7;
const PAGE_EDGE_HEIGHT: i32 = 186;

/// Copies a rectangle between bitmaps (all five planes, plain copy).
#[allow(clippy::too_many_arguments)] // source and destination bitmap with their rectangle
fn copy_rect(
    src: *mut BitMap,
    sx: i32,
    sy: i32,
    dst: *mut BitMap,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
) {
    blt_bit_map(
        src,
        sx,
        sy,
        dst,
        dx,
        dy,
        w,
        h,
        BLIT_COPY,
        ALL_PLANES,
        std::ptr::null_mut(),
    );
}

/// Copies the half page that starts at column `column` onto the screen, in the same place.
fn copy_half_page(src: *mut BitMap, temp: *mut BitMap, column: i32) {
    copy_rect(
        src,
        column,
        0,
        temp,
        column,
        0,
        PAGE_HALF_WIDTH,
        SCREEN_HEIGHT,
    );
}

/// Copies a strip `wide` columns wide from `from` to `to`, `inset` lines in from the top and bottom
/// (the curl of the page).
fn copy_strip(src: *mut BitMap, temp: *mut BitMap, (from, to): (i32, i32), inset: i32, wide: i32) {
    copy_rect(
        src,
        from,
        inset,
        temp,
        to,
        inset,
        wide,
        SCREEN_HEIGHT - inset - inset,
    );
}

/// The page turn animation, 22 frames.
///
/// The first 11 frames lift the right page: the old page is drawn as thin vertical strips that
/// get narrower and shorter (using `page_turn_inset` for the curl), on top of the new page. The last 11
/// frames lay the curling page down on the left. `FLIP_RATE`, `FLIP_WIDE` and `FLIP_DELAY` give
/// the strip step, strip width and pause for each frame.
pub fn turn_page() {
    unsafe {
        let (a, b) = (&raw mut the_screen.pagea, &raw mut the_screen.pageb);

        for frame in 0..FLIP_RATE.len() {
            let temp = (*(*the_screen.drawing_page).ri_page).BitMap;
            let rate = FLIP_RATE[frame];
            let wide = FLIP_WIDE[frame] as i32;

            if frame < FIRST_HALF {
                turn_right_page_frame(a, b, temp, rate, wide);
            } else {
                turn_left_page_frame(a, b, temp, rate, wide);
            }

            flip_pages();

            if FLIP_DELAY[frame] != 0 {
                delay(FLIP_DELAY[frame]);
            }
        }
    }
}

/// One frame of the first half of the page turn: the right page `b` stays and the strips of the old
/// page `a` bend away from the spine, each `wide` pixels wide and `rate` apart.
fn turn_right_page_frame(a: *mut BitMap, b: *mut BitMap, temp: *mut BitMap, rate: i16, wide: i32) {
    let bcol = SPINE_RIGHT - wide;
    let mut dcol: i32 = 0;
    copy_half_page(b, temp, SPINE);

    if rate == 0 {
        return;
    }

    let mut h = 0;
    let mut scol = wide as i16;

    while scol < PAGE_STRIP_END {
        h = crate::game::world::map::page_turn_inset(scol as i32);
        copy_strip(a, temp, (bcol + scol as i32, SPINE_RIGHT + dcol), h, wide);

        dcol += wide;
        scol += rate;
    }

    copy_rect(
        a,
        PAGE_RIGHT_EDGE,
        EDGE_INSET,
        temp,
        SPINE_RIGHT + dcol,
        h,
        1,
        PAGE_EDGE_HEIGHT,
    );
}

/// One frame of the second half of the page turn: the strips of the new page `b` land on the left.
fn turn_left_page_frame(a: *mut BitMap, b: *mut BitMap, temp: *mut BitMap, rate: i16, wide: i32) {
    let bcol = SPINE;

    if rate == 0 {
        copy_half_page(b, temp, PAGE_LEFT_EDGE);

        return;
    }

    copy_half_page(a, temp, PAGE_LEFT_EDGE);
    let mut dcol: i32 = 0;
    let mut scol = wide as i16;

    while scol < PAGE_STRIP_END {
        let h = crate::game::world::map::page_turn_inset(scol as i32);
        dcol += wide;
        copy_strip(b, temp, (bcol - scol as i32, bcol - dcol), h, wide);

        scol += rate;
    }

    copy_strip(b, temp, (PAGE_LEFT_EDGE, SPINE - 1 - dcol), EDGE_INSET, 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_tables_have_one_entry_per_frame() {
        assert_eq!(FLIP_RATE.len(), FLIP_WIDE.len());
        assert_eq!(FLIP_RATE.len(), FLIP_DELAY.len());
    }
}
