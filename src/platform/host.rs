//! What the platform tells the game: key and mouse input, and the commands a test script asks for.
//! The platform only knows this interface; `main` installs the game's implementation, so the
//! platform code does not depend on the game.

use std::sync::OnceLock;

/// The commands a test script can queue (`FTA_CMD`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Move the hero to the point, like going through a door.
    Teleport { x: i32, y: i32 },
    /// Kill the hero, who has the given luck left.
    KillHero { luck: i32 },
    /// Set inventory slot `slot` to `count`.
    Give { slot: i32, count: i32 },
    /// Set a stat: 0 hunger, 1 fatigue, anything else wealth.
    SetStat { stat: i32, value: i32 },
    /// Kill every enemy that is out.
    SlayEnemies,
}

/// The game, as the platform sees it. Calls can come from any thread.
pub trait Host: Sync {
    /// A key press (or release, with bit 7 set) as the game's key code.
    fn key_event(&self, code: i32);

    /// The pointer moved to (x, y), in lores pixels and screen lines.
    fn mouse_moved(&self, x: i32, y: i32);

    /// The mouse buttons changed.
    fn mouse_buttons_changed(&self, left: bool, right: bool);

    /// The joystick fire button was pressed or released.
    fn set_joystick_fire(&self, down: bool);

    /// A test script asked for a command; the game runs it at the top of its next frame.
    fn queue_command(&self, command: Command);
}

static HOST: OnceLock<&'static dyn Host> = OnceLock::new();

/// Connects the game. Only the first call counts.
pub fn install(host: &'static dyn Host) {
    let _ = HOST.set(host);
}

/// The installed game, if any (before `install`, input and commands are dropped).
pub fn host() -> Option<&'static dyn Host> {
    HOST.get().copied()
}
