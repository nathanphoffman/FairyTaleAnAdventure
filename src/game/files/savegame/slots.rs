//! Saving to and loading from a disk slot: finding it, waiting for the disk and the transfer loop.

use super::records::{
    save_variables, transfer_actor, transfer_extent, transfer_object, transfer_party, transfer_u16,
};
use crate::game::render::display::delay;
use crate::game::render::screen::{FontKind, screen};
use crate::platform::dos::{close_file, io_error, lock_path, open_file, unlock_path};

use super::{FLOCK, SAVENAME, SVERR, SVFILE, is_saving};

use crate::amiga::BPTR;
use crate::game::data::object_tables::{dstobs, glbobs, mapobs, ob_listg, ob_table};
use crate::game::data::state::{the_actors, the_encounter, the_screen, the_world};
use crate::game::data::tables::extent_list;
use crate::game::data::types::{Extent, WorldObject};
use crate::game::files::loading::load_start_sprites;
use crate::game::input::handler::take_new_disk;
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::messages::print;
use crate::game::ui::text::queue_print;
use std::ffi::c_char;

const ACCESS_READ: i32 = -2;
const ACCESS_WRITE: i32 = -1;
const MODE_OLDFILE: i32 = 1005;
const MODE_NEWFILE: i32 = 1006;

/// How many 1/50 s ticks `wait_for_disk` waits between checks, and how many checks it makes.
const DISK_POLL_TICKS: i32 = 5;
const DISK_POLL_COUNT: i32 = 300;

/// Offset of the file name in "df1:A.faery", the drive prefix being 4 characters.
const NAME_AFTER_DRIVE: usize = 4;
const DRIVE_DIGIT_INDEX: usize = 2;
const SLOT_LETTER_INDEX: usize = 4;

/// Entries of `extent_list` that are saved, and entries in the per-region object tables.
const SAVED_EXTENTS: usize = 2;
const OBJECT_REGIONS: usize = 10;

/// Value of `view_status` meaning the screen must be redrawn from scratch.
const VIEW_CORRUPT: i8 = 99;

/// Print queue requests that redraw the vitality line and the stats line.
const PRINT_VITALITY: i32 = 4;
const PRINT_STATS: i32 = 7;

/// Non-zero if `name` can be locked with the given access (AmigaDOS's way of asking "does this exist / is it writable").
pub fn can_lock(name: *const c_char, access: i32) -> i32 {
    unsafe {
        FLOCK = lock_path(name, access);

        if FLOCK != 0 {
            unlock_path(FLOCK);
        }

        FLOCK as i32
    }
}

/// Waits (about 30 seconds) for a disk to be inserted. Returns 1 if one was, 0 if not.
pub fn wait_for_disk() -> i32 {
    for _ in 0..DISK_POLL_COUNT {
        if take_new_disk() {
            return 1;
        }

        delay(DISK_POLL_TICKS);
    }

    0
}

/// Where a game is saved to or loaded from.
enum Target {
    /// The file `image` exists, so the save goes in the current directory.
    Folder,
    Floppy,
    NoDisk,
}

/// Picks the drive for the save file, updating the drive digit in `save_path`.
unsafe fn find_target() -> Target {
    unsafe {
        if can_lock(c"image".as_ptr(), ACCESS_READ) != 0 {
            Target::Folder
        } else if can_lock(c"df1:".as_ptr(), ACCESS_WRITE) != 0 {
            SAVENAME[DRIVE_DIGIT_INDEX] = b'1';

            Target::Floppy
        } else if can_lock(c"df0:".as_ptr(), ACCESS_WRITE) != 0
            && can_lock(c"df0:winpic".as_ptr(), ACCESS_READ) == 0
        {
            SAVENAME[DRIVE_DIGIT_INDEX] = b'0';

            Target::Floppy
        } else {
            Target::NoDisk
        }
    }
}

