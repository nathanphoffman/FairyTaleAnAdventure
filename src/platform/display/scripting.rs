//! Scripted input for tests: key presses, clicks, recorded sessions and game commands.

use super::Display;
use crate::platform::input;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::cell::RefCell;

unsafe extern "C" {
    fn fta_queue_cmd(kind: i32, a: i32, b: i32);
}

/// A key press stays down this many ticks unless the script says otherwise.
const DEFAULT_HOLD_TICKS: u32 = 4;
/// A scripted click releases the button this many ticks after pressing it.
const CLICK_HOLD_TICKS: u32 = 3;
/// `fta_queue_cmd` kinds.
const CMD_TELEPORT: i32 = 1;
const CMD_KILL: i32 = 2;
/// A test command has at least tick, name and one number.
const MIN_CMD_FIELDS: usize = 3;
const LORES_TO_FRAME: i32 = 2;

/// FTA_INJECT="100:Space,200:Right:60" presses keys at those 1/50 s ticks, held for 4 ticks or
/// the given number.
pub(super) fn parse_script() -> Vec<(u32, String, bool)> {
    let mut script = Vec::new();
    for item in std::env::var("FTA_INJECT").unwrap_or_default().split(',') {
        let mut parts = item.split(':');
        let (Some(t), Some(name)) = (parts.next(), parts.next()) else {
            continue;
        };
        let hold: u32 = parts
            .next()
            .and_then(|h| h.parse().ok())
            .unwrap_or(DEFAULT_HOLD_TICKS);
        if let Ok(t) = t.parse::<u32>() {
            script.push((t, name.to_string(), true));
            script.push((t + hold, name.to_string(), false));
        }
    }
    script
}

/// FTA_CLICK="200:225:166,260:225:184" clicks the left button at those ticks and positions
/// (x in 320 wide lores pixels, y in screen lines).
pub(super) fn parse_clicks() -> Vec<(u32, i32, i32)> {
    parse_click_list(&std::env::var("FTA_CLICK").unwrap_or_default())
}

/// Splits "tick:x:y,tick:x:y" into numbers, skipping items that are not three numbers.
fn parse_click_list(text: &str) -> Vec<(u32, i32, i32)> {
    text.split(',')
        .filter_map(|item| {
            let mut p = item.split(':').map(|v| v.parse::<i32>().ok());
            Some((p.next()?? as u32, p.next()??, p.next()??))
        })
        .collect()
}

type ReplayLog = Vec<(u32, Vec<String>)>;

/// Reads the file named by FTA_REPLAY: one event per line, "tick word word ...".
fn load_replay() -> ReplayLog {
    let Some(path) = std::env::var_os("FTA_REPLAY") else {
        return Vec::new();
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Could not read the replay file: {e}");
            return Vec::new();
        }
    };
    let mut log = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace().map(String::from);
        if let Some(tick) = words.next().and_then(|t| t.parse().ok()) {
            log.push((tick, words.collect()));
        }
    }
    log
}

/// Plays one logged event: "key NAME 1", "move X Y" or "button left|right 1".
fn replay_event(w: &[String]) {
    let down = w.last().map(|s| s == "1").unwrap_or(false);
    match w[0].as_str() {
        "key" => {
            if let Some(k) = Keycode::from_name(&w[1].replace('_', " ")) {
                input::key_event(k, down);
            }
        }
        "move" => {
            if let (Ok(x), Ok(y)) = (w[1].parse(), w[2].parse()) {
                input::mouse_move(x, y);
            }
        }
        "button" => {
            let b = if w[1] == "left" {
                MouseButton::Left
            } else {
                MouseButton::Right
            };
            input::mouse_button(b, down);
        }
        _ => {}
    }
}

/// FTA_REPLAY=file replays a log written by FTA_RECORD.
fn replay_events(tick: u32) {
    thread_local! {
        static LOG: RefCell<ReplayLog> = RefCell::new(load_replay());
    }
    LOG.with(|log| {
        for (_, w) in log.borrow().iter().filter(|(t, _)| *t == tick) {
            replay_event(w);
        }
    });
}

/// FTA_CMD="900:tp:18310:15969,1200:kill:0" runs test commands at those ticks: tp = teleport the
/// hero like a door does (x, y), kill = set luck and kill the hero.
fn run_cmds(tick: u32) {
    let Ok(cmds) = std::env::var("FTA_CMD") else {
        return;
    };
    for item in cmds.split(',') {
        let p: Vec<&str> = item.split(':').collect();
        if p.len() < MIN_CMD_FIELDS || p[0].parse::<u32>().ok() != Some(tick) {
            continue;
        }
        let a: i32 = p[2].parse().unwrap_or(0);
        let b: i32 = p.get(3).and_then(|v| v.parse().ok()).unwrap_or(0);
        match p[1] {
            "tp" => unsafe { fta_queue_cmd(CMD_TELEPORT, a, b) },
            "kill" => unsafe { fta_queue_cmd(CMD_KILL, a, b) },
            _ => {}
        }
    }
}

/// Runs everything the test environment variables scheduled for the current tick.
pub(super) fn run_script(d: &mut Display) {
    let tick = d.ticks;
    run_cmds(tick);
    input::TICK.store(tick, std::sync::atomic::Ordering::Relaxed);
    replay_events(tick);
    for &(t, x, y) in &d.clicks {
        if t == tick {
            input::mouse_move(x * LORES_TO_FRAME, y);
            input::mouse_button(MouseButton::Left, true);
        } else if t + CLICK_HOLD_TICKS == tick {
            input::mouse_button(MouseButton::Left, false);
        }
    }
    for (t, name, down) in &d.script {
        if *t == tick
            && let Some(k) = Keycode::from_name(name)
        {
            input::key_event(k, *down);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_script_is_parsed_from_the_environment_format() {
        let parsed = parse_click_list("200:225:166,bad,260:1:2");
        assert_eq!(parsed, vec![(200, 225, 166), (260, 1, 2)]);
    }
}
