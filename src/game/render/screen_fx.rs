//! Screen colour effects and panel updates (from csrc/fmain2.c): the compass, fading the page
//! colours with the time of day, the teleport flash, the print queue and the win sequence.
//! The colour tables live in `tables`; the view ports are still defined in csrc/fmain.c.
//!
//! Colours: the Amiga screen has 32 colours, each 12 bits (4 bits red, green, blue). The game
//! fades the whole picture by scaling those components (`fade_page`) and loads them with
//! `LoadRGB4`. Night is a darker, bluer scaling; indoors it is always full brightness.

use crate::amiga::{BitMap, RastPort, View, ViewPort};
use crate::game::data::globals::main_view;
use crate::game::data::object_tables::{
    comptable, fader, page_colors, pplay, prec, print_que, sun_colors,
};

use crate::game::data::random::bitrand;
use crate::game::data::state::{the_actors, the_hero, the_memory, the_screen, the_world};
use crate::game::data::tables::{black_colors, intro_colors};
use crate::game::files::iff_picture::load_picture;
use crate::game::render::display::flip_pages;
use crate::game::ui::menus::print_options;
use crate::game::ui::messages::{name, print};
use crate::game::ui::text::{draw_placard_border, draw_text, move_to, placard_text, print_number};

unsafe extern "C" {
    fn BltBitMap(
        src: *mut BitMap,
        sx: i32,
        sy: i32,
        dst: *mut BitMap,
        dx: i32,
        dy: i32,
        w: i32,
        h: i32,
        minterm: u32,
        mask: u32,
        temp: *mut u8,
    ) -> i32;

    fn LoadRGB4(current_viewport: *mut ViewPort, colors: *const u16, count: i32);

    fn Delay(ticks: i32);

    fn SetDrMd(current_rastport: *mut RastPort, mode: u32);

    fn SetAPen(current_rastport: *mut RastPort, pen: u32);

    fn SetRast(current_rastport: *mut RastPort, pen: u32);

    fn AvailMem(flags: u32) -> u32;

    fn MakeVPort(view: *mut View, port: *mut ViewPort) -> i32;
}

const BLIT_COPY: u32 = 0xC0;

/// The compass only touches the plane the highlight is in.
const COMPASS_PLANE_MASK: u32 = 4;
const COMPASS_PLANE: usize = 2;
const COMPASS_X: i32 = 567;
const COMPASS_Y: i32 = 15;
const COMPASS_WIDTH: i32 = 48;
const COMPASS_HEIGHT: i32 = 24;

/// Compass directions below this are lit; the rest mean "not walking".
const COMPASS_DIRECTIONS: i16 = 9;

/// Redraws the compass with direction `dir` lit. First the unlit compass is copied, then the lit
/// part for that direction (if any) is copied on top from a second image.
pub fn draw_compass(dir: i16) {
    unsafe {
        let part = &comptable[dir as usize];
        let (xr, yr) = (part.xrect as i32, part.yrect as i32);
        (*the_screen.bm_source).Planes[COMPASS_PLANE] = the_memory.nhinor;
        BltBitMap(
            the_screen.bm_source,
            0,
            0,
            the_screen.bm_text,
            COMPASS_X,
            COMPASS_Y,
            COMPASS_WIDTH,
            COMPASS_HEIGHT,
            BLIT_COPY,
            COMPASS_PLANE_MASK,
            std::ptr::null_mut(),
        );

        if dir < COMPASS_DIRECTIONS {
            (*the_screen.bm_source).Planes[COMPASS_PLANE] = the_memory.nhivar;
            BltBitMap(
                the_screen.bm_source,
                xr,
                yr,
                the_screen.bm_text,
                COMPASS_X + xr,
                COMPASS_Y + yr,
                part.xsize as i32,
                part.ysize as i32,
                BLIT_COPY,
                COMPASS_PLANE_MASK,
                std::ptr::null_mut(),
            );
        }
    }
}

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
const COLOR_COUNT: usize = 32;
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
        page_colors[COLOR_31] = match the_world.region {
            REGION_SWAMP => SWAMP_COLOR,
            REGION_SECRET if the_world.secret_timer != 0 => SECRET_OPEN_COLOR,
            REGION_SECRET => SECRET_SHUT_COLOR,
            _ => DEFAULT_COLOR_31,
        };

        let (mut r, mut g, mut b) = (r as i32, g as i32, b as i32);
        r = r.min(FULL_LEVEL);
        g = g.min(FULL_LEVEL);
        b = b.min(FULL_LEVEL);
        let g2;

        if limit != 0 {
            r = r.max(NIGHT_MIN.0);
            g = g.max(NIGHT_MIN.1);
            b = b.max(NIGHT_MIN.2);
            g2 = (FULL_LEVEL - g) / NIGHT_BLUE_SHARE;
        } else {
            r = r.max(0);
            g = g.max(0);
            b = b.max(0);
            g2 = 0;
        }

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
    let mut g1 = color & GREEN_MASK;
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

