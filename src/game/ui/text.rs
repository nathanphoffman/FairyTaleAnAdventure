//! Text and story routines that were 68000 assembly (fsubs.asm, narr.asm and inline #asm in
//! fmain2.c; from csrc/asm_text.c). The story strings are in `data/story_text.rs`.
//!
//! Two groups of routines live here:
//! - Printing: numbers (`format_number`, `print_number`), strings with position escapes (`print_positioned`), and the lists of
//!   messages and speeches (`show_event`, `speak`, `show_message`, `placard_text`, `question`).
//! - The print queue and the typing cursor (`queue_print`, `cursor`) and the story page border (`draw_placard_border`).
//!
//! Message lists are zero-terminated strings stored one after another; picking message `n` means
//! skipping `n` zero bytes.

use crate::game::data::object_tables::{pplay, prec, print_que};
use crate::game::data::state::the_hero;
use crate::game::data::story_text::{EVENT_MSG, PLACARD_MSGS, SPEECHES};
use crate::game::data::tables::number_buffer;
use crate::game::render::screen::{PortId, screen};
use crate::game::ui::messages::print_wrapped;
use std::ffi::c_char;

/// In a string: followed by x/2 and y, moves the text position.
const XY: u8 = 128;

/// A positioning code in a string is three bytes long: the marker, then x / 2 and y.
const XY_BYTES: usize = 3;
const XY_X_SCALE: i32 = 2;

