//! Palette: the 12 bit colour table of a viewport.

use crate::amiga::*;
use std::os::raw::c_void;

pub fn get_color_map(n: LONG) -> *mut ColorMap {
    unsafe {
        let cm = libc::calloc(1, std::mem::size_of::<ColorMap>()) as *mut ColorMap;
        (*cm).Count = n as UWORD;
        (*cm).ColorTable = libc::calloc(n.max(1) as usize, 2) as *mut UWORD;

        cm
    }
}

pub fn free_color_map(cm: *mut ColorMap) {
    if !cm.is_null() {
        unsafe {
            libc::free((*cm).ColorTable as *mut c_void);
            libc::free(cm as *mut c_void);
        }
    }
}

/// A colour channel is 4 bits, and the red and green ones sit 8 and 4 bits up.
const COMPONENT_MASK: ULONG = 15;
const RED_SHIFT: u32 = 8;
const GREEN_SHIFT: u32 = 4;

/// Colours are 12 bit: 0x0RGB with 4 bits per channel.
pub fn set_rgb4(vp: *mut ViewPort, n: LONG, r: ULONG, g: ULONG, b: ULONG) {
    let cm = unsafe { (*vp).ColorMap };

    if cm.is_null() || n < 0 || n >= unsafe { (*cm).Count } as i32 {
        return;
    }

    unsafe {
        // A 12 bit colour: four bits each of red, green and blue.
        *(*cm).ColorTable.add(n as usize) = ((r & COMPONENT_MASK) << RED_SHIFT
            // Shifts by GREEN_SHIFT, converting between pixels, tiles, sectors or bytes.
            | (g & COMPONENT_MASK) << GREEN_SHIFT
            // Masks with COMPONENT_MASK, keeping only the bits that matter here.
            | (b & COMPONENT_MASK)) as UWORD
    };
}

pub fn load_rgb4(vp: *mut ViewPort, colors: *const UWORD, count: LONG) {
    let cm = unsafe { (*vp).ColorMap };

    if cm.is_null() {
        return;
    }

    let n = count.min(unsafe { (*cm).Count } as i32).max(0) as usize;

    unsafe { std::ptr::copy_nonoverlapping(colors, (*cm).ColorTable, n) };
}
