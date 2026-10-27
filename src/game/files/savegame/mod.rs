//! Saving and loading the game (from csrc/fmain2.c). The save file is written field by field in
//! big-endian order, so it does not depend on how the C compiler laid out the variables.
//!
//! The same code does saving and loading: each `transfer_*` function writes a value when `is_saving` is set
//! and reads it back when it is clear. So the order of the calls in `save_all` is the file format:
//! the misc variables, the region, the actors, the inventories and missiles, the extents and the
//! object lists.
//!
//! The original asked for a floppy disk with a free drive (df1: or df0:). When a file named `image`
//! exists (always, on Linux) the save is just named `A.faery`, `B.faery` ... (one letter per menu
//! choice) and the platform layer puts the files in the `saves/` folder.

mod records;

mod slots;

pub use self::slots::savegame;

use crate::amiga::BPTR;

/// Set when saving, clear when loading.
pub static mut is_saving: i8 = 0;
pub static mut SVFILE: i32 = 0;
pub static mut SVERR: i32 = 0;
pub static mut SAVENAME: [u8; 12] = *b"df1:A.faery\0";
pub static mut FLOCK: BPTR = 0;
