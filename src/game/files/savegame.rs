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

use crate::amiga::{BPTR, RastPort, TextFont};
use crate::game::data::globals::{
    panel_font, princesses_rescued, save_padding_1, save_padding_2, save_padding_3, story_font,
};
use crate::game::data::object_tables::{dstobs, glbobs, mapobs, ob_listg, ob_table};
use crate::game::data::state::{
    the_actors, the_encounter, the_hero, the_screen, the_ui, the_world,
};
use crate::game::data::tables::{brother_stats, extent_list};
use crate::game::data::types::{Extent, Missile, Shape, WorldObject};
use crate::game::files::loading::load_start_sprites;
use crate::game::input::handler::take_new_disk;
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::messages::print;
use crate::game::ui::text::queue_print;
use std::ffi::{c_char, c_void};

unsafe extern "C" {
    fn Lock(name: *const c_char, mode: i32) -> BPTR;
    fn UnLock(lock: BPTR);
    fn Open(name: *const c_char, mode: i32) -> BPTR;
    fn Close(file: BPTR);
    fn Read(file: BPTR, buf: *mut u8, len: i32) -> i32;
    fn Write(file: BPTR, buf: *const c_void, len: i32) -> i32;
    fn IoErr() -> i32;
    fn Delay(ticks: i32);
    fn SetFont(current_rastport: *mut RastPort, font: *mut TextFont);
}

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
const MISC_VARIABLES: usize = 40;

/// Set when saving, clear when loading.
pub static mut is_saving: i8 = 0;
pub static mut SVFILE: i32 = 0;
pub static mut SVERR: i32 = 0;
pub static mut SAVENAME: [u8; 12] = *b"df1:A.faery\0";
pub static mut FLOCK: BPTR = 0;

