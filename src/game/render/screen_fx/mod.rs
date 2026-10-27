//! Screen colour effects and panel updates (from csrc/fmain2.c): the compass, fading the page
//! colours with the time of day, the teleport flash, the print queue and the win sequence.
//! The colour tables live in `tables`; the view ports are still defined in csrc/fmain.c.
//!
//! Colours: the Amiga screen has 32 colours, each 12 bits (4 bits red, green, blue). The game
//! fades the whole picture by scaling those components (`fade_page`) and loads them with
//! `LoadRGB4`. Night is a darker, bluer scaling; indoors it is always full brightness.

mod compass;

mod fading;

mod message_page;

mod print_queue;

mod win;

mod zoom;

pub use self::compass::draw_compass;
pub use self::fading::{day_fade, fade_down, fade_normal, fade_page, flicker_colors};
pub use self::message_page::{map_message, message_off, show_still_screen};
pub use self::print_queue::handle_print_request;
pub use self::win::win_colors;
pub use self::zoom::set_intro_zoom;

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

const COLOR_COUNT: usize = 32;

// The view modes of the text panel.
const HIRES: u16 = 0x8000;
const SPRITES: u16 = 0x4000;
const VP_HIDE: u16 = 0x2000;
