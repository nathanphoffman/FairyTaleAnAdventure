//! Gamepad buttons and the left stick.

use super::{PAD_ARROWS, direction, send_key, set_fire};
use sdl2::controller::Button;

/// Pad buttons act as the original's menu keys.
const KEY_TAKE: u8 = b'T';
const KEY_LIST: u8 = b'I';
const KEY_LOOK: u8 = b'?';
const KEY_USE: u8 = b'U';
const KEY_GIVE: u8 = b'G';
const KEY_PAUSE: u8 = 0x20;

pub fn pad_direction(i: usize, down: bool) {
    direction(PAD_ARROWS + i, down);
}

/// Gamepad buttons act as the original's menu keys.
pub fn pad_button(button: Button, down: bool) {
    let code = match button {
        Button::A => return set_fire(down),
        Button::X => KEY_TAKE,
        Button::Y => KEY_LIST,
        Button::B => KEY_LOOK,
        Button::LeftShoulder => KEY_USE,
        Button::RightShoulder => KEY_GIVE,
        Button::Start => KEY_PAUSE,
        Button::DPadUp => return pad_direction(0, down),
        Button::DPadDown => return pad_direction(1, down),
        Button::DPadLeft => return pad_direction(2, down),
        Button::DPadRight => return pad_direction(3, down),
        _ => return,
    };
    send_key(code, down);
}
