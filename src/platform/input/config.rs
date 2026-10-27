//! Key remapping from controls.cfg.

use sdl2::keyboard::Keycode;
use std::cell::RefCell;
use std::collections::HashMap;

const CONFIG_VAR: &str = "FTA_CONTROLS";
const DEFAULT_CONFIG: &str = "controls.cfg";

thread_local! {
    /// key remapping from controls.cfg: pressed key -> key the game sees
    static REMAP: RefCell<HashMap<Keycode, Keycode>> = RefCell::new(HashMap::new());
}

/// The key the game should see when `key` is pressed.
pub(super) fn remapped(key: Keycode) -> Keycode {
    REMAP.with(|r| r.borrow().get(&key).copied()).unwrap_or(key)
}

/// Splits "Key = Action  # comment" into its two names. None for blank and comment lines.
fn parse_line(line: &str) -> Option<(&str, &str)> {
    let line = line.split('#').next().unwrap_or("").trim();
    let (from, to) = line.split_once('=')?;

    Some((from.trim(), to.trim()))
}

/// Reads `controls.cfg` (or the file named by FTA_CONTROLS). Each line is `Key = Action`, where
/// both are SDL key names, e.g. `W = Up`. Lines starting with # are comments.
pub fn load_config() {
    let path = std::env::var(CONFIG_VAR).unwrap_or_else(|_| DEFAULT_CONFIG.to_string());
    let Ok(text) = std::fs::read_to_string(&path) else {
        return; // no config file is normal
    };

    for line in text.lines() {
        let Some((from, to)) = parse_line(line) else {
            continue;
        };

        match (Keycode::from_name(from), Keycode::from_name(to)) {
            (Some(f), Some(t)) => {
                REMAP.with(|r| r.borrow_mut().insert(f, t));
            }

            _ => eprintln!("{path}: cannot read line '{line}'"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_key_pairs_and_skips_comments() {
        assert_eq!(parse_line("W = Up"), Some(("W", "Up")));
        assert_eq!(parse_line("  A=Left # strafe"), Some(("A", "Left")));
        assert_eq!(parse_line("# just a comment"), None);
        assert_eq!(parse_line(""), None);
    }
}
