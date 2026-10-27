//! Text and story routines that were 68000 assembly (fsubs.asm, narr.asm and inline #asm in
//! fmain2.c; from csrc/asm_text.c). The story strings are in csrc/narr_data.c.
//!
//! Two groups of routines live here:
//! - Printing: numbers (`format_number`, `print_number`), strings with position escapes (`print_positioned`), and the lists of
//!   messages and speeches (`show_event`, `speak`, `show_message`, `placard_text`, `question`).
//! - The print queue and the typing cursor (`queue_print`, `cursor`) and the story page border (`draw_placard_border`).
//!
//! Message lists are zero-terminated strings stored one after another; picking message `n` means
//! skipping `n` zero bytes.

use crate::amiga::RastPort;
use crate::game::data::object_tables::{pplay, prec, print_que};
use crate::game::data::state::{the_hero, the_screen};
use crate::game::data::tables::number_buffer;
use crate::game::ui::messages::print_wrapped;
use std::ffi::c_char;

unsafe extern "C" {
    static event_msg: [c_char; 0];
    static speeches: [c_char; 0];
    static placard_msgs: [*const c_char; 0];

    fn Move(current_rastport: *mut RastPort, x: i32, y: i32);
    fn Draw(current_rastport: *mut RastPort, x: i32, y: i32);
    fn Text(current_rastport: *mut RastPort, s: *const c_char, len: i32) -> i32;
    fn SetAPen(current_rastport: *mut RastPort, pen: u32);
}

/// In a string: followed by x/2 and y, moves the text position.
const XY: u8 = 128;
/// Digits in the number buffer; one trailing space follows them.
const NUMBER_DIGITS: usize = 10;
/// Size of the print queue; the indexes wrap with this mask.
const PRINT_QUEUE_MASK: i16 = 31;

/// Right-aligned decimal in the first 10 characters of `number_buffer`, leading positions blank.
/// (The 11th character is a space, which `print_number` prints after the number.)
pub fn format_number(mut value: u32) -> i32 {
    unsafe {
        let digits = &mut (&mut *(&raw mut number_buffer))[..NUMBER_DIGITS];
        let mut i = NUMBER_DIGITS;
        loop {
            i -= 1;
            digits[i] = b'0' as c_char + (value % 10) as c_char;
            value /= 10;
            if value == 0 || i == 0 {
                break;
            }
        }
        digits[..i].fill(b' ' as c_char);
    }
    0
}

/// Prints the number in a field of `len` characters, plus the trailing space.
pub fn print_number(value: i32, len: i32) {
    format_number(value as u32);
    unsafe {
        Text(
            the_screen.current_rastport,
            (&raw const number_buffer)
                .cast::<c_char>()
                .add(NUMBER_DIGITS)
                .offset(-(len as isize)),
            len + 1,
        )
    };
}

// `move` is a Rust keyword, so this one is exported under the C name.
/// Moves the text position (the C name is `move`, a keyword in Rust).
pub extern "C" fn move_to(x: i32, y: i32) {
    unsafe { Move(the_screen.current_rastport, x, y) };
}

/// Prints `len` characters at the text position.
pub fn draw_text(s: *const c_char, len: i32) {
    unsafe { Text(the_screen.current_rastport, s, len) };
}

/// Prints a string that may hold XY escapes (128, x/2, y) to move the text position. A string
/// ends at a zero byte; bytes of 128 and above end each plain run of text.
/// (The x position is stored halved so it fits in a byte.)
pub fn print_positioned(mut s: *const u8) {
    unsafe {
        while *s != 0 {
            if *s == XY {
                Move(
                    the_screen.current_rastport,
                    *s.add(1) as i32 * 2,
                    *s.add(2) as i32,
                );
                s = s.add(3);
            } else {
                let mut n = 0;
                while *s.add(n) != 0 && *s.add(n) < XY {
                    n += 1;
                }
                Text(the_screen.current_rastport, s.cast(), n as i32);
                s = s.add(n);
                if n == 0 {
                    s = s.add(1); // stray byte >= 128 that is not XY
                }
            }
        }
    }
}

/// Prints story page text number `n` (the strings use position escapes, see `print_positioned`).
pub fn placard_text(n: i32) {
    unsafe {
        print_positioned(
            (*(&raw const placard_msgs)
                .cast::<*const c_char>()
                .offset(n as isize))
            .cast(),
        )
    };
}

