//! Showing the finished page and the input device (from csrc/fmain.c).
//!
//! The game draws on one of two pages while the other is on screen (`drawing_page`, `viewing_page`),
//! then swaps them. The Amiga calls here are mostly stand-ins in the platform layer.

use crate::amiga::{View, ViewPort};
use crate::game::data::globals::main_view;
use crate::game::data::state::the_screen;
use crate::game::input::handler::reset_key_buffer;
use std::ffi::{c_char, c_void};

unsafe extern "C" {
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

/// Empties the key buffer (the hero is being revived, so old key presses are dropped).
pub fn reset_input() {
    reset_key_buffer();
}

const INPUT_DEVICE: &[u8] = b"input.device\0";

/// The port and request used to talk to the input device.
static mut INPUT_PORT: *mut c_void = std::ptr::null_mut();
static mut INPUT_REQUEST: *mut c_void = std::ptr::null_mut();

/// Swaps the page being drawn with the one being shown, and shows the one just finished.
pub fn flip_pages() {
    unsafe {
        let finished = the_screen.drawing_page;
        std::ptr::swap(
            &raw mut the_screen.drawing_page,
            &raw mut the_screen.viewing_page,
        );

        the_screen.vp_page.RasInfo = (*finished).ri_page;
        main_view.LOFCprList = (*finished).savecop;

        MakeVPort(&raw mut main_view, &raw mut the_screen.vp_page);
        MrgCop(&raw mut main_view);
        LoadView(&raw mut main_view);
        (*finished).savecop = main_view.LOFCprList;

        WaitBOVP(&raw mut the_screen.vp_text);
    }
}

/// Sets up the input device: empties the key buffer and opens the device with the game's handler.
/// Returns 1 if it worked and 0 if not. The handler itself is the Rust code in `input.rs`, so only
/// the device requests are made here.
pub fn install_input_handler() -> i32 {
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
pub fn remove_input_handler() {
    unsafe {
        DoIO(INPUT_REQUEST);
        CloseDevice(INPUT_REQUEST);
        DeleteStdIO(INPUT_REQUEST);
        DeletePort(INPUT_PORT);
    }
}
