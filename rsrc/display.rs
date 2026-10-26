//! Showing the finished page and the input device (from csrc/fmain.c).
//!
//! The game draws on one of two pages while the other is on screen (`fp_drawing`, `fp_viewing`),
//! then swaps them. The Amiga calls here are mostly stand-ins in the platform layer.

use super::input::reset_key_buffer;
use super::types::FPage;
use crate::amiga::{View, ViewPort};
use std::ffi::{c_char, c_void};

unsafe extern "C" {
    static mut v: View;
    static mut vp_page: ViewPort;
    static mut vp_text: ViewPort;
    static mut fp_drawing: *mut FPage;
    static mut fp_viewing: *mut FPage;

    fn MakeVPort(view: *mut View, port: *mut ViewPort) -> i32;
    fn MrgCop(view: *mut View) -> i32;
    fn LoadView(view: *mut View);
    fn WaitBOVP(port: *mut ViewPort);
    fn CreatePort(name: *const c_char, pri: i32) -> *mut c_void;
    fn DeletePort(port: *mut c_void);
    fn CreateStdIO(port: *mut c_void) -> *mut c_void;
    fn DeleteStdIO(io: *mut c_void);
    fn OpenDevice(name: *const c_char, unit: u32, io: *mut c_void, flags: u32) -> i32;
    fn CloseDevice(io: *mut c_void);
    fn DoIO(io: *mut c_void) -> i32;
}

const INPUT_DEVICE: &[u8] = b"input.device\0";

/// The port and request used to talk to the input device.
static mut INPUT_PORT: *mut c_void = std::ptr::null_mut();
static mut INPUT_REQUEST: *mut c_void = std::ptr::null_mut();

/// Swaps the page being drawn with the one being shown, and shows the one just finished.
#[unsafe(no_mangle)]
pub extern "C" fn pagechange() {
    unsafe {
        let finished = fp_drawing;
        fp_drawing = fp_viewing;
        fp_viewing = finished;
        vp_page.RasInfo = (*finished).ri_page;
        v.LOFCprList = (*finished).savecop;

        MakeVPort(&raw mut v, &raw mut vp_page);
        MrgCop(&raw mut v);
        LoadView(&raw mut v);
        (*finished).savecop = v.LOFCprList;

        WaitBOVP(&raw mut vp_text);
    }
}

/// Sets up the input device: empties the key buffer and opens the device with the game's handler.
/// Returns 1 if it worked and 0 if not. The handler itself is the Rust code in `input.rs`, so only
/// the device requests are made here.
#[unsafe(no_mangle)]
pub extern "C" fn add_device() -> i32 {
    unsafe {
        reset_key_buffer();
        INPUT_PORT = CreatePort(std::ptr::null(), 0);
        if INPUT_PORT.is_null() {
            return 0;
        }
        INPUT_REQUEST = CreateStdIO(INPUT_PORT);
        if INPUT_REQUEST.is_null() {
            DeletePort(INPUT_PORT);
            return 0;
        }
        // the original also filled in a handler (priority 51) and asked for it to be added
        let error = OpenDevice(INPUT_DEVICE.as_ptr().cast::<c_char>(), 0, INPUT_REQUEST, 0) as i16;
        if error != 0 {
            return 0;
        }
        DoIO(INPUT_REQUEST);
        1
    }
}

/// Takes the input device down again.
#[unsafe(no_mangle)]
pub extern "C" fn wrap_device() {
    unsafe {
        DoIO(INPUT_REQUEST);
        CloseDevice(INPUT_REQUEST);
        DeleteStdIO(INPUT_REQUEST);
        DeletePort(INPUT_PORT);
    }
}
