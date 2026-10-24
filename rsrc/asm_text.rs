//! Text, story and enemy-steering routines that were 68000 assembly (fsubs.asm, narr.asm and
//! inline #asm in fmain2.c; from csrc/asm_text.c). The story strings are in csrc/narr_data.c.

use super::random::fta_rand;
use crate::amiga::RastPort;
use std::ffi::{c_char, c_void};

/// Mirror of `struct shape` in csrc/ftale.h: one actor.
#[repr(C)]
struct Shape {
    abs_x: u16,
    abs_y: u16,
    rel_x: u16,
    rel_y: u16,
    kind: i8,
    race: u8,
    index: i8,
    visible: i8,
    weapon: i8,
    environ: i8,
    goal: i8,
    tactic: i8,
    state: i8,
    facing: i8,
    vitality: i16,
    vel_x: i8,
    vel_y: i8,
}

unsafe extern "C" {
    static mut rp: *mut RastPort;
    static mut rp_map: RastPort;
    static mut anim_list: [Shape; 20];
    static stuff: *const u8;
    static answr: [c_char; 10];
    static xx: i16;
    static yy: i16;
    static mut print_que: [c_char; 32];
    static mut prec: i16;
    static pplay: i16;
    static mut numbuf: [c_char; 11];
    static event_msg: [c_char; 0];
    static speeches: [c_char; 0];
    static placard_msgs: [*const c_char; 0];
    static question_msgs: [*const c_char; 0];

    fn Move(rp: *mut RastPort, x: i32, y: i32);
    fn Draw(rp: *mut RastPort, x: i32, y: i32);
    fn Text(rp: *mut RastPort, s: *const c_char, len: i32) -> i32;
    fn SetAPen(rp: *mut RastPort, pen: u32);
    fn SetBPen(rp: *mut RastPort, pen: u32);
    fn print_cont(s: *const c_void) -> i32;
    fn extract(s: *const c_char);
}

/// In a string: followed by x/2 and y, moves the text position.
const XY: u8 = 128;
/// Digits in the number buffer; one trailing space follows them.
const NUMBER_DIGITS: usize = 10;
/// Size of the print queue; the indexes wrap with this mask.
const PRINT_QUEUE_MASK: i16 = 31;

// Actor states (csrc/ftale.h).
const WALKING: i8 = 12;
const STILL: i8 = 13;