// Field widths for printed numbers, and the map-coordinate shift (256 pixels to a sector).
const COORD_DIGITS: i32 = 6;
const SECTOR_DIGITS: i32 = 3;
const EXTENT_DIGITS: i32 = 2;
const VITALITY_DIGITS: i32 = 3;
const STAT_DIGITS: i32 = 3;
const SECTOR_PIXELS: i32 = 256;
const EXTENT_LABEL_LEN: i32 = 10;
const VITALITY_LABEL_LEN: i32 = 4;

// The print queue requests.
const REQUEST_DEBUG_COORDS: i32 = 2;
const REQUEST_DEBUG_SECTOR: i32 = 3;
const REQUEST_VITALITY: i32 = 4;
const REQUEST_OPTIONS: i32 = 5;
const REQUEST_STATS: i32 = 7;
const REQUEST_TAKE_WHAT: i32 = 10;
const QUEUE_MASK: i16 = 31;
const VITALITY_POSITION: (i32, i32) = (245, 52);
const STATS_Y: i32 = 52;
const BRAVERY_X: i32 = 14;
const LUCK_X: i32 = 90;
const KINDNESS_X: i32 = 168;
const WEALTH_X: i32 = 321;

/// Handles one request from the print queue (or waits a tick if there is none).
/// Requests: 2 and 3 debug information, 4 the vitality figure, 5 the menu, 7 the stats line
/// (bravery, luck, kindness, wealth), 10 "Take What?".
pub fn handle_print_request() {
    unsafe {
        if prec == pplay {
            Delay(1);

            return;
        }

        let request = (*(&raw const print_que))[pplay as usize] as i32;
        pplay = (pplay + 1) & QUEUE_MASK;

        match request {
            REQUEST_DEBUG_COORDS => {
                print(c"Coords = ".as_ptr());
                print_number(the_hero.x as i32, COORD_DIGITS);
                print_number(the_hero.y as i32, COORD_DIGITS);
                print(c"Memory Available: ".as_ptr());
                print_number(AvailMem(0) as i32, COORD_DIGITS);
            }

            REQUEST_DEBUG_SECTOR => {
                print(c"You are at: ".as_ptr());
                print_number(the_hero.x as i32 / SECTOR_PIXELS, SECTOR_DIGITS);
                print_number(the_hero.y as i32 / SECTOR_PIXELS, SECTOR_DIGITS);
                print(c"H/Sector = ".as_ptr());
                print_number(the_hero.sector as i32, SECTOR_DIGITS);
                draw_text(c" Extent = ".as_ptr(), EXTENT_LABEL_LEN);
                print_number(the_world.extent_type as i32, EXTENT_DIGITS);
            }

            REQUEST_VITALITY => {
                move_to(VITALITY_POSITION.0, VITALITY_POSITION.1);
                draw_text(c"Vit:".as_ptr(), VITALITY_LABEL_LEN);
                print_number(the_actors.list[0].vitality as i32, VITALITY_DIGITS);
            }

            REQUEST_OPTIONS => {
                print_options();
            }

            REQUEST_STATS => draw_stats(),
            REQUEST_TAKE_WHAT => print(c"Take What?".as_ptr()),
            _ => {}
        }
    }
}

/// Draws the hero's four stats (bravery, luck, kindness, wealth) along the stats line.
unsafe fn draw_stats() {
    unsafe {
        if the_hero.luck < 0 {
            the_hero.luck = 0;
        }

        for (x, label, len, value) in [
            (BRAVERY_X, c"Brv:", 4, the_hero.brave),
            (LUCK_X, c"Lck:", 4, the_hero.luck),
            (KINDNESS_X, c"Knd:", 4, the_hero.kindness),
            (WEALTH_X, c"Wlth:", 5, the_hero.wealth),
        ] {
            move_to(x, STATS_Y);
            draw_text(label.as_ptr(), len);
            print_number(value as i32, STAT_DIGITS);
        }
    }
}

