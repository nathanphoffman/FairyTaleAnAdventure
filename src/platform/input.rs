//! Keyboard and mouse. SDL events become the key codes and mouse state the original input
//! handler (fsubs.asm HandlerInterface) produced; the logic that fills the game's buffers lives
//! in csrc/input.c.
//!
//! Key codes are the ones the game's `keytrans` table gave: upper-case letters, digits and
//! symbols as ASCII, F1-F10 = 10-19, keypad digits = direction codes 20-29, bit 7 = released.

use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::raw::c_int;

unsafe extern "C" {
    fn fta_key(code: c_int);
    fn fta_mouse_move(x: c_int, y: c_int);
    fn fta_mouse_button(left: c_int, right: c_int);
    static mut fta_pia: u8;
}

thread_local! {
    /// up, down, left, right from the keyboard (0-3) and the gamepad (4-7)
    static ARROWS: Cell<[bool; 8]> = const { Cell::new([false; 8]) };
    /// key remapping from controls.cfg: pressed key -> key the game sees
    static REMAP: RefCell<HashMap<Keycode, Keycode>> = RefCell::new(HashMap::new());
    /// direction key code currently held down (20-27), 0 = none
    static DIR: Cell<u8> = const { Cell::new(0) };
    static BUTTONS: Cell<(bool, bool)> = const { Cell::new((false, false)) };
    /// pointer position in lores pixels / screen lines, as the game sees it
    static POINTER: Cell<(i32, i32)> = const { Cell::new((160, 170)) };
}

/// Direction codes: the game's compass numbering, offset by 20.
///   20 21 22      keypad 7 8 9
///   27 29 23      keypad 4 5 6
///   26 25 24      keypad 1 2 3
fn dir_code(up: bool, down: bool, left: bool, right: bool) -> u8 {
    let dx = right as i32 - left as i32;
    let dy = down as i32 - up as i32;
    match (dx, dy) {
        (-1, -1) => 20,
        (0, -1) => 21,
        (1, -1) => 22,
        (1, 0) => 23,
        (1, 1) => 24,
        (0, 1) => 25,
        (-1, 1) => 26,
        (-1, 0) => 27,
        _ => 0,
    }
}

/// Game key code for a key, 0 if the game has no use for it.
fn translate(key: Keycode) -> u8 {
    match key {
        Keycode::A => b'A',
        Keycode::B => b'B',
        Keycode::C => b'C',
        Keycode::D => b'D',
        Keycode::E => b'E',
        Keycode::F => b'F',
        Keycode::G => b'G',
        Keycode::H => b'H',
        Keycode::I => b'I',
        Keycode::J => b'J',
        Keycode::K => b'K',
        Keycode::L => b'L',
        Keycode::M => b'M',
        Keycode::N => b'N',
        Keycode::O => b'O',
        Keycode::P => b'P',
        Keycode::Q => b'Q',
        Keycode::R => b'R',
        Keycode::S => b'S',
        Keycode::T => b'T',
        Keycode::U => b'U',
        Keycode::V => b'V',
        Keycode::W => b'W',
        Keycode::X => b'X',
        Keycode::Y => b'Y',
        Keycode::Z => b'Z',
        Keycode::Num1 => b'1',
        Keycode::Num2 => b'2',
        Keycode::Num3 => b'3',
        Keycode::Num4 => b'4',
        Keycode::Num5 => b'5',
        Keycode::Num6 => b'6',
        Keycode::Num7 => b'7',
        Keycode::Num8 => b'8',
        Keycode::Num9 => b'9',
        Keycode::Num0 => b'0',
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
        Keycode::Space => 0x20,
        Keycode::Backspace => 8,
        Keycode::Tab => 9,
        Keycode::Return | Keycode::KpEnter => 13,
        Keycode::Delete => 0x7f,
        Keycode::F1 => 10,
        Keycode::F2 => 11,
        Keycode::F3 => 12,
        Keycode::F4 => 13,
        Keycode::F5 => 14,
        Keycode::F6 => 15,
        Keycode::F7 => 16,
        Keycode::F8 => 17,
        Keycode::F9 => 18,
        Keycode::F10 => 19,
        Keycode::Kp0 => b'0',
        Keycode::Kp1 => 26,
        Keycode::Kp2 => 25,
        Keycode::Kp3 => 24,
        Keycode::Kp4 => 27,
        Keycode::Kp5 => 29,
        Keycode::Kp6 => 23,
        Keycode::Kp7 => 20,
        Keycode::Kp8 => 21,
        Keycode::Kp9 => 22,
        _ => 0,
    }
}

