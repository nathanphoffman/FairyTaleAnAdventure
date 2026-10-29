//! The game's side of the platform interface (`platform::host`): input goes to the input handler
//! and test commands to the queue that `run_pending_command` empties.

use crate::game::input::handler;
use crate::game::statedump::queue_command;
use crate::platform::host::{Command, Host};

pub struct GameHost;

impl Host for GameHost {
    fn key_event(&self, code: i32) {
        handler::key_event(code);
    }

    fn mouse_moved(&self, x: i32, y: i32) {
        handler::mouse_moved(x, y);
    }

    fn mouse_buttons_changed(&self, left: bool, right: bool) {
        handler::mouse_buttons_changed(left, right);
    }

    fn set_joystick_fire(&self, down: bool) {
        handler::set_joystick_fire(down);
    }

    fn queue_command(&self, command: Command) {
        queue_command(command);
    }
}