// The view modes of the text panel.
const HIRES: u16 = 0x8000;
const SPRITES: u16 = 0x4000;
const VP_HIDE: u16 = 0x2000;
const MAP_PEN: u32 = 24;
const JAM1: u32 = 0;

/// `view_status` values: showing a map message, and showing the normal view again.
const VIEW_MESSAGE: i8 = 2;
const VIEW_NORMAL_AGAIN: i8 = 3;

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

/// Shows the page without scrolling.
pub fn show_still_screen() {
    unsafe {
        let page = (*the_screen.drawing_page).ri_page;
        (*page).RxOffset = 0;
        (*page).RyOffset = 0;
        flip_pages();
    }
}

/// Switches to the map page for a message (the text panel is hidden) and fades in.
pub fn map_message() {
    unsafe {
        fade_down();
        the_screen.current_rastport = &raw mut the_screen.rp_map;
        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
        SetDrMd(the_screen.current_rastport, JAM1);
        SetAPen(the_screen.current_rastport, MAP_PEN);
        SetRast(the_screen.current_rastport, 0);

        the_screen.vp_text.Modes = HIRES | SPRITES | VP_HIDE;
        show_still_screen();
        LoadRGB4(
            &raw mut the_screen.vp_page,
            (&raw const page_colors).cast(),
            COLOR_COUNT as i32,
        );

        the_screen.view_status = VIEW_MESSAGE;
    }
}

/// Goes back from the map message to the normal view.
pub fn message_off() {
    unsafe {
        fade_down();
        the_screen.current_rastport = &raw mut the_screen.rp_text;
        the_screen.vp_text.Modes = HIRES | SPRITES;
        flip_pages();
        the_screen.view_status = VIEW_NORMAL_AGAIN;
    }
}

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

// The intro zoom: the picture is centred on a 320 x 200 screen and its shape is 8:5.
const SCREEN_CENTER_X: i32 = 160;
const SCREEN_CENTER_Y: i32 = 100;
const ZOOM_ASPECT: (i32, i32) = (5, 8);
const ZOOM_TEXT_HEIGHT: i32 = 95;
const ZOOM_FADE_OFFSETS: (i32, i32, i32) = (40, 70, 100);

/// The size of the picture during the intro's zoom: `x` is half the width in pixels (0 to 160),
/// the height keeps the 8:5 shape. Moves the page and text view ports to match, fades the colours
/// with the size (black when small), and shows the result.
pub fn set_intro_zoom(x: i32) {
    unsafe {
        let y = (x * ZOOM_ASPECT.0) / ZOOM_ASPECT.1;

        Delay(2);

        let left = (SCREEN_CENTER_X - x) as i16;
        the_screen.ri_page2.RxOffset = left;
        the_screen.ri_page1.RxOffset = left;
        the_screen.vp_page.DxOffset = left;
        the_screen.vp_page.DWidth = (x + x) as i16;

        let top = (SCREEN_CENTER_Y - y) as i16;
        the_screen.ri_page2.RyOffset = top;
        the_screen.ri_page1.RyOffset = top;
        the_screen.vp_page.DyOffset = top;
        the_screen.vp_page.DHeight = (y + y) as i16;

        the_screen.vp_text.DxOffset = 0;
        the_screen.vp_text.DyOffset = 0;
        the_screen.vp_text.DHeight = (ZOOM_TEXT_HEIGHT - y) as i16;

        let colors = (&raw const intro_colors).cast::<u16>();
        fade_page(
            (y * 2 - ZOOM_FADE_OFFSETS.0) as i16,
            (y * 2 - ZOOM_FADE_OFFSETS.1) as i16,
            (y * 2 - ZOOM_FADE_OFFSETS.2) as i16,
            0,
            colors,
        );

        MakeVPort(&raw mut main_view, &raw mut the_screen.vp_text);
        flip_pages();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_part_is_four_bytes() {
        assert_eq!(
            size_of::<crate::game::data::object_tables::CompassPart>(),
            4
        );
    }

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
