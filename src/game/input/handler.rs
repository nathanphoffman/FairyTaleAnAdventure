//! Keyboard and mouse input, replacing the original input handler in fsubs.asm (HandlerInterface,
//! getkey, decode_mouse; from csrc/input.c). The platform layer (src/platform/input) translates
//! SDL events and calls the `fta_*` functions below.
//!
//! How input reaches the game:
//! - Keys go into a 128-byte ring buffer (`keybuf`). The game's main loop pops them with `next_key`.
//!   A key code with bit 7 set means the key was released.
//! - The mouse pointer position is kept inside the text panel, where the menus and the compass are.
//!   A left click on the menu column is turned into a menu key press ('a' for the first row, 'b' for
//!   the second column of the first row, and so on).
//! - `read_walk_direction` works out which way the hero walks, from the mouse over the compass or from the
//!   direction keys, and redraws the compass.
//!
//! Directions are numbered 0-7 clockwise starting at north-west: 0 NW, 1 N, 2 NE, 3 E, 4 SE, 5 S,
//! 6 SW, 7 W. The value 9 means "not walking".

use crate::game::data::state::Ui;
use crate::game::output::Output;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// What the keyboard and the mouse did since the game last looked.
struct Handler {
    /// Pointer position, in lores pixels (x) and screen lines (y).
    pointer: (i16, i16),
    /// Which mouse buttons are down (bit 0x4000 left, 0x2000 right).
    qualifier: i16,
    /// Keys pressed and released, oldest first (`next_key` takes the oldest).
    keys: VecDeque<u8>,
    /// The last menu key sent, so its release can be sent with the same code.
    last_menu: i8,
}

static HANDLER: Mutex<Handler> = Mutex::new(Handler {
    pointer: (POINTER_START, POINTER_START),
    qualifier: 0,
    keys: VecDeque::new(),
    last_menu: 0,
});

/// The pointer starts off the play field.
const POINTER_START: i16 = 320;

/// Whether the joystick fire button is held.
static JOYSTICK_FIRE: AtomicBool = AtomicBool::new(false);

fn handler() -> std::sync::MutexGuard<'static, Handler> {
    HANDLER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The joystick fire button was pressed or released.
pub fn set_joystick_fire(down: bool) {
    JOYSTICK_FIRE.store(down, Ordering::Relaxed);
}

/// Whether the joystick fire button is held.
pub fn joystick_fire() -> bool {
    JOYSTICK_FIRE.load(Ordering::Relaxed)
}

/// The most keys the buffer holds (it was a ring of 128 bytes with one slot always empty).
const KEY_BUFFER_CAPACITY: usize = 127;
const KEY_RELEASED: i32 = 0x80;

// Qualifier bits for the mouse buttons.
const LEFT_BUTTON: i16 = 0x4000;
const RIGHT_BUTTON: i16 = 0x2000;

// The pointer lives in the text panel, so it is kept inside these bounds.
const POINTER_MIN_X: i32 = 5;
const POINTER_MAX_X: i32 = 315;
const POINTER_MIN_Y: i32 = 147;
const POINTER_MAX_Y: i32 = 195;

// The menu column: x range, first row's y, row height, and where the right-hand column starts.
const MENU_MIN_X: i32 = 215;
const MENU_MAX_X: i32 = 265;
const MENU_RIGHT_COLUMN_X: i32 = 240;
const MENU_TOP_Y: i32 = 144;
const MENU_ROW_HEIGHT: i32 = 9;
const MENU_FIRST_KEY: i32 = 0x61; // 'a'

// The on-screen compass.
const COMPASS_MIN_X: i32 = 265;
const COMPASS_LEFT_END_X: i32 = 292;
const COMPASS_CENTER_END_X: i32 = 300;
const COMPASS_LOWER_Y: i32 = 166;
const COMPASS_MIDDLE_END_Y: i32 = 174;

/// Direction value meaning "not walking".
const DIR_NONE: i16 = 9;

/// Keyboard direction keys are numbered from this value (keypad digits, offset by 20).
const KEYDIR_BASE: i16 = 20;
const DIRECTION_COUNT: i16 = 10;

/// Appends a key to the buffer; the key is dropped if the buffer is full.
fn push_key(handler: &mut Handler, code: i32) {
    if handler.keys.len() < KEY_BUFFER_CAPACITY {
        handler.keys.push_back(code as u8);
    }
}

/// Key event from the platform layer. `code` is the game's key code, bit 7 set = key released.
pub fn key_event(code: i32) {
    push_key(&mut handler(), code);
}

/// Empties the key buffer (the hero is being revived, so old key presses are dropped).
pub fn reset_key_buffer() {
    handler().keys.clear();
}

/// The mouse buttons held (bit 0x4000 left, 0x2000 right).
pub fn mouse_buttons() -> i16 {
    handler().qualifier
}

/// Forgets the last menu key sent, so its release is not sent (used when a menu is switched).
pub fn forget_last_menu() {
    handler().last_menu = 0;
}

/// Pops the oldest key code from the buffer, or 0 if there is none.
pub fn next_key() -> i32 {
    handler().keys.pop_front().map_or(0, |code| code as i32)
}