/// Right-aligned decimal in the first 10 characters of `numbuf`, leading positions blank.
#[unsafe(no_mangle)]
pub extern "C" fn ion(mut value: u32) -> i32 {
    unsafe {
        let digits = &mut (&mut *(&raw mut numbuf))[..NUMBER_DIGITS];
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
#[unsafe(no_mangle)]
pub extern "C" fn prdec(value: i32, len: i32) {
    ion(value as u32);
    unsafe { Text(rp, (&raw const numbuf).cast::<c_char>().add(NUMBER_DIGITS).offset(-(len as isize)), len + 1) };
}

// `move` is a Rust keyword, so this one is exported under the C name.
#[unsafe(export_name = "move")]
pub extern "C" fn move_to(x: i32, y: i32) {
    unsafe { Move(rp, x, y) };
}

#[unsafe(no_mangle)]
pub extern "C" fn text(s: *const c_char, len: i32) {
    unsafe { Text(rp, s, len) };
}

/// Prints a string that may hold XY escapes (128, x/2, y) to move the text position. A string
/// ends at a zero byte; bytes of 128 and above end each plain run of text.
#[unsafe(no_mangle)]
pub extern "C" fn ssp(mut s: *const u8) {
    unsafe {
        while *s != 0 {
            if *s == XY {
                Move(rp, *s.add(1) as i32 * 2, *s.add(2) as i32);
                s = s.add(3);
            } else {
                let mut n = 0;
                while *s.add(n) != 0 && *s.add(n) < XY {
                    n += 1;
                }
                Text(rp, s.cast(), n as i32);
                s = s.add(n);
                if n == 0 {
                    s = s.add(1); // stray byte >= 128 that is not XY
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn placard_text(n: i32) {
    unsafe { ssp((*(&raw const placard_msgs).cast::<*const c_char>().offset(n as isize)).cast()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn question(n: i32) {
    unsafe { print_cont((*(&raw const question_msgs).cast::<*const c_char>().offset(n as isize)).cast()) };
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

/// Shows the `num`-th string of a list of messages.
#[unsafe(no_mangle)]
pub extern "C" fn msg(start: *const c_char, num: i32) {
    unsafe { extract(skip_strings(start, num)) };
}

#[unsafe(no_mangle)]
pub extern "C" fn event(n: i32) {
    msg((&raw const event_msg).cast(), n);
}

#[unsafe(no_mangle)]
pub extern "C" fn speak(n: i32) {
    msg((&raw const speeches).cast(), n);
}

/// Adds a request to the print queue (ignored if the queue is full).
#[unsafe(no_mangle)]
pub extern "C" fn prq(x: i32) {
    unsafe {
        let next = (prec + 1) & PRINT_QUEUE_MASK;
        if next == pplay {
            return;
        }
        (*(&raw mut print_que))[prec as usize] = x as c_char;
        prec = next;
    }
}

/// Redraws the typed answer and a block cursor after it.
#[unsafe(no_mangle)]
pub extern "C" fn cursor(len: i32, pen: i32) {
    let space = b" ".as_ptr().cast::<c_char>();
    unsafe {
        SetBPen(rp, 0);
        Move(rp, xx as i32, yy as i32);
        Text(rp, answr.as_ptr(), len);
        SetBPen(rp, pen as u32);
        Text(rp, space, 1);
        SetBPen(rp, 0);
        Text(rp, space, 1);
    }
}

// The decorative border drawn on story pages.
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

unsafe fn line(r: *mut RastPort, x0: i32, y0: i32, x1: i32, y1: i32) {
    unsafe {
        Move(r, x0, y0);
        Draw(r, x1, y1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn placard() {
    unsafe {
        let r = &raw mut rp_map;
        let (mut xorg, mut yorg) = (BORDER_START_X, 0);
        for pass in (0..BORDER_PASSES).rev() {
            for step in 0..16 {
                let dx = xorg + BORDER_X_STEPS[step];
                let dy = yorg + BORDER_Y_STEPS[step];
                for stroke in (0..STROKES_PER_STEP).rev() {
                    SetAPen(r, if stroke == 0 { BORDER_HIGHLIGHT_PEN } else { BORDER_PEN });
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

/// Menu state for an item: 10 if the player has item `n`, otherwise 8.
#[unsafe(no_mangle)]
pub extern "C" fn stuff_flag(n: i32) -> i32 {
    if unsafe { *stuff.offset(n as isize) } != 0 { 10 } else { 8 }
}

/// Direction number for each of the 9 combinations of horizontal and vertical steering.
const DIRECTION_TABLE: [u8; 9] = [0, 1, 2, 7, 9, 3, 6, 5, 4];
const NO_DIRECTION: i32 = 9;

/// What the steering decides for an actor.
#[derive(Debug, PartialEq)]
enum Course {
    Stop,
    Face { facing: i32 },
}

/// The sign of a difference as -1, 0 or 1.
fn sign(v: i16) -> i32 {
    (v as i32).signum()
}

/// Works out the direction toward (or away from) a target. `xdif`/`ydif` are actor minus target
/// (or the offset itself in mode 6). Modes: 0 toward, 1 and 2 toward with a sideways wobble when
/// close, 3 away, 4 straight toward (no smart steering), 5 turn only, 6 the offset is given.
fn steer(xdif: i16, ydif: i16, mode: i32, random_bit: impl FnOnce() -> bool) -> Course {
    let xabs = (xdif as i32).abs() as u16;
    let yabs = (ydif as i32).abs() as u16;
    let (mut xdir, mut ydir) = (sign(xdif), sign(ydif));

    if mode != 4 {
        if (yabs as i16) < (xabs >> 1) as i16 {
            ydir = 0;
        }
        if (xabs as i16) < (yabs >> 1) as i16 {
            xdir = 0;
        }
    }

    let close = (xabs as i32 + yabs as i32) as i16;
    let mut deviation = 0;
    if mode == 1 && close < 40 {
        deviation = 1;
    }
    if mode == 2 && close < 30 {
        deviation = 1;
    }
    if mode == 3 {
        xdir = -xdir;
        ydir = -ydir;
    }

    let mut facing = DIRECTION_TABLE[(4 - ydir * 3 - xdir) as usize] as i32;
    if facing == NO_DIRECTION {
        return Course::Stop;
    }
    facing += if random_bit() { deviation } else { -deviation };
    Course::Face { facing: facing & 7 }
}

/// Points an actor toward (or away from) a target and sets it walking (see `steer` for modes).
#[unsafe(no_mangle)]
pub extern "C" fn set_course(object: i32, target_x: i32, target_y: i32, mode: i32) {
    let mode = mode & 0xff;
    unsafe {
        let actor = &mut (*(&raw mut anim_list))[object as usize];
        let (xdif, ydif) = if mode == 6 {
            (target_x as i16, target_y as i16)
        } else {
            ((actor.abs_x as i32 - target_x) as i16, (actor.abs_y as i32 - target_y) as i16)
        };
        match steer(xdif, ydif, mode, || fta_rand() & 2 != 0) {
            Course::Stop => actor.state = STILL,
            Course::Face { facing } => {
                actor.facing = facing as i8;
                if mode != 5 {
                    actor.state = WALKING;
                }
            }
        }
    }
}

const ERROR_SCREEN_X: i32 = 200;
const ERROR_SCREEN_Y: i32 = 90;
/// Error number meaning graphics are not open, so nothing can be printed.
const ERROR_NO_GRAPHICS: i32 = 2;

/// Shows "ERROR n" on the text panel (startup failures).
#[unsafe(no_mangle)]
pub extern "C" fn do_error(n: i32) {
    if n == ERROR_NO_GRAPHICS {
        return;
    }
    move_to(ERROR_SCREEN_X, ERROR_SCREEN_Y);
    text(c"ERROR ".as_ptr(), 6);
    prdec(n, 2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_actor_matches_the_c_struct() {
        assert_eq!(size_of::<Shape>(), 22);
    }

    #[test]
    fn steers_toward_a_target() {
        // actor is right of and level with the target: direction 7
        assert_eq!(steer(100, 0, 0, || false), Course::Face { facing: 7 });
        // below the target: direction 1
        assert_eq!(steer(0, 100, 0, || false), Course::Face { facing: 1 });
        // exactly on the target: stop
        assert_eq!(steer(0, 0, 0, || false), Course::Stop);
    }

    #[test]
    fn steering_away_reverses_direction() {
        assert_eq!(steer(100, 0, 3, || false), Course::Face { facing: 3 });
    }

    #[test]
    fn close_targets_wobble_in_modes_1_and_2() {
        assert_eq!(steer(10, 0, 1, || true), Course::Face { facing: 0 });
        assert_eq!(steer(10, 0, 1, || false), Course::Face { facing: 6 });
        // 35 is close enough for mode 1 (under 40) but not for mode 2 (under 30)
        assert_eq!(steer(35, 0, 1, || true), Course::Face { facing: 0 });
        assert_eq!(steer(35, 0, 2, || true), Course::Face { facing: 7 });
    }

    #[test]
    fn diagonal_needs_both_axes_to_matter() {
        // a 3:1 slope is treated as straight along x
        assert_eq!(steer(90, 30, 0, || false), Course::Face { facing: 7 });
        assert_eq!(steer(60, 60, 0, || false), Course::Face { facing: 0 });
    }

    #[test]
    fn the_random_bit_is_only_used_when_the_actor_moves() {
        let mut asked = false;
        steer(0, 0, 0, || {
            asked = true;
            true
        });
        assert!(!asked);
    }
}
