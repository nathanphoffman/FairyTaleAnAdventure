//! Libraries and the current task.

use crate::amiga::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr::{addr_of_mut, null_mut};

/// Stands in for every library except graphics.
const DUMMY_LIB_BYTES: usize = 64;
const GRAPHICS_LIBRARY: &str = "graphics.library";

static mut GFX_BASE: GfxBase = GfxBase {
    ActiView: null_mut(),
    DisplayFlags: 0,
};
static mut DUMMY_LIB: [u8; DUMMY_LIB_BYTES] = [0; DUMMY_LIB_BYTES];
static mut THIS_TASK: Process = Process {
    pr_Task: Task {
        tc_Node_pri: 0,
        tc_TrapCode: null_mut(),
    },
    pr_WindowPtr: null_mut(),
};

#[unsafe(no_mangle)]
pub extern "C" fn OpenLibrary(name: *const c_char, _version: ULONG) -> *mut c_void {
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    {
        if name == GRAPHICS_LIBRARY {
            addr_of_mut!(GFX_BASE) as *mut c_void
        } else {
            addr_of_mut!(DUMMY_LIB) as *mut c_void
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn CloseLibrary(_lib: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn FindTask(_name: *const c_char) -> *mut Process {
    addr_of_mut!(THIS_TASK)
}