/// Reads `controls.cfg` (or the file named by FTA_CONTROLS). Each line is `Key = Action`, where
/// both are SDL key names, e.g. `W = Up`. Lines starting with # are comments.
pub fn load_config() {
    let path = std::env::var("FTA_CONTROLS").unwrap_or_else(|_| "controls.cfg".to_string());
    let Ok(text) = std::fs::read_to_string(&path) else {
        return;
    };
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((from, to)) = line.split_once('=') else {
            continue;
        };
        match (
            Keycode::from_name(from.trim()),
            Keycode::from_name(to.trim()),
        ) {
            (Some(f), Some(t)) => {
                REMAP.with(|r| r.borrow_mut().insert(f, t));
            }
            _ => eprintln!("{path}: cannot read line '{line}'"),
        }
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
        unsafe {
            if old != 0 {
                fta_key(old as c_int | 0x80);
            }
            if new != 0 {
                fta_key(new as c_int);
            }
        }
    }
}

pub fn pad_direction(i: usize, down: bool) {
    direction(4 + i, down);
}

/// Gamepad buttons act as the original's menu keys.
pub fn pad_button(button: sdl2::controller::Button, down: bool) {
    use sdl2::controller::Button as B;
    let code = match button {
        B::A => {
            unsafe { fta_pia = if down { 0x7f } else { 0xff } }; // fire
            return;
        }
        B::X => b'T',             // Take
        B::Y => b'I',             // List
        B::B => b'?',             // Look
        B::LeftShoulder => b'U',  // Use
        B::RightShoulder => b'G', // Give
        B::Start => 0x20,         // Pause
        B::DPadUp => return pad_direction(0, down),
        B::DPadDown => return pad_direction(1, down),
        B::DPadLeft => return pad_direction(2, down),
        B::DPadRight => return pad_direction(3, down),
        _ => return,
    };
    unsafe { fta_key(code as c_int | if down { 0 } else { 0x80 }) };
}

/// Tick counter set by the display loop; stamps the lines of the FTA_RECORD log.
pub static TICK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// FTA_RECORD=file logs every key and mouse event as "tick key NAME 1|0", "tick move X Y" or
/// "tick button left|right 1|0". Feed the file back with FTA_REPLAY=file to repeat the session.
fn record(line: std::fmt::Arguments) {
    use std::io::Write;
    use std::sync::{Mutex, OnceLock};
    static LOG: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    let log = LOG.get_or_init(|| {
        let path = std::env::var_os("FTA_RECORD")?;
        std::fs::File::create(path).ok().map(Mutex::new)
    });
    if let Some(f) = log {
        let tick = TICK.load(std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut f) = f.lock() {
            let _ = writeln!(f, "{tick} {line}");
        }
    }
}

pub fn key_event(key: Keycode, down: bool) {
    record(format_args!(
        "key {} {}",
        key.name().replace(' ', "_"),
        down as u8
    ));
    let key = REMAP.with(|r| r.borrow().get(&key).copied()).unwrap_or(key);
    let arrow = match key {
        Keycode::Up => Some(0),
        Keycode::Down => Some(1),
        Keycode::Left => Some(2),
        Keycode::Right => Some(3),
        _ => None,
    };
    if let Some(i) = arrow {
        direction(i, down);
        return;
    }
    if key == Keycode::LCtrl || key == Keycode::RCtrl {
        // joystick fire button: bit 7 of the CIA port is low while pressed
        unsafe { fta_pia = if down { 0x7f } else { 0xff } };
        return;
    }
    let code = translate(key);
    if code != 0 {
        unsafe { fta_key(code as c_int | if down { 0 } else { 0x80 }) };
    }
}

/// `x`, `y` are positions in the 640x200 frame.
pub fn mouse_move(frame_x: i32, frame_y: i32) {
    record(format_args!("move {frame_x} {frame_y}"));
    let (x, y) = (frame_x / 2, frame_y);
    unsafe { fta_mouse_move(x, y) };
    POINTER.with(|p| p.set((x.clamp(5, 315), y.clamp(147, 195))));
}

pub fn mouse_button(button: MouseButton, down: bool) {
    let (mut l, mut r) = BUTTONS.with(|b| b.get());
    match button {
        MouseButton::Left => l = down,
        MouseButton::Right => r = down,
        _ => return,
    }
    record(format_args!(
        "button {} {}",
        if button == MouseButton::Left {
            "left"
        } else {
            "right"
        },
        down as u8
    ));
    BUTTONS.with(|b| b.set((l, r)));
    unsafe { fta_mouse_button(l as c_int, r as c_int) };
}

/// Pointer position for drawing the cursor.
pub fn pointer() -> (i32, i32) {
    POINTER.with(|p| p.get())
}