/// Writes (is_saving set) or reads the whole game state to or from the open save file.
pub(super) unsafe fn save_all() {
    unsafe {
        save_variables();
        transfer_u16(&raw mut the_world.region);

        transfer_u16((&raw mut the_actors.active).cast());
        transfer_u16((&raw mut the_actors.drawn).cast());
        transfer_u16((&raw mut the_actors.next_missile).cast());

        for i in 0..the_actors.active as usize {
            transfer_actor(&raw mut the_actors.list[i]);
        }

        transfer_party();

        for i in 0..SAVED_EXTENTS {
            transfer_extent((&raw mut extent_list).cast::<Extent>().add(i));
        }

        for i in 0..glbobs as usize {
            transfer_object((&raw mut ob_listg).cast::<WorldObject>().add(i));
        }

        for count in (*(&raw mut mapobs)).iter_mut().take(OBJECT_REGIONS) {
            transfer_u16((count as *mut i16).cast());
        }

        for count in (*(&raw mut dstobs)).iter_mut().take(OBJECT_REGIONS) {
            transfer_u16((count as *mut i16).cast());
        }

        for i in 0..OBJECT_REGIONS {
            for k in 0..mapobs[i] as usize {
                transfer_object(ob_table[i].add(k));
            }
        }
    }
}

/// Writes in the given font from now on.
fn set_current_font(kind: FontKind) {
    let screen = screen();
    let current = screen.current_port;
    screen.set_font_kind(current, kind);
}

/// Saves the game (`is_saving` set) or loads it into slot `chosen_entry` (0 = A ...).
///
/// Steps: find a place to save (asking for a disk on a real Amiga), open the file, run `save_all`,
/// close it. When loading, afterwards the sprites are reloaded for the loaded game and the
/// status lines are redrawn.
pub fn savegame(chosen_entry: i16) {
    unsafe {
        SVERR = 0;
        set_current_font(FontKind::Panel);

        let Some((name_offset, on_floppy)) = find_save_place() else {
            set_current_font(FontKind::Story);

            return;
        };

        transfer_file(chosen_entry, name_offset);

        if SVERR != 0 {
            print(if is_saving != 0 {
                c"ERROR: Couldn't save game.".as_ptr()
            } else {
                c"ERROR: Couldn't load game.".as_ptr()
            });
        }

        if on_floppy {
            wait_for_game_disk();
        }

        if is_saving == 0 {
            reset_after_load();
        }

        set_current_font(FontKind::Story);
    }
}

/// Finds where the file goes, asking for a disk until there is one. Returns where the name starts in
/// `SAVENAME` and whether it is a floppy, or `None` if the player gave up.
unsafe fn find_save_place() -> Option<(usize, bool)> {
    unsafe {
        loop {
            match find_target() {
                Target::Folder => return Some((NAME_AFTER_DRIVE, false)),
                Target::Floppy => return Some((0, true)),
                Target::NoDisk => {
                    print(c"Insert a writable disk in ANY drive.".as_ptr());

                    if wait_for_disk() == 0 {
                        print(c"Aborted.".as_ptr());

                        return None;
                    }
                }
            }
        }
    }
}

/// Opens the save file of slot `chosen_entry` and writes or reads the game through it.
unsafe fn transfer_file(chosen_entry: i16, name_offset: usize) {
    unsafe {
        SAVENAME[SLOT_LETTER_INDEX] = b'A'.wrapping_add(chosen_entry as u8);
        let name = (&raw const SAVENAME).cast::<c_char>().add(name_offset);
        SVFILE = open_file(
            name,
            if is_saving != 0 {
                MODE_NEWFILE
            } else {
                MODE_OLDFILE
            },
        ) as i32;

        if SVFILE != 0 {
            save_all();
            close_file(SVFILE as BPTR);
        } else {
            SVERR = io_error();
        }
    }
}

/// Asks for the game disk back until the winpic file can be found on drive 0.
unsafe fn wait_for_game_disk() {
    unsafe {
        loop {
            FLOCK = lock_path(c"df0:winpic".as_ptr(), ACCESS_READ);

            if FLOCK != 0 {
                unlock_path(FLOCK);

                return;
            }

            print(c"Please insert GAME disk.".as_ptr());
            wait_for_disk();
        }
    }
}

/// After loading a game: clears the encounter state and redraws the status panel for the new hero.
unsafe fn reset_after_load() {
    unsafe {
        the_encounter.weapon_type = 0;
        the_encounter.number = 0;
        load_start_sprites();
        update_menu_availability();
        the_screen.view_status = VIEW_CORRUPT;
        queue_print(PRINT_VITALITY);
        queue_print(PRINT_STATS);
        print(c"".as_ptr());
        print(c"".as_ptr());
        print(c"".as_ptr());
        the_encounter.kind = 0;
        the_actors.loading = 0;
    }
}