const DECIMAL_BASE: u32 = 10;

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
            digits[i] = b'0' as c_char + (value % DECIMAL_BASE) as c_char;
            value /= DECIMAL_BASE;

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
        draw_text(
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
pub fn move_to(x: i32, y: i32) {
    let screen = screen();
    screen.move_to(screen.current_port, x, y);
}

/// Prints `len` characters at the text position.
pub fn draw_text(s: *const c_char, len: i32) {
    let bytes = unsafe { std::slice::from_raw_parts(s.cast::<u8>(), len.max(0) as usize) };
    let screen = screen();
    screen.text(screen.current_port, bytes);
}

/// Prints a string that may hold XY escapes (128, x/2, y) to move the text position. A string
/// ends at a zero byte; bytes of 128 and above end each plain run of text.
/// (The x position is stored halved so it fits in a byte.)
pub fn print_positioned(mut s: *const u8) {
    unsafe {
        while *s != 0 {
            if *s == XY {
                move_to(*s.add(1) as i32 * XY_X_SCALE, *s.add(2) as i32);

                s = s.add(XY_BYTES);
            } else {
                let mut n = 0;

                while *s.add(n) != 0 && *s.add(n) < XY {
                    n += 1;
                }

                draw_text(s.cast(), n as i32);
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
    print_positioned(PLACARD_MSGS[n as usize].as_ptr());
}

/// The `n`-th zero-terminated string of a list of messages (the first is number 0). A number past
/// the end gives the final zero byte, an empty string.
fn nth_string(list: &[u8], n: i32) -> *const c_char {
    let mut at = 0;

    for _ in 0..n {
        let length = list[at..].iter().position(|&b| b == 0).unwrap_or(0);
        at = (at + length + 1).min(list.len() - 1);
    }

    list[at..].as_ptr().cast()
}

/// Shows the `num`-th string of a list of messages, word-wrapped (see `print_wrapped`).
pub fn show_message(list: &[u8], num: i32) {
    print_wrapped(nth_string(list, num));
}

/// Shows event message `n` (things that happen to the hero, such as hunger).
pub fn show_event(n: i32) {
    show_message(EVENT_MSG, n);
}

/// Shows speech number `n` (what people say).
pub fn speak(n: i32) {
    show_message(SPEECHES, n);
}

/// Adds a request to the print queue (ignored if the queue is full). `handle_print_request` handles them one
/// per tick, which spreads the redrawing work over several frames.
pub fn queue_print(x: i32) {
    unsafe {
        // Masks with PRINT_QUEUE_MASK, keeping only the bits that matter here.
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
const BORDER_STEPS: usize = 16;

/// The page the border goes around is 284 x 124 (the long edges) and 268 x 112 (the short ones,
/// drawn turned a quarter), with the short edges offset by 16 and 12.
const LONG_EDGE_FAR: (i32, i32) = (284, 124);
const SHORT_EDGE_NEAR: (i32, i32) = (16, 12);
const SHORT_EDGE_FAR: (i32, i32) = (268, 112);

/// Passes numbered above this also draw the long (top and bottom) edges.
const LONG_EDGE_ABOVE_PASS: i32 = 9;
const BORDER_PEN: u8 = 1;
const BORDER_HIGHLIGHT_PEN: u8 = 24;

/// Strokes per step; the last one is drawn in the highlight pen.
const STROKES_PER_STEP: i32 = 5;

/// Draws a line from (x0, y0) to (x1, y1).
fn line(x0: i32, y0: i32, x1: i32, y1: i32) {
    let screen = screen();
    screen.move_to(PortId::Map, x0, y0);
    screen.line_to(PortId::Map, x1, y1);
}

/// Draws the decorative border around a story page: many short strokes, five each with the last in
/// a highlight colour, winding around the four edges. The first passes also draw the top and bottom.
pub fn draw_placard_border() {
    let (mut xorg, mut yorg) = (BORDER_START_X, 0);

    for pass in (0..BORDER_PASSES).rev() {
        for step in 0..BORDER_STEPS {
            let dx = xorg + BORDER_X_STEPS[step];
            let dy = yorg + BORDER_Y_STEPS[step];

            for stroke in (0..STROKES_PER_STEP).rev() {
                let pen = if stroke == 0 {
                    BORDER_HIGHLIGHT_PEN
                } else {
                    BORDER_PEN
                };

                screen().set_a_pen(PortId::Map, pen);
                draw_border_stroke(pass > LONG_EDGE_ABOVE_PASS, (xorg, yorg), (dx, dy));
            }

            xorg = dx;
            yorg = dy;
        }
    }
}

/// One stroke of the border from `from` to `to`, repeated on the opposite edge, and on the short
/// edges (turned a quarter) with the x and y swapped.
fn draw_border_stroke(long_edges: bool, (xorg, yorg): (i32, i32), (dx, dy): (i32, i32)) {
    if long_edges {
        line(xorg, yorg, dx, dy);
        line(
            LONG_EDGE_FAR.0 - xorg,
            LONG_EDGE_FAR.1 - yorg,
            LONG_EDGE_FAR.0 - dx,
            LONG_EDGE_FAR.1 - dy,
        );
    }

    line(
        SHORT_EDGE_NEAR.0 + yorg,
        SHORT_EDGE_NEAR.1 - xorg,
        SHORT_EDGE_NEAR.0 + dy,
        SHORT_EDGE_NEAR.1 - dx,
    );

    line(
        SHORT_EDGE_FAR.0 - yorg,
        SHORT_EDGE_FAR.1 + xorg,
        SHORT_EDGE_FAR.0 - dy,
        SHORT_EDGE_FAR.1 + dx,
    );
}

/// Menu states: an item the player has, and one they lack.
const MENU_STATE_HAVE: i32 = 10;
const MENU_STATE_LACKING: i32 = 8;

/// Menu state for an item: 10 if the player has item `n`, otherwise 8 (a menu item state).
pub fn item_menu_state(n: i32) -> i32 {
    if unsafe { *the_hero.stuff.offset(n as isize) } != 0 {
        MENU_STATE_HAVE
    } else {
        MENU_STATE_LACKING
    }
}

const ERROR_SCREEN_X: i32 = 200;
const ERROR_SCREEN_Y: i32 = 90;

/// Error number meaning graphics are not open, so nothing can be printed.
const ERROR_NO_GRAPHICS: i32 = 2;
const ERROR_LABEL_LEN: i32 = 6;

/// Shows "ERROR n" on the text panel (startup failures). Error 2 (no graphics) cannot be shown.
pub fn show_error(n: i32) {
    if n == ERROR_NO_GRAPHICS {
        return;
    }

    move_to(ERROR_SCREEN_X, ERROR_SCREEN_Y);
    draw_text(c"ERROR ".as_ptr(), ERROR_LABEL_LEN);
    print_number(n, 2);
}
