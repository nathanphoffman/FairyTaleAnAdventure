//! The few Amiga-only symbols the game still names (from csrc/asm_stubs.c).
//!
//! - `trapper`: the 68000 trap handler hook; nothing to do on Linux
//! - `HandlerInterface`: the Amiga input handler; `input` replaces it
//! - `WBenchMsg`: Workbench start message; the game is never started from Workbench
//!
//! They exist only so the C code that mentions them still links.

use std::ffi::c_void;

#[unsafe(no_mangle)]
pub extern "C" fn trapper() -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn HandlerInterface() -> i32 {
    0
}

#[unsafe(no_mangle)]
pub static mut WBenchMsg: *mut c_void = std::ptr::null_mut();
