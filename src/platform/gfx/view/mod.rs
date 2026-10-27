//! Views (what is on screen), the no-op blitter calls and sprites.

use crate::amiga::*;
use std::os::raw::c_void;

mod palette;

pub fn init_view(v: *mut View) {
    unsafe { std::ptr::write_bytes(v, 0, 1) }
}

pub fn init_vport(vp: *mut ViewPort) {
    unsafe { std::ptr::write_bytes(vp, 0, 1) }
}

pub fn make_vport(_v: *mut View, _vp: *mut ViewPort) -> LONG {
    0
}

pub fn merge_copper(_v: *mut View) -> LONG {
    0
}

pub fn free_vport_cop_lists(_vp: *mut ViewPort) {}

pub fn free_cpr_list(_c: *mut c_void) {}

/// Makes this view the one on screen. The game calls this to flip between its two pages, so
/// it is also where a new frame is shown.
pub fn load_view(v: *mut View) {
    crate::platform::display::set_view(v);
    crate::platform::display::present();
}

pub fn wait_bovp(_vp: *mut ViewPort) {
    crate::platform::display::wait_frame();
}

// ---- blitter (no-ops, we are not racing hardware) ----

pub fn own_blitter() {}
pub fn disown_blitter() {}
pub fn wait_blit() {}

// ---- sprites (the mouse pointer; drawn by the display later) ----

pub fn get_sprite(_s: *mut SimpleSprite, _n: LONG) -> LONG {
    0
}

pub fn free_sprite(_n: LONG) {}
pub fn change_sprite(_vp: *mut ViewPort, _s: *mut SimpleSprite, data: *mut c_void) {
    crate::platform::display::set_sprite(data as *const u32);
}

pub use self::palette::{free_color_map, get_color_map, load_rgb4, set_rgb4};
