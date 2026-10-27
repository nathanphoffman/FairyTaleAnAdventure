//! Screen colour effects and panel updates (from csrc/fmain2.c): the compass, fading the page
//! colours with the time of day, the teleport flash, the print queue and the win sequence.
//! The colour tables live in `tables`; the view ports are still defined in csrc/fmain.c.
//!
//! Colours: the Amiga screen has 32 colours, each 12 bits (4 bits red, green, blue). The game
//! fades the whole picture by scaling those components (`fade_page`) and loads them with
//! `LoadRGB4`. Night is a darker, bluer scaling; indoors it is always full brightness.

use super::asm_text::{draw_placard_border, draw_text, move_to, placard_text, print_number};
use super::display::flip_pages;
use super::game_tables::{black_colors, intro_colors};
use super::globals::{
    bm_draw, bm_source, bm_text, current_rastport, drawing_page, nhinor, nhivar, ri_page1,
    ri_page2, rp_map, rp_text, v, view_status, vp_page, vp_text,
};
use super::iffsubs::load_picture;
use super::menus::print_options;
use super::messages::{name, print};
use super::random::bitrand;
use super::state::{the_actors, the_hero, the_world};
use super::tables::{comptable, fader, page_colors, pplay, prec, print_que, sun_colors};
use crate::amiga::{BitMap, RastPort, View, ViewPort};

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
        (*bm_source).Planes[COMPASS_PLANE] = nhinor;
        BltBitMap(
            bm_source,
            0,
            0,
            bm_text,
            COMPASS_X,
            COMPASS_Y,
            COMPASS_WIDTH,
            COMPASS_HEIGHT,
            BLIT_COPY,
            COMPASS_PLANE_MASK,
            std::ptr::null_mut(),
        );
        if dir < COMPASS_DIRECTIONS {
            (*bm_source).Planes[COMPASS_PLANE] = nhivar;
            BltBitMap(
                bm_source,
                xr,
                yr,
                bm_text,
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
            g2 = (FULL_LEVEL - g) / 3;
        } else {
            r = r.max(0);
            g = g.max(0);
            b = b.max(0);
            g2 = 0;
        }
        for i in 0..COLOR_COUNT {
            fader[i] = faded_color(
                i,
                *colors.add(i),
                (r, g, b),
                g2,
                limit != 0,
                the_world.light_timer != 0,
            );
        }
        LoadRGB4(
            &raw mut vp_page,
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
    let mut r1 = (color & 0x0f00) >> 4;
    let mut g1 = color & 0x00f0;
    let mut b1 = color & 0x000f;
    if lit && r1 < g1 {
        r1 = g1;
    }
    r1 = (r * r1) / 1600;
    g1 = (g * g1) / 1600;
    b1 = (b * b1 + g2 * g1) / 100;
    if night {
        if (16..=24).contains(&i) && g > 20 {
            if g < 50 {
                b1 += 2;
            } else if g < 75 {
                b1 += 1;
            }
        }
        b1 = b1.min(MAX_COMPONENT);
    }
    ((r1 << 8) + (g1 << 4) + b1) as u16
}

/// The teleport effect: random colours flicker for a moment (32 flashes).
pub fn flicker_colors() {
    unsafe {
        for _ in 0..COLOR_COUNT {
            for i in 1..COLOR_COUNT {
                fader[i] = bitrand(0xfff) as u16;
            }
            LoadRGB4(
                &raw mut vp_page,
                (&raw const fader).cast(),
                COLOR_COUNT as i32,
            );
            Delay(1);
        }
    }
}

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
                print_number(the_hero.x as i32, 6);
                print_number(the_hero.y as i32, 6);
                print(c"Memory Available: ".as_ptr());
                print_number(AvailMem(0) as i32, 6);
            }
            REQUEST_DEBUG_SECTOR => {
                print(c"You are at: ".as_ptr());
                print_number(the_hero.x as i32 / 256, 3);
                print_number(the_hero.y as i32 / 256, 3);
                print(c"H/Sector = ".as_ptr());
                print_number(the_hero.sector as i32, 3);
                draw_text(c" Extent = ".as_ptr(), 10);
                print_number(the_world.extent_type as i32, 2);
            }
            REQUEST_VITALITY => {
                move_to(VITALITY_POSITION.0, VITALITY_POSITION.1);
                draw_text(c"Vit:".as_ptr(), 4);
                print_number(the_actors.list[0].vitality as i32, 3);
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
            (14, c"Brv:", 4, the_hero.brave),
            (90, c"Lck:", 4, the_hero.luck),
            (168, c"Knd:", 4, the_hero.kindness),
            (321, c"Wlth:", 5, the_hero.wealth),
        ] {
            move_to(x, STATS_Y);
            draw_text(label.as_ptr(), len);
            print_number(value as i32, 3);
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
        let page = (*drawing_page).ri_page;
        (*page).RxOffset = 0;
        (*page).RyOffset = 0;
        flip_pages();
    }
}

