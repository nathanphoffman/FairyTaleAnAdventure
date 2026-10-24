//! Views (what is on screen), the no-op blitter calls and sprites.

use crate::amiga::*;
use std::os::raw::c_void;

mod palette;

#[unsafe(no_mangle)]
pub extern "C" fn InitView(v: *mut View) {
    unsafe { std::ptr::write_bytes(v, 0, 1) }
}

#[unsafe(no_mangle)]
pub extern "C" fn InitVPort(vp: *mut ViewPort) {
    unsafe { std::ptr::write_bytes(vp, 0, 1) }
}

#[unsafe(no_mangle)]
pub extern "C" fn MakeVPort(_v: *mut View, _vp: *mut ViewPort) -> LONG {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn MrgCop(_v: *mut View) -> LONG {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn FreeVPortCopLists(_vp: *mut ViewPort) {}

#[unsafe(no_mangle)]
pub extern "C" fn FreeCprList(_c: *mut c_void) {}

/// Makes this view the one on screen. The game calls this to flip between its two pages, so
/// it is also where a new frame is shown.
#[unsafe(no_mangle)]
pub extern "C" fn LoadView(v: *mut View) {
    crate::platform::display::set_view(v);
    crate::platform::display::present();
}

#[unsafe(no_mangle)]
pub extern "C" fn WaitBOVP(_vp: *mut ViewPort) {
    crate::platform::display::wait_frame();
}

#[unsafe(no_mangle)]
pub extern "C" fn WaitTOF() {
    crate::platform::display::wait_frame();
}

// ---- blitter (no-ops, we are not racing hardware) ----

#[unsafe(no_mangle)]
pub extern "C" fn OwnBlitter() {}
#[unsafe(no_mangle)]
pub extern "C" fn DisownBlitter() {}
#[unsafe(no_mangle)]
pub extern "C" fn WaitBlit() {}

// ---- sprites (the mouse pointer; drawn by the display later) ----

#[unsafe(no_mangle)]
pub extern "C" fn GetSprite(_s: *mut SimpleSprite, _n: LONG) -> LONG {
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn FreeSprite(_n: LONG) {}
#[unsafe(no_mangle)]
pub extern "C" fn ChangeSprite(_vp: *mut ViewPort, _s: *mut SimpleSprite, data: *mut c_void) {
    crate::platform::display::set_sprite(data as *const u32);
}
#[unsafe(no_mangle)]
pub extern "C" fn MoveSprite(_vp: *mut ViewPort, _s: *mut SimpleSprite, _x: LONG, _y: LONG) {}
