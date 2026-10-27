//! Ending the program.

use crate::game::files::disk::close_disk;
use crate::platform::audio::wrap_music;

/// Stops the music, closes the disk image and ends the program. (The memory the game took is given
/// back by the operating system.)
pub fn close_all() {
    wrap_music();
    close_disk();

    std::process::exit(0);
}