/// Non-zero if `name` can be locked with the given access (AmigaDOS's way of asking "does this exist / is it writable").
pub fn can_lock(name: *const c_char, access: i32) -> i32 {
    unsafe {
        FLOCK = Lock(name, access);
        if FLOCK != 0 {
            UnLock(FLOCK);
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
        unsafe { Delay(DISK_POLL_TICKS) };
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
unsafe fn save_all() {
    unsafe {
        save_variables();
        transfer_u16(&raw mut the_world.region);

        transfer_u16((&raw mut the_actors.active).cast());
        transfer_u16((&raw mut the_actors.drawn).cast());
        transfer_u16((&raw mut the_actors.next_missile).cast());
        for i in 0..the_actors.active as usize {
            transfer_actor(&raw mut the_actors.list[i]);
        }

        transfer_brothers_and_missiles();

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

/// Saves the game (`is_saving` set) or loads it into slot `chosen_entry` (0 = A ...).
///
/// Steps: find a place to save (asking for a disk on a real Amiga), open the file, run `save_all`,
/// close it. When loading, afterwards the sprites are reloaded for the loaded game and the
/// status lines are redrawn.
pub fn savegame(chosen_entry: i16) {
    unsafe {
        SVERR = 0;
        SetFont(the_screen.current_rastport, panel_font);

        let mut on_floppy = true;
        let name_offset = loop {
            match find_target() {
                Target::Folder => {
                    on_floppy = false;
                    break NAME_AFTER_DRIVE;
                }
                Target::Floppy => break 0,
                Target::NoDisk => {
                    print(c"Insert a writable disk in ANY drive.".as_ptr());
                    if wait_for_disk() == 0 {
                        print(c"Aborted.".as_ptr());
                        SetFont(the_screen.current_rastport, story_font);
                        return;
                    }
                }
            }
        };
        SAVENAME[SLOT_LETTER_INDEX] = b'A'.wrapping_add(chosen_entry as u8);
        let name = (&raw const SAVENAME).cast::<c_char>().add(name_offset);
        SVFILE = Open(
            name,
            if is_saving != 0 {
                MODE_NEWFILE
            } else {
                MODE_OLDFILE
            },
        ) as i32;
        if SVFILE != 0 {
            save_all();
            Close(SVFILE as BPTR);
        } else {
            SVERR = IoErr();
        }
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
        SetFont(the_screen.current_rastport, story_font);
    }
}

/// Asks for the game disk back until the winpic file can be found on drive 0.
unsafe fn wait_for_game_disk() {
    unsafe {
        loop {
            FLOCK = Lock(c"df0:winpic".as_ptr(), ACCESS_READ);
            if FLOCK != 0 {
                UnLock(FLOCK);
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

/// Writes or reads `length` bytes of the save file, remembering an error.
pub fn transfer_bytes(buffer: *mut c_void, length: i32) {
    unsafe {
        let result = if is_saving != 0 {
            Write(SVFILE as BPTR, buffer, length)
        } else {
            Read(SVFILE as BPTR, buffer.cast(), length)
        };
        // the original kept the result in a 16 bit variable
        if (result as i16) < 0 {
            SVERR = IoErr();
        }
    }
}

/// Saves or loads one byte.
pub fn transfer_u8(v: *mut u8) {
    transfer_bytes(v.cast(), 1);
}

/// Saves or loads a 16-bit number, always stored big-endian (as the Amiga's 68000 kept it).
pub fn transfer_u16(v: *mut u16) {
    unsafe {
        let mut bytes = [0u8; 2];
        if is_saving != 0 {
            bytes = (*v).to_be_bytes();
            transfer_bytes(bytes.as_mut_ptr().cast(), 2);
        } else {
            transfer_bytes(bytes.as_mut_ptr().cast(), 2);
            *v = u16::from_be_bytes(bytes);
        }
    }
}

/// The 40 variables the original saved as one 80 byte block, in that order: the map and hero
/// positions, the stats (bravery, luck, kindness, wealth, hunger, fatigue), which brother is
/// playing, the day/night clock, the magic timers and so on. The order is the file format.
fn save_variables() {
    let variables: [*mut u16; MISC_VARIABLES] = unsafe {
        [
            &raw mut the_world.map_x,
            &raw mut the_world.map_y,
            &raw mut the_hero.x,
            &raw mut the_hero.y,
            &raw mut the_hero.safe_x,
            &raw mut the_hero.safe_y,
            &raw mut the_hero.safe_r,
            &raw mut the_world.tile_x,
            &raw mut the_world.tile_y,
            (&raw mut the_ui.cheat_enabled).cast(),
            (&raw mut the_hero.riding).cast(),
            (&raw mut the_hero.flying).cast(),
            (&raw mut the_actors.carrier_slot).cast(),
            (&raw mut the_actors.turtle_nearness).cast(),
            (&raw mut the_actors.raft_nearness).cast(),
            (&raw mut the_hero.brave).cast(),
            (&raw mut the_hero.luck).cast(),
            (&raw mut the_hero.kindness).cast(),
            (&raw mut the_hero.wealth).cast(),
            (&raw mut the_hero.hunger).cast(),
            (&raw mut the_hero.fatigue).cast(),
            (&raw mut the_hero.brother).cast(),
            (&raw mut princesses_rescued).cast(),
            (&raw mut the_hero.sector).cast(),
            &raw mut the_hero.place,
            &raw mut the_world.day_clock,
            &raw mut the_world.light_level,
            (&raw mut the_actors.loaded_actor_file).cast(),
            (&raw mut the_actors.loaded_setfig_file).cast(),
            (&raw mut the_actors.active_carrier).cast(),
            &raw mut the_world.extent_type,
            (&raw mut the_actors.leader).cast(),
            (&raw mut the_world.secret_timer).cast(),
            (&raw mut the_world.light_timer).cast(),
            (&raw mut the_world.freeze_timer).cast(),
            (&raw mut the_ui.current_menu).cast(),
            (&raw mut the_encounter.kind).cast(),
            &raw mut save_padding_1,
            &raw mut save_padding_2,
            &raw mut save_padding_3,
        ]
    };
    for v in variables {
        transfer_u16(v);
    }
}

/// Saves or loads one actor.
pub fn transfer_actor(a: *mut Shape) {
    unsafe {
        let a = &mut *a;
        transfer_u16(&mut a.abs_x);
        transfer_u16(&mut a.abs_y);
        transfer_u16(&mut a.rel_x);
        transfer_u16(&mut a.rel_y);
        transfer_u8((&mut a.kind as *mut i8).cast());
        transfer_u8(&mut a.race);
        for byte in [
            &mut a.index,
            &mut a.visible,
            &mut a.weapon,
            &mut a.environ,
            &mut a.goal,
            &mut a.tactic,
            &mut a.state,
            &mut a.facing,
        ] {
            transfer_u8((byte as *mut i8).cast());
        }
        transfer_u16((&mut a.vitality as *mut i16).cast());
        transfer_u8((&mut a.vel_x as *mut i8).cast());
        transfer_u8((&mut a.vel_y as *mut i8).cast());
    }
}

/// Saves or loads one extent (a special rectangle of the world).
pub fn transfer_extent(e: *mut Extent) {
    unsafe {
        let e = &mut *e;
        transfer_u16(&mut e.x1);
        transfer_u16(&mut e.y1);
        transfer_u16(&mut e.x2);
        transfer_u16(&mut e.y2);
        transfer_u8(&mut e.etype);
        transfer_u8(&mut e.v1);
        transfer_u8(&mut e.v2);
        transfer_u8(&mut e.v3);
    }
}

/// Saves or loads one object (an item or person in the world).
pub fn transfer_object(o: *mut WorldObject) {
    unsafe {
        let o = &mut *o;
        transfer_u16(&mut o.xc);
        transfer_u16(&mut o.yc);
        transfer_u8((&mut o.ob_id as *mut i8).cast());
        transfer_u8((&mut o.ob_stat as *mut i8).cast());
    }
}

/// Saves or loads one missile (arrow, rock, fireball). Called from `transfer_brothers_and_missiles` in csrc/fmain.c.
pub fn transfer_missile(m: *mut Missile) {
    unsafe {
        let m = &mut *m;
        transfer_u16(&mut m.abs_x);
        transfer_u16(&mut m.abs_y);
        for byte in [
            &mut m.missile_type,
            &mut m.time_of_flight,
            &mut m.speed,
            &mut m.direction,
            &mut m.archer,
        ] {
            transfer_u8((byte as *mut i8).cast());
        }
    }
}

/// Size of one brother's item list.
const STUFF_SIZE: i32 = 35;

/// Saves or loads the three brothers' item lists and the 6 missiles (the count `next_missile` is saved
/// elsewhere). After loading, `stuff` points at the current brother's list.
pub fn transfer_brothers_and_missiles() {
    unsafe {
        transfer_bytes((&raw mut the_hero.julstuff).cast(), STUFF_SIZE);
        transfer_bytes((&raw mut the_hero.philstuff).cast(), STUFF_SIZE);
        transfer_bytes((&raw mut the_hero.kevstuff).cast(), STUFF_SIZE);
        the_hero.stuff = brother_stats[the_hero.brother as usize - 1].stuff;
        for i in 0..6 {
            transfer_missile((&raw mut the_actors.missiles).cast::<Missile>().add(i));
        }
    }
}
