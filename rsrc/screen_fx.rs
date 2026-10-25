//! Screen colour effects and panel updates (from csrc/fmain2.c): the compass, fading the page
//! colours with the time of day, the teleport flash, the print queue and the win sequence.
//! The colour tables live in `tables`; the view ports are still defined in csrc/fmain.c.
//!
//! Colours: the Amiga screen has 32 colours, each 12 bits (4 bits red, green, blue). The game
//! fades the whole picture by scaling those components (`fade_page`) and loads them with
//! `LoadRGB4`. Night is a darker, bluer scaling; indoors it is always full brightness.

use super::asm_text::{move_to, placard, placard_text, prdec, text};
use super::iffsubs::unpackbrush;
use super::messages::{name, print};
use super::random::bitrand;
use super::tables::{comptable, fader, pagecolors, pplay, prec, print_que, sun_colors};
use super::types::{FPage, Shape};
use crate::amiga::{BitMap, RastPort, ViewPort};

unsafe extern "C" {
    static blackcolors: [i16; 32];
    static mut vp_page: ViewPort;
    static mut vp_text: ViewPort;
    static mut rp: *mut RastPort;
    static mut rp_map: RastPort;
    static mut rp_text: RastPort;
    static fp_drawing: *mut FPage;
    static mut bm_draw: *mut BitMap;
    static bm_text: *mut BitMap;
    static bm_source: *mut BitMap;
    static nhinor: *mut u8;
    static nhivar: *mut u8;
    static mut viewstatus: i8;
    static region_num: u16;
    static secret_timer: i16;
    static light_timer: i16;
    static daynight: u16;
    static lightlevel: u16;
    static anim_list: [Shape; 20];
    static hero_x: u16;
    static hero_y: u16;
    static hero_sector: i16;
    static xtype: u16;
    static mut luck: i16;
    static brave: i16;
    #[link_name = "kind"]
    static hero_kind: i16;
    static wealth: i16;

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
    fn LoadRGB4(vp: *mut ViewPort, colors: *const u16, count: i32);
    fn Delay(ticks: i32);
    fn SetDrMd(rp: *mut RastPort, mode: u32);
    fn SetAPen(rp: *mut RastPort, pen: u32);
    fn SetRast(rp: *mut RastPort, pen: u32);
    fn AvailMem(flags: u32) -> u32;
    fn print_options() -> i32;
    fn pagechange();
    fn screen_size(x: i32) -> i32;
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
#[unsafe(no_mangle)]
pub extern "C" fn drawcompass(dir: i16) {
    unsafe {
        let part = &comptable[dir as usize];
        let (xr, yr) = (part.xrect as i32, part.yrect as i32);
        (*bm_source).Planes[COMPASS_PLANE] = nhinor;
        BltBitMap(bm_source, 0, 0, bm_text, COMPASS_X, COMPASS_Y, COMPASS_WIDTH, COMPASS_HEIGHT, BLIT_COPY, COMPASS_PLANE_MASK, std::ptr::null_mut());
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
#[unsafe(no_mangle)]
pub extern "C" fn fade_page(r: i16, g: i16, b: i16, limit: i16, colors: *const u16) {
    unsafe {
        pagecolors[COLOR_31] = match region_num {
            REGION_SWAMP => SWAMP_COLOR,
            REGION_SECRET if secret_timer != 0 => SECRET_OPEN_COLOR,
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
            let color = *colors.add(i) as i32;
            let mut r1 = (color & 0x0f00) >> 4;
            let mut g1 = color & 0x00f0;
            let mut b1 = color & 0x000f;
            if light_timer != 0 && r1 < g1 {
                r1 = g1;
            }
            r1 = (r * r1) / 1600;
            g1 = (g * g1) / 1600;
            b1 = (b * b1 + g2 * g1) / 100;
            if limit != 0 {
                if (16..=24).contains(&i) && g > 20 {
                    if g < 50 {
                        b1 += 2;
                    } else if g < 75 {
                        b1 += 1;
                    }
                }
                b1 = b1.min(MAX_COMPONENT);
            }
            fader[i] = ((r1 << 8) + (g1 << 4) + b1) as u16;
        }
        LoadRGB4(&raw mut vp_page, (&raw const fader).cast(), COLOR_COUNT as i32);
    }
}

/// The teleport effect: random colours flicker for a moment (32 flashes).
#[unsafe(no_mangle)]
pub extern "C" fn colorplay() {
    unsafe {
        for _ in 0..COLOR_COUNT {
            for i in 1..COLOR_COUNT {
                fader[i] = bitrand(0xfff) as u16;
            }
            LoadRGB4(&raw mut vp_page, (&raw const fader).cast(), COLOR_COUNT as i32);
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
#[unsafe(no_mangle)]
pub extern "C" fn ppick() {
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
                prdec(hero_x as i32, 6);
                prdec(hero_y as i32, 6);
                print(c"Memory Available: ".as_ptr());
                prdec(AvailMem(0) as i32, 6);
            }
            REQUEST_DEBUG_SECTOR => {
                print(c"You are at: ".as_ptr());
                prdec(hero_x as i32 / 256, 3);
                prdec(hero_y as i32 / 256, 3);
                print(c"H/Sector = ".as_ptr());
                prdec(hero_sector as i32, 3);
                text(c" Extent = ".as_ptr(), 10);
                prdec(xtype as i32, 2);
            }
            REQUEST_VITALITY => {
                move_to(VITALITY_POSITION.0, VITALITY_POSITION.1);
                text(c"Vit:".as_ptr(), 4);
                prdec(anim_list[0].vitality as i32, 3);
            }
            REQUEST_OPTIONS => {
                print_options();
            }
            REQUEST_STATS => {
                if luck < 0 {
                    luck = 0;
                }
                for (x, label, len, value) in [
                    (14, c"Brv:", 4, brave),
                    (90, c"Lck:", 4, luck),
                    (168, c"Knd:", 4, hero_kind),
                    (321, c"Wlth:", 5, wealth),
                ] {
                    move_to(x, STATS_Y);
                    text(label.as_ptr(), len);
                    prdec(value as i32, 3);
                }
            }
            REQUEST_TAKE_WHAT => print(c"Take What?".as_ptr()),
            _ => {}
        }
    }
}

// The view modes of the text panel.
const HIRES: u16 = 0x8000;
const SPRITES: u16 = 0x4000;
const VP_HIDE: u16 = 0x2000;
const MAP_PEN: u32 = 24;
const JAM1: u32 = 0;
/// `viewstatus` values: showing a map message, and showing the normal view again.
const VIEW_MESSAGE: i8 = 2;
const VIEW_NORMAL_AGAIN: i8 = 3;
/// Fading goes in steps of 5 percent, one tick each.
const FADE_STEP: usize = 5;

/// Fades the page out to black.
#[unsafe(no_mangle)]
pub extern "C" fn fade_down() {
    for level in (0..=FULL_LEVEL).rev().step_by(FADE_STEP) {
        fade_page(level as i16, level as i16, level as i16, 0, (&raw const pagecolors).cast());
        unsafe { Delay(1) };
    }
}

/// Fades the page back in from black.
#[unsafe(no_mangle)]
pub extern "C" fn fade_normal() {
    for level in (0..=FULL_LEVEL).step_by(FADE_STEP) {
        fade_page(level as i16, level as i16, level as i16, 0, (&raw const pagecolors).cast());
        unsafe { Delay(1) };
    }
}

/// Shows the page without scrolling.
#[unsafe(no_mangle)]
pub extern "C" fn stillscreen() {
    unsafe {
        let page = (*fp_drawing).ri_page;
        (*page).RxOffset = 0;
        (*page).RyOffset = 0;
        pagechange();
    }
}

/// Switches to the map page for a message (the text panel is hidden) and fades in.
#[unsafe(no_mangle)]
pub extern "C" fn map_message() {
    unsafe {
        fade_down();
        rp = &raw mut rp_map;
        rp_map.BitMap = (*(*fp_drawing).ri_page).BitMap;
        SetDrMd(rp, JAM1);
        SetAPen(rp, MAP_PEN);
        SetRast(rp, 0);

        vp_text.Modes = HIRES | SPRITES | VP_HIDE;
        stillscreen();
        LoadRGB4(&raw mut vp_page, (&raw const pagecolors).cast(), COLOR_COUNT as i32);
        viewstatus = VIEW_MESSAGE;
    }
}

/// Goes back from the map message to the normal view.
#[unsafe(no_mangle)]
pub extern "C" fn message_off() {
    unsafe {
        fade_down();
        rp = &raw mut rp_text;
        vp_text.Modes = HIRES | SPRITES;
        pagechange();
        viewstatus = VIEW_NORMAL_AGAIN;
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
#[unsafe(no_mangle)]
pub extern "C" fn win_colors() {
    unsafe {
        placard_text(WIN_MESSAGE_FIRST);
        name();
        placard_text(WIN_MESSAGE_SECOND);
        placard();
        Delay(80);
        bm_draw = (*(*fp_drawing).ri_page).BitMap;
        unpackbrush(c"winpic".as_ptr(), bm_draw, 0, 0);
        LoadRGB4(&raw mut vp_page, blackcolors.as_ptr().cast(), COLOR_COUNT as i32);
        LoadRGB4(&raw mut vp_text, blackcolors.as_ptr().cast(), COLOR_COUNT as i32);
        vp_text.Modes = HIRES | SPRITES | VP_HIDE;
        screen_size(WIN_SCREEN_SIZE);
        for i in (SUNRISE_END + 1..=SUNRISE_START).rev() {
            fader[0] = 0;
            fader[31] = 0;
            fader[1] = 0xfff;
            fader[28] = 0xfff;
            for j in SUNRISE_COLORS {
                let index = i + j as i32;
                fader[j] = if index > 0 { sun_colors[index as usize] as u16 } else { 0 };
            }
            if i > SUN_GLOW_UNTIL {
                fader[29] = 0x800;
                fader[30] = 0x400;
            } else {
                let j = (i + 30) / 2;
                fader[29] = (0x100 * j) as u16;
                fader[30] = (0x100 * (j / 2)) as u16;
            }
            LoadRGB4(&raw mut vp_page, (&raw const fader).cast(), COLOR_COUNT as i32);
            if i == SUNRISE_START {
                Delay(60);
            }
            Delay(9);
        }
        Delay(30);
        LoadRGB4(&raw mut vp_page, blackcolors.as_ptr().cast(), COLOR_COUNT as i32);
    }
}

// Daylight: the page colours follow the time of day.
const DAY_DARKEN: (i32, i32, i32) = (80, 61, 62);
const LIGHT_SPELL_BONUS: i32 = 200;
/// Inside buildings (regions from here on) it is never night.
const FIRST_INDOOR_REGION: u16 = 8;
/// `viewstatus` above this means the screen has to be rebuilt.
const VIEW_REBUILD: i8 = 97;

/// Updates the page colours for the time of day, every fourth tick. `lightlevel` follows the sun;
/// inside buildings (regions 8 and 9) it is always bright.
#[unsafe(no_mangle)]
pub extern "C" fn day_fade() {
    unsafe {
        let bonus = if light_timer != 0 { LIGHT_SPELL_BONUS } else { 0 };
        if daynight & 3 == 0 || viewstatus > VIEW_REBUILD {
            let colors = (&raw const pagecolors).cast();
            if region_num < FIRST_INDOOR_REGION {
                let level = lightlevel as i32;
                fade_page((level - DAY_DARKEN.0 + bonus) as i16, (level - DAY_DARKEN.1) as i16, (level - DAY_DARKEN.2) as i16, 1, colors);
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
}
