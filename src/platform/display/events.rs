//! Window, keyboard, mouse and gamepad events.

use super::Display;
use crate::platform::input;
use sdl2::controller::Axis;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::video::FullscreenType;

/// How far a stick must move (of 32767) before it counts as a direction.
const STICK_DEAD_ZONE: i16 = 12000;

/// SDL already reports mouse positions in the 640x400 logical screen (it does that when a logical
/// size is set, and it handles the letterboxing), so only the doubled rows need undoing.
fn to_frame(lx: i32, ly: i32) -> (i32, i32) {
    (lx, ly / super::ROW_SCALE as i32)
}

/// F11 or Alt+Enter turn fullscreen on and off.
fn is_fullscreen_toggle(key: Keycode, keymod: Mod) -> bool {
    key == Keycode::F11
        || (key == Keycode::Return && keymod.intersects(Mod::LALTMOD | Mod::RALTMOD))
}

fn toggle_fullscreen(d: &mut Display) {
    let window = d.canvas.window_mut();
    let next = match window.fullscreen_state() {
        FullscreenType::Off => FullscreenType::Desktop,
        _ => FullscreenType::Off,
    };
    if let Err(e) = window.set_fullscreen(next) {
        eprintln!("Could not change fullscreen: {e}");
    }
}

/// Turns the left stick into the four direction inputs, reporting only changes.
/// Each change is (index into `stick`, direction input, pressed).
fn stick_motion(d: &mut Display, axis: Axis, value: i16) {
    let changes: [(usize, usize, bool); 2] = match axis {
        Axis::LeftX => [
            (0, 2, value < -STICK_DEAD_ZONE),
            (1, 3, value > STICK_DEAD_ZONE),
        ],
        Axis::LeftY => [
            (2, 0, value < -STICK_DEAD_ZONE),
            (3, 1, value > STICK_DEAD_ZONE),
        ],
        _ => return,
    };
    for (si, dir, on) in changes {
        if d.stick[si] != on {
            d.stick[si] = on;
            input::pad_direction(dir, on);
        }
    }
}

fn handle_mouse(event: Event) {
    match event {
        Event::MouseMotion { x, y, .. } => {
            let (fx, fy) = to_frame(x, y);
            input::mouse_move(fx, fy);
        }
        Event::MouseButtonDown {
            mouse_btn, x, y, ..
        } => {
            let (fx, fy) = to_frame(x, y);
            input::mouse_move(fx, fy);
            input::mouse_button(mouse_btn, true);
        }
        Event::MouseButtonUp { mouse_btn, .. } => input::mouse_button(mouse_btn, false),
        _ => {}
    }
}

fn handle_event(d: &mut Display, event: Event) {
    match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => std::process::exit(0),
        Event::KeyDown {
            keycode: Some(k),
            keymod,
            ..
        } if is_fullscreen_toggle(k, keymod) => toggle_fullscreen(d),
        Event::ControllerDeviceAdded { which, .. } => {
            if let Ok(c) = d.controller_system.open(which) {
                d.pads.push(c);
            }
        }
        Event::ControllerButtonDown { button, .. } => input::pad_button(button, true),
        Event::ControllerButtonUp { button, .. } => input::pad_button(button, false),
        Event::ControllerAxisMotion { axis, value, .. } => stick_motion(d, axis, value),
        Event::KeyDown {
            keycode: Some(k),
            repeat: false,
            ..
        } => input::key_event(k, true),
        Event::KeyUp {
            keycode: Some(k), ..
        } => input::key_event(k, false),
        other => handle_mouse(other),
    }
}

pub(super) fn handle_events(d: &mut Display) {
    let events: Vec<Event> = d.events.poll_iter().collect();
    for event in events {
        handle_event(d, event);
    }
}