/// Switches to the map page for a message (the text panel is hidden) and fades in.
pub fn map_message() {
    unsafe {
        fade_down();
        current_rastport = &raw mut rp_map;
        rp_map.BitMap = (*(*drawing_page).ri_page).BitMap;
        SetDrMd(current_rastport, JAM1);
        SetAPen(current_rastport, MAP_PEN);
        SetRast(current_rastport, 0);

        vp_text.Modes = HIRES | SPRITES | VP_HIDE;
        show_still_screen();
        LoadRGB4(
            &raw mut vp_page,
            (&raw const page_colors).cast(),
            COLOR_COUNT as i32,
        );
        view_status = VIEW_MESSAGE;
    }
}

/// Goes back from the map message to the normal view.
pub fn message_off() {
    unsafe {
        fade_down();
        current_rastport = &raw mut rp_text;
        vp_text.Modes = HIRES | SPRITES;
        flip_pages();
        view_status = VIEW_NORMAL_AGAIN;
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

/// The win screen: shows the picture and brings the sun up behind it by cycling the sky colours
/// through `sun_colors`, with two glowing colours that fade out near the end.
pub fn win_colors() {
    unsafe {
        placard_text(WIN_MESSAGE_FIRST);
        name();
        placard_text(WIN_MESSAGE_SECOND);
        draw_placard_border();
        Delay(80);
        bm_draw = (*(*drawing_page).ri_page).BitMap;
        load_picture(c"winpic".as_ptr(), bm_draw, 0, 0);
        LoadRGB4(
            &raw mut vp_page,
            (&raw const black_colors).cast::<u16>(),
            COLOR_COUNT as i32,
        );
        LoadRGB4(
            &raw mut vp_text,
            (&raw const black_colors).cast::<u16>(),
            COLOR_COUNT as i32,
        );
        vp_text.Modes = HIRES | SPRITES | VP_HIDE;
        set_intro_zoom(WIN_SCREEN_SIZE);
        for i in (SUNRISE_END + 1..=SUNRISE_START).rev() {
            fader = sunrise_palette(i);
            LoadRGB4(
                &raw mut vp_page,
                (&raw const fader).cast(),
                COLOR_COUNT as i32,
            );
            if i == SUNRISE_START {
                Delay(60);
            }
            Delay(9);
        }
        Delay(30);
        LoadRGB4(
            &raw mut vp_page,
            (&raw const black_colors).cast::<u16>(),
            COLOR_COUNT as i32,
        );
    }
}

/// The palette for step `i` of the sunrise over the ending picture. Colours 1 and 28 are white,
/// 0 and 31 black; the sky and sun colours come from `sun_colors` offset by `i`, and the glow
/// (29, 30) is a fixed orange at first and then fades out.
fn sunrise_palette(i: i32) -> [u16; 32] {
    let mut colors = [0u16; 32];
    colors[1] = 0xfff;
    colors[28] = 0xfff;
    for j in SUNRISE_COLORS {
        let index = i + j as i32;
        colors[j] = if index > 0 {
            sun_colors[index as usize] as u16
        } else {
            0
        };
    }
    if i > SUN_GLOW_UNTIL {
        colors[29] = 0x800;
        colors[30] = 0x400;
    } else {
        let j = (i + 30) / 2;
        colors[29] = (0x100 * j) as u16;
        colors[30] = (0x100 * (j / 2)) as u16;
    }
    colors
}

// Daylight: the page colours follow the time of day.
const DAY_DARKEN: (i32, i32, i32) = (80, 61, 62);
const LIGHT_SPELL_BONUS: i32 = 200;
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
        if the_world.day_clock & 3 == 0 || view_status > VIEW_REBUILD {
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
                fade_page(100, 100, 100, 1, colors);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_part_is_four_bytes() {
        assert_eq!(size_of::<super::super::tables::CompassPart>(), 4);
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

/// The size of the picture during the intro's zoom: `x` is half the width in pixels (0 to 160),
/// the height keeps the 8:5 shape. Moves the page and text view ports to match, fades the colours
/// with the size (black when small), and shows the result.
pub fn set_intro_zoom(x: i32) {
    unsafe {
        let y = (x * 5) / 8;

        Delay(2);

        let left = (160 - x) as i16;
        ri_page2.RxOffset = left;
        ri_page1.RxOffset = left;
        vp_page.DxOffset = left;
        vp_page.DWidth = (x + x) as i16;

        let top = (100 - y) as i16;
        ri_page2.RyOffset = top;
        ri_page1.RyOffset = top;
        vp_page.DyOffset = top;
        vp_page.DHeight = (y + y) as i16;

        vp_text.DxOffset = 0;
        vp_text.DyOffset = 0;
        vp_text.DHeight = (95 - y) as i16;

        let colors = (&raw const intro_colors).cast::<u16>();
        fade_page(
            (y * 2 - 40) as i16,
            (y * 2 - 70) as i16,
            (y * 2 - 100) as i16,
            0,
            colors,
        );

        MakeVPort(&raw mut v, &raw mut vp_text);
        flip_pages();
    }
}
