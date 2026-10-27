//! AmigaDOS file calls on top of normal files.
//!
//! The game opens files by bare name ("image", "songs", "fonts/Amber/9"). They are looked up in
//! the data directory (env `FTA_DATA`, default `faery-tale-amiga/game`), ignoring case. Names
//! like `df1:A.faery` (save disk) go to the `saves/` directory instead.

mod files;

mod locks;

mod paths;

pub use paths::resolve;

pub use self::files::{close_file, open_file, read_file, write_file};
pub use self::locks::{io_error, lock_path, unlock_path};
