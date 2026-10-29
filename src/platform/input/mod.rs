//! Keyboard and mouse. SDL events become the key codes and mouse state the original input
//! handler (fsubs.asm HandlerInterface) produced; the logic that fills the game's buffers lives
//! in csrc/input.c.
//!
//! Key codes are the ones the game's `keytrans` table gave: upper-case letters, digits and
//! symbols as ASCII, F1-F10 = 10-19, keypad digits = direction codes 20-29, bit 7 = released.

mod config;

mod keys;

mod pad;

mod record;

pub use config::load_config;
pub use pad::{pad_button, pad_direction};
pub use record::TICK;

use keys::dir_code;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::cell::Cell;
use std::os::raw::c_int;

use crate::platform::host::host;

/// Added to a key code to say the key was released.
const RELEASED: c_int = 0x80;

/// Offset of the gamepad inputs in `ARROWS`.
const PAD_ARROWS: usize = 4;

/// Lores pixels are doubled horizontally in the 640 wide frame.
const FRAME_TO_LORES: i32 = 2;

/// Where the pointer can go, in lores pixels / screen lines (the game's play field and panel).
const POINTER_X_RANGE: (i32, i32) = (5, 315);
const POINTER_Y_RANGE: (i32, i32) = (147, 195);
const POINTER_START: (i32, i32) = (160, 170);

thread_local! {
    /// up, down, left, right from the keyboard (0-3) and the gamepad (4-7)
    static ARROWS: Cell<[bool; 8]> = const { Cell::new([false; 8]) };

    /// direction key code currently held down (20-27), 0 = none
    static DIR: Cell<u8> = const { Cell::new(0) };
    static BUTTONS: Cell<(bool, bool)> = const { Cell::new((false, false)) };

    /// pointer position in lores pixels / screen lines, as the game sees it
    static POINTER: Cell<(i32, i32)> = const { Cell::new(POINTER_START) };
}

/// Sends a key press or release to the game.
fn send_key(code: u8, down: bool) {
    let flag = if down { 0 } else { RELEASED };

    if let Some(host) = host() {
        host.key_event(code as c_int | flag);
    }
}

/// Presses or releases the joystick fire button.
fn set_fire(down: bool) {
    if let Some(host) = host() {
        host.set_joystick_fire(down);
    }
}

/// Sets one direction input (0-3 keyboard up/down/left/right, 4-7 the same on the gamepad) and
/// sends the matching keypad direction key to the game.
fn direction(i: usize, down: bool) {
    let mut a = ARROWS.with(|c| c.get());
    a[i] = down;
    ARROWS.with(|c| c.set(a));

    // arrows are turned into the keypad direction keys, so two arrows give a diagonal
    let new = dir_code(a[0] || a[4], a[1] || a[5], a[2] || a[6], a[3] || a[7]);
    let old = DIR.with(|c| c.replace(new));

    if new != old {
        if old != 0 {
            send_key(old, false);
        }

        if new != 0 {
            send_key(new, true);
        }
    }
}

/// Which of the four direction inputs an arrow key is.
fn arrow_index(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Up => Some(0),
        Keycode::Down => Some(1),
        Keycode::Left => Some(2),
        Keycode::Right => Some(3),
        _ => None,
    }
}

/// A key was pressed or released: records it, applies the key remapping, and passes it on to the
/// game as an arrow or a key code.
pub fn key_event(key: Keycode, down: bool) {
    record::record(format_args!(
        "key {} {}",
        key.name().replace(' ', "_"),
        down as u8
    ));

    let key = config::remapped(key);

    if let Some(i) = arrow_index(key) {
        direction(i, down);
    } else if key == Keycode::LCtrl || key == Keycode::RCtrl {
        set_fire(down);
    } else {
        let code = keys::translate(key);

        if code != 0 {
            send_key(code, down);
        }
    }
}

/// `frame_x`, `frame_y` are positions in the 640x200 frame.
pub fn mouse_move(frame_x: i32, frame_y: i32) {
    record::record(format_args!("move {frame_x} {frame_y}"));
    let (x, y) = (frame_x / FRAME_TO_LORES, frame_y);

    if let Some(host) = host() {
        host.mouse_moved(x, y);
    }
    let clamped = (
        x.clamp(POINTER_X_RANGE.0, POINTER_X_RANGE.1),
        y.clamp(POINTER_Y_RANGE.0, POINTER_Y_RANGE.1),
    );

    POINTER.with(|p| p.set(clamped));
}

/// A mouse button was pressed or released: records it and tells the game the state of both buttons.
pub fn mouse_button(button: MouseButton, down: bool) {
    let (mut l, mut r) = BUTTONS.with(|b| b.get());

    match button {
        MouseButton::Left => l = down,
        MouseButton::Right => r = down,
        _ => return,
    }

    let name = if button == MouseButton::Left {
        "left"
    } else {
        "right"
    };

    record::record(format_args!("button {name} {}", down as u8));
    BUTTONS.with(|b| b.set((l, r)));

    if let Some(host) = host() {
        host.mouse_buttons_changed(l, r);
    }
}

/// Pointer position for drawing the cursor.
pub fn pointer() -> (i32, i32) {
    POINTER.with(|p| p.get())
}