/// Skips `n` zero-terminated strings.
unsafe fn skip_strings(mut p: *const c_char, mut n: i32) -> *const c_char {
    unsafe {
        while n > 0 {
            while *p != 0 {
                p = p.add(1);
            }
            p = p.add(1);
            n -= 1;
        }
    }
    p
}

/// Shows the `num`-th string of a list of messages, word-wrapped (see `print_wrapped`).
pub fn show_message(start: *const c_char, num: i32) {
    unsafe { print_wrapped(skip_strings(start, num)) };
}

/// Shows event message `n` (things that happen to the hero, such as hunger).
pub fn show_event(n: i32) {
    show_message((&raw const event_msg).cast(), n);
}

/// Shows speech number `n` (what people say).
pub fn speak(n: i32) {
    show_message((&raw const speeches).cast(), n);
}

/// Adds a request to the print queue (ignored if the queue is full). `handle_print_request` handles them one
/// per tick, which spreads the redrawing work over several frames.
pub fn queue_print(x: i32) {
    unsafe {
        let next = (prec + 1) & PRINT_QUEUE_MASK;
        if next == pplay {
            return;
        }
        (*(&raw mut print_que))[prec as usize] = x as c_char;
        prec = next;
    }
}

// The decorative border drawn on story pages.
// The border is a closed path made of 16 small steps repeated with changing offsets.
const BORDER_X_STEPS: [i32; 16] = [-4, -4, -4, 0, 0, 0, 4, 4, 0, -4, 0, 4, 4, 0, 0, 0];
const BORDER_Y_STEPS: [i32; 16] = [0, 0, 0, 4, 4, 4, 0, 0, -4, 0, -4, 0, 0, 4, 4, 4];
const BORDER_START_X: i32 = 12;
const BORDER_PASSES: i32 = 17;
/// Passes numbered above this also draw the long (top and bottom) edges.
const LONG_EDGE_ABOVE_PASS: i32 = 9;
const BORDER_PEN: u32 = 1;
const BORDER_HIGHLIGHT_PEN: u32 = 24;
/// Strokes per step; the last one is drawn in the highlight pen.
const STROKES_PER_STEP: i32 = 5;

/// Draws a line from (x0, y0) to (x1, y1).
unsafe fn line(r: *mut RastPort, x0: i32, y0: i32, x1: i32, y1: i32) {
    unsafe {
        Move(r, x0, y0);
        Draw(r, x1, y1);
    }
}

/// Draws the decorative border around a story page: many short strokes, five each with the last in
/// a highlight colour, winding around the four edges. The first passes also draw the top and bottom.
pub fn draw_placard_border() {
    unsafe {
        let r = &raw mut the_screen.rp_map;
        let (mut xorg, mut yorg) = (BORDER_START_X, 0);
        for pass in (0..BORDER_PASSES).rev() {
            for step in 0..16 {
                let dx = xorg + BORDER_X_STEPS[step];
                let dy = yorg + BORDER_Y_STEPS[step];
                for stroke in (0..STROKES_PER_STEP).rev() {
                    SetAPen(
                        r,
                        if stroke == 0 {
                            BORDER_HIGHLIGHT_PEN
                        } else {
                            BORDER_PEN
                        },
                    );
                    if pass > LONG_EDGE_ABOVE_PASS {
                        line(r, xorg, yorg, dx, dy);
                        line(r, 284 - xorg, 124 - yorg, 284 - dx, 124 - dy);
                    }
                    line(r, 16 + yorg, 12 - xorg, 16 + dy, 12 - dx);
                    line(r, 268 - yorg, 112 + xorg, 268 - dy, 112 + dx);
                }
                xorg = dx;
                yorg = dy;
            }
        }
    }
}

/// Menu state for an item: 10 if the player has item `n`, otherwise 8 (a menu item state).
pub fn item_menu_state(n: i32) -> i32 {
    if unsafe { *the_hero.stuff.offset(n as isize) } != 0 {
        10
    } else {
        8
    }
}

const ERROR_SCREEN_X: i32 = 200;
const ERROR_SCREEN_Y: i32 = 90;
/// Error number meaning graphics are not open, so nothing can be printed.
const ERROR_NO_GRAPHICS: i32 = 2;

/// Shows "ERROR n" on the text panel (startup failures). Error 2 (no graphics) cannot be shown.
pub fn show_error(n: i32) {
    if n == ERROR_NO_GRAPHICS {
        return;
    }
    move_to(ERROR_SCREEN_X, ERROR_SCREEN_Y);
    draw_text(c"ERROR ".as_ptr(), 6);
    print_number(n, 2);
}
