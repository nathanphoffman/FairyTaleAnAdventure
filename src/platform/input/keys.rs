//! Translating SDL keys into the game's key codes.

use sdl2::keyboard::Keycode;

/// Direction codes are the compass numbers 0-9 offset by this.
const DIR_BASE: u8 = 20;
const CODE_SPACE: u8 = 0x20;
const CODE_BACKSPACE: u8 = 8;
const CODE_TAB: u8 = 9;
const CODE_RETURN: u8 = 13;
const CODE_DELETE: u8 = 0x7f;
/// F1 is code 10, F2 is 11 ... F10 is 19.
const FUNCTION_KEY_BASE: u8 = 10;

/// Direction codes: the game's compass numbering, offset by 20.
///   20 21 22      keypad 7 8 9
///   27 29 23      keypad 4 5 6
///   26 25 24      keypad 1 2 3
pub(super) fn dir_code(up: bool, down: bool, left: bool, right: bool) -> u8 {
    let dx = right as i32 - left as i32;
    let dy = down as i32 - up as i32;
    match (dx, dy) {
        (-1, -1) => DIR_BASE,
        (0, -1) => DIR_BASE + 1,
        (1, -1) => DIR_BASE + 2,
        (1, 0) => DIR_BASE + 3,
        (1, 1) => DIR_BASE + 4,
        (0, 1) => DIR_BASE + 5,
        (-1, 1) => DIR_BASE + 6,
        (-1, 0) => DIR_BASE + 7,
        _ => 0,
    }
}

/// Keypad keys give the direction code of the key's position.
fn keypad_code(key: Keycode) -> Option<u8> {
    let offset = match key {
        Keycode::Kp7 => 0,
        Keycode::Kp8 => 1,
        Keycode::Kp9 => 2,
        Keycode::Kp6 => 3,
        Keycode::Kp3 => 4,
        Keycode::Kp2 => 5,
        Keycode::Kp1 => 6,
        Keycode::Kp4 => 7,
        Keycode::Kp5 => 9,
        _ => return None,
    };
    Some(DIR_BASE + offset)
}

/// Letters and digits are their upper-case ASCII code (SDL names them "A" and "1").
fn alphanumeric_code(key: Keycode) -> Option<u8> {
    match key.name().as_bytes() {
        [c] if c.is_ascii_uppercase() || c.is_ascii_digit() => Some(*c),
        _ => None,
    }
}

/// F1-F10 in order.
fn function_key_code(key: Keycode) -> Option<u8> {
    let keys = [
        Keycode::F1,
        Keycode::F2,
        Keycode::F3,
        Keycode::F4,
        Keycode::F5,
        Keycode::F6,
        Keycode::F7,
        Keycode::F8,
        Keycode::F9,
        Keycode::F10,
    ];
    let index = keys.iter().position(|&k| k == key)?;
    Some(FUNCTION_KEY_BASE + index as u8)
}

/// Game key code for a key, 0 if the game has no use for it.
pub(super) fn translate(key: Keycode) -> u8 {
    let simple = match key {
        Keycode::Minus | Keycode::KpMinus => b'-',
        Keycode::Equals => b'=',
        Keycode::Backquote => b'`',
        Keycode::Backslash => b'\\',
        Keycode::LeftBracket => b'{',
        Keycode::RightBracket => b'}',
        Keycode::Semicolon => b':',
        Keycode::Quote | Keycode::Slash => b'?',
        Keycode::Comma => b',',
        Keycode::Period | Keycode::KpPeriod => b'.',
        Keycode::Space => CODE_SPACE,
        Keycode::Backspace => CODE_BACKSPACE,
        Keycode::Tab => CODE_TAB,
        Keycode::Return | Keycode::KpEnter => CODE_RETURN,
        Keycode::Delete => CODE_DELETE,
        Keycode::Kp0 => b'0',
        _ => 0,
    };
    if simple != 0 {
        return simple;
    }
    alphanumeric_code(key)
        .or_else(|| function_key_code(key))
        .or_else(|| keypad_code(key))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_and_digits_become_ascii() {
        assert_eq!(translate(Keycode::A), b'A');
        assert_eq!(translate(Keycode::Z), b'Z');
        assert_eq!(translate(Keycode::Num0), b'0');
        assert_eq!(translate(Keycode::Num9), b'9');
    }

    #[test]
    fn function_and_keypad_keys_use_the_game_codes() {
        assert_eq!(translate(Keycode::F1), 10);
        assert_eq!(translate(Keycode::F10), 19);
        assert_eq!(translate(Keycode::Kp7), 20);
        assert_eq!(translate(Keycode::Kp5), 29);
        assert_eq!(translate(Keycode::Kp0), b'0');
    }

    #[test]
    fn unused_keys_give_zero() {
        assert_eq!(translate(Keycode::LShift), 0);
    }

    #[test]
    fn two_arrows_make_a_diagonal() {
        assert_eq!(dir_code(true, false, false, false), 21);
        assert_eq!(dir_code(true, false, false, true), 22);
        assert_eq!(dir_code(false, false, false, false), 0);
        assert_eq!(dir_code(true, true, false, false), 0);
    }
}
