//! Printing messages in the text panel (from csrc/fmain2.c): scrolling a line in, continuing it,
//! word-wrapping story text and the "found ..." announcements.
//!
//! The text panel at the bottom of the screen shows the last few lines of messages. `print` scrolls
//! the panel up one line and writes a new line at the bottom; `print_cont` adds more text to the
//! line that is already there. `extract` is the general routine: it takes a message string from the
//! story tables, breaks it into lines that fit the panel and prints each one.

use super::asm_text::{move_to, text};
use crate::amiga::RastPort;
use std::ffi::{CStr, c_char};
use super::game_tables::{datanames};
use super::globals::{brother, rp};

unsafe extern "C" {
    fn ScrollRaster(rp: *mut RastPort, dx: i32, dy: i32, x0: i32, y0: i32, x1: i32, y1: i32);
}

// The scrolling message area of the text panel.
const TEXT_X_MIN: i32 = 16;
const TEXT_X_MAX: i32 = 400;
const TEXT_Y_MIN: i32 = 5;
const TEXT_Y_MAX: i32 = 44;
/// Pixels the panel scrolls up for each new line, and the baseline of that new line.
const LINE_HEIGHT: i32 = 10;
const NEW_LINE_Y: i32 = 42;

/// Characters on a line before `extract` looks for a place to wrap it.
const WRAP_COLUMN: i16 = 37;
/// Past this count `extract` knows the message is finished.
const END_COLUMN: i16 = 38;
/// Set when the message text ends, to leave the line loop.
const MESSAGE_ENDED: i16 = 1000;
const CARRIAGE_RETURN: c_char = 13;
/// In message text: replaced by the current brother's name.
const NAME_MARK: c_char = b'%' as c_char;

pub static mut mbuf: *mut c_char = std::ptr::null_mut();
pub static mut mesbuf: [c_char; 200] = [0; 200];

/// Length of a C string.
unsafe fn length(s: *const c_char) -> i32 {
    unsafe { CStr::from_ptr(s).to_bytes().len() as i32 }
}

/// The current brother's name.
unsafe fn brother_name() -> *const c_char {
    unsafe { datanames[brother as usize - 1] }
}

/// Starts a new line in the panel and prints `s` on it.
/// The old lines scroll up by one text line (10 pixels) inside the message area.
pub fn print(s: *const c_char) {
    unsafe {
        ScrollRaster(
            rp,
            0,
            LINE_HEIGHT,
            TEXT_X_MIN,
            TEXT_Y_MIN,
            TEXT_X_MAX,
            TEXT_Y_MAX,
        );
        move_to(TEXT_X_MIN, NEW_LINE_Y);
        text(s, length(s));
    }
}

/// Continues the current line with `s`.
pub fn print_cont(s: *const c_char) {
    unsafe { text(s, length(s)) };
}

/// Prints message text, wrapping it into lines at spaces. A carriage return (13) starts a new
/// line and a `%` is replaced by the current brother's name.
///
/// How it works: characters are copied into `mesbuf`, remembering where the last space was. When
/// a line reaches 37 characters it is cut at that space (the space becomes the end of the string)
/// and printed, and the leftover characters start the next line. The message ends at a zero byte.
pub fn extract(mut start: *const c_char) {
    unsafe {
        let mut line_start = (&raw mut mesbuf).cast::<c_char>();
        mbuf = line_start;
        let mut i: i16 = 0;
        loop {
            let mut line_break: *mut c_char = std::ptr::null_mut(); // end of the current line
            while i < WRAP_COLUMN {
                let c = *start;
                start = start.add(1);
                if c == b' ' as c_char {
                    line_break = mbuf;
                }
                if c == 0 {
                    line_break = mbuf;
                    *line_break = 0;
                    i = MESSAGE_ENDED;
                }
                if c == CARRIAGE_RETURN {
                    line_break = mbuf;
                    *line_break = 0;
                    break;
                }
                if c == NAME_MARK {
                    let mut name = brother_name();
                    while *name != 0 {
                        *mbuf = *name;
                        mbuf = mbuf.add(1);
                        name = name.add(1);
                        i += 1;
                    }
                } else {
                    *mbuf = c;
                    mbuf = mbuf.add(1);
                }
                i += 1;
            }
            if !line_break.is_null() {
                // found a wrap point: end the line and print it
                *line_break = 0;
                line_break = line_break.add(1);
                print(line_start);
                if i > END_COLUMN {
                    break;
                }
                i = mbuf.offset_from(line_break) as i16;
                line_start = line_break;
            } else {
                // no wrap point, so print the whole thing
                *mbuf = 0;
                mbuf = mbuf.add(1);
                if i > END_COLUMN {
                    break;
                }
                print(line_start);
                i = 0;
                line_start = mbuf;
            }
        }
    }
}

/// Prints "<Brother> found <container> containing ", ready for the contents to be added.
pub fn announce_container(s: *const c_char) {
    unsafe {
        print(brother_name());
        print_cont(c" found ".as_ptr());
        print_cont(s);
        print_cont(c" containing ".as_ptr());
    }
}

/// Prints "<Brother> found <treasure>".
pub fn announce_treasure(s: *const c_char) {
    unsafe {
        print(brother_name());
        print_cont(c" found ".as_ptr());
        print_cont(s);
    }
}

/// Adds the current brother's name to the line being printed.
pub fn name() {
    unsafe { print_cont(brother_name()) };
}