/// Pointer position in lores pixels (x 0-319) and screen lines (y 0-199).
/// The pointer is kept inside the text panel (the bottom of the screen), where the menus and
/// the compass are, because that is the only place the mouse is used.
pub fn mouse_moved(x: i32, y: i32) {
    handler().pointer = (
        x.clamp(POINTER_MIN_X, POINTER_MAX_X) as i16,
        y.clamp(POINTER_MIN_Y, POINTER_MAX_Y) as i16,
    );
}

/// The menu key under the pointer, if any: 'a' plus twice the row number, plus one for the
/// right-hand column. The menu is two columns of rows 9 lines high, starting at line 144.
fn menu_key_at(x: i32, y: i32) -> Option<i32> {
    if !(MENU_MIN_X..=MENU_MAX_X).contains(&x) || y < MENU_TOP_Y {
        return None;
    }

    let row = (y - MENU_TOP_Y) / MENU_ROW_HEIGHT;
    let right = (x >= MENU_RIGHT_COLUMN_X) as i32;

    Some(row * 2 + MENU_FIRST_KEY + right)
}

/// Mouse button change. A left click in the menu column becomes a menu key on press and the same
/// key with bit 7 set on release, as if it had been typed. The right button only sets a flag
/// (it makes the hero fight, see `read_walk_direction`).
pub fn mouse_buttons_changed(left: bool, right: bool) {
    let new_qualifier =
        (if left { LEFT_BUTTON } else { 0 }) | (if right { RIGHT_BUTTON } else { 0 });
    let handler = &mut handler();

    // Xor with the old state leaves only the bits that changed: the left button was pressed or released.
    if (new_qualifier ^ handler.qualifier) & LEFT_BUTTON != 0 {
        let key = if new_qualifier & LEFT_BUTTON != 0 {
            menu_key_at(handler.pointer.0 as i32, handler.pointer.1 as i32)
        } else if handler.last_menu != 0 {
            let released = (handler.last_menu as u8 as i32) | KEY_RELEASED;
            handler.last_menu = 0;

            Some(released)
        } else {
            None
        };

        if let Some(key) = key {
            push_key(handler, key);

            // as in the original: the last key sent is remembered, release included
            handler.last_menu = key as i8;
        }
    }

    handler.qualifier = new_qualifier;
}

/// Walking direction (0-7, 9 = none) for a pointer position over the on-screen compass.
///
/// The compass is split into three columns and three rows; the middle square means no direction.
/// Left of the compass nothing is selected.
fn compass_direction(x: i32, y: i32) -> i16 {
    if x <= COMPASS_MIN_X {
        return DIR_NONE;
    }

    let (up, center, down) = if x < COMPASS_LEFT_END_X {
        (0, 7, 6)
    } else if x <= COMPASS_CENTER_END_X {
        (1, DIR_NONE, 5)
    } else {
        (2, 3, 4)
    };

    if y < COMPASS_LOWER_Y {
        up
    } else if y <= COMPASS_MIDDLE_END_Y {
        center
    } else {
        down
    }
}

/// Decides the walking direction from the mouse over the compass or from the keyboard direction
/// keys, and redraws the compass when it changes.
///
/// While a mouse button is down the pointer position decides; otherwise the direction key
/// (numbered from 20, the keypad digits) does. Anything outside 20-29 means no direction.
pub fn read_walk_direction(ui: &mut Ui, out: &mut dyn Output) {
    let (qualifier, pointer) = {
        let handler = handler();

        (handler.qualifier, handler.pointer)
    };

    let dir = if qualifier & (LEFT_BUTTON | RIGHT_BUTTON) != 0 {
        compass_direction(pointer.0 as i32, pointer.1 as i32)
    } else {
        let dir = ui.held_direction.wrapping_sub(KEYDIR_BASE);

        if (0..DIRECTION_COUNT).contains(&dir) {
            dir
        } else {
            ui.held_direction = 0;

            DIR_NONE
        }
    };

    if dir != ui.compass_lit_direction {
        ui.compass_lit_direction = dir;
        out.draw_compass(dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_regions() {
        assert_eq!(compass_direction(200, 150), 9);
        assert_eq!(compass_direction(280, 150), 0);
        assert_eq!(compass_direction(280, 170), 7);
        assert_eq!(compass_direction(280, 190), 6);
        assert_eq!(compass_direction(296, 150), 1);
        assert_eq!(compass_direction(296, 170), 9);
        assert_eq!(compass_direction(296, 190), 5);
        assert_eq!(compass_direction(310, 150), 2);
        assert_eq!(compass_direction(310, 170), 3);
        assert_eq!(compass_direction(310, 190), 4);
    }

    #[test]
    fn menu_keys() {
        assert_eq!(menu_key_at(225, 148), Some(0x61));
        assert_eq!(menu_key_at(250, 148), Some(0x62));
        assert_eq!(menu_key_at(225, 166), Some(0x65));
        assert_eq!(menu_key_at(100, 148), None);
        assert_eq!(menu_key_at(225, 100), None);
    }
}
