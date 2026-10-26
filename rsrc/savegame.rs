//! Saving and loading the game (from csrc/fmain2.c). The save file is written field by field in
//! big-endian order, so it does not depend on how the C compiler laid out the variables.
//!
//! The same code does saving and loading: each `sl_*` function writes a value when `svflag` is set
//! and reads it back when it is clear. So the order of the calls in `save_all` is the file format:
//! the misc variables, the region, the actors, the inventories and missiles, the extents and the
//! object lists.
//!
//! The original asked for a floppy disk with a free drive (df1: or df0:). When a file named `image`
//! exists (always, on Linux) the save is just named `A.faery`, `B.faery` ... (one letter per menu
//! choice) and the platform layer puts the files in the `saves/` folder.

use super::asm_text::prq;
use super::input::take_new_disk;
use super::messages::print;
use super::tables::{dstobs, glbobs, mapobs, ob_listg, ob_table};
use super::types::{Extent, Missile, Shape, WorldObject};
use crate::amiga::{BPTR, RastPort, TextFont};
use std::ffi::{c_char, c_void};
use super::loading::{shape_read};
use super::menus::{set_options};
use super::game_tables::{blist, extent_list, region_num};
use super::globals::{active_carrier, actor_file, actors_loading, afont, anim_list, anix, anix2, brave, brother, cheat1, cmode, daynight, encounter_number, encounter_type, fatigue, flying, freeze_timer, hero_place, hero_sector, hero_x, hero_y, hunger, img_x, img_y, julstuff, kevstuff, kind, leader, light_timer, lightlevel, luck, map_x, map_y, mdex, missile_list, pad1, pad2, pad3, philstuff, princess, raftprox, riding, rp, safe_r, safe_x, safe_y, secret_timer, set_file, stuff, tfont, turtleprox, viewstatus, wcarry, wealth, wt, xtype};

unsafe extern "C" {
    fn Lock(name: *const c_char, mode: i32) -> BPTR;
    fn UnLock(lock: BPTR);
    fn Open(name: *const c_char, mode: i32) -> BPTR;
    fn Close(file: BPTR);
    fn Read(file: BPTR, buf: *mut u8, len: i32) -> i32;
    fn Write(file: BPTR, buf: *const c_void, len: i32) -> i32;
    fn IoErr() -> i32;
    fn Delay(ticks: i32);
    fn SetFont(rp: *mut RastPort, font: *mut TextFont);
}

const ACCESS_READ: i32 = -2;
const ACCESS_WRITE: i32 = -1;
const MODE_OLDFILE: i32 = 1005;
const MODE_NEWFILE: i32 = 1006;

/// How many 1/50 s ticks `waitnewdisk` waits between checks, and how many checks it makes.
const DISK_POLL_TICKS: i32 = 5;
const DISK_POLL_COUNT: i32 = 300;
/// Offset of the file name in "df1:A.faery", the drive prefix being 4 characters.
const NAME_AFTER_DRIVE: usize = 4;
const DRIVE_DIGIT_INDEX: usize = 2;
const SLOT_LETTER_INDEX: usize = 4;
/// Entries of `extent_list` that are saved, and entries in the per-region object tables.
const SAVED_EXTENTS: usize = 2;
const OBJECT_REGIONS: usize = 10;
/// Value of `viewstatus` meaning the screen must be redrawn from scratch.
const VIEW_CORRUPT: i8 = 99;
/// Print queue requests that redraw the vitality line and the stats line.
const PRINT_VITALITY: i32 = 4;
const PRINT_STATS: i32 = 7;
const MISC_VARIABLES: usize = 40;

/// Set when saving, clear when loading.
pub static mut svflag: i8 = 0;
pub static mut svfile: i32 = 0;
pub static mut sverr: i32 = 0;
pub static mut savename: [u8; 12] = *b"df1:A.faery\0";
pub static mut flock: BPTR = 0;

/// Non-zero if `name` can be locked with the given access (AmigaDOS's way of asking "does this exist / is it writable").
pub fn locktest(name: *const c_char, access: i32) -> i32 {
    unsafe {
        flock = Lock(name, access);
        if flock != 0 {
            UnLock(flock);
        }
        flock as i32
    }
}

/// Waits (about 30 seconds) for a disk to be inserted. Returns 1 if one was, 0 if not.
pub fn waitnewdisk() -> i32 {
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

/// Picks the drive for the save file, updating the drive digit in `savename`.
unsafe fn find_target() -> Target {
    unsafe {
        if locktest(c"image".as_ptr(), ACCESS_READ) != 0 {
            Target::Folder
        } else if locktest(c"df1:".as_ptr(), ACCESS_WRITE) != 0 {
            savename[DRIVE_DIGIT_INDEX] = b'1';
            Target::Floppy
        } else if locktest(c"df0:".as_ptr(), ACCESS_WRITE) != 0
            && locktest(c"df0:winpic".as_ptr(), ACCESS_READ) == 0
        {
            savename[DRIVE_DIGIT_INDEX] = b'0';
            Target::Floppy
        } else {
            Target::NoDisk
        }
    }
}

/// Writes (svflag set) or reads the whole game state to or from the open save file.
unsafe fn save_all() {
    unsafe {
        save_variables();
        sl_u16(&raw mut region_num);

        sl_u16((&raw mut anix).cast());
        sl_u16((&raw mut anix2).cast());
        sl_u16((&raw mut mdex).cast());
        for i in 0..anix as usize {
            sl_shape(&raw mut anim_list[i]);
        }

        mod1save();

        for i in 0..SAVED_EXTENTS {
            sl_extent((&raw mut extent_list).cast::<Extent>().add(i));
        }
        for i in 0..glbobs as usize {
            sl_object((&raw mut ob_listg).cast::<WorldObject>().add(i));
        }
        for i in 0..OBJECT_REGIONS {
            sl_u16((&raw mut mapobs[i]).cast());
        }
        for i in 0..OBJECT_REGIONS {
            sl_u16((&raw mut dstobs[i]).cast());
        }
        for i in 0..OBJECT_REGIONS {
            for k in 0..mapobs[i] as usize {
                sl_object(ob_table[i].add(k));
            }
        }
    }
}

/// Saves the game (`svflag` set) or loads it into slot `hit` (0 = A ...).
///
/// Steps: find a place to save (asking for a disk on a real Amiga), open the file, run `save_all`,
/// close it. When loading, afterwards the sprites are reloaded for the loaded game and the
/// status lines are redrawn.
pub fn savegame(hit: i16) {
    unsafe {
        sverr = 0;
        SetFont(rp, tfont);

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
                    if waitnewdisk() == 0 {
                        print(c"Aborted.".as_ptr());
                        SetFont(rp, afont);
                        return;
                    }
                }
            }
        };
        savename[SLOT_LETTER_INDEX] = b'A'.wrapping_add(hit as u8);
        let name = (&raw const savename).cast::<c_char>().add(name_offset);
        svfile = Open(
            name,
            if svflag != 0 {
                MODE_NEWFILE
            } else {
                MODE_OLDFILE
            },
        ) as i32;
        if svfile != 0 {
            save_all();
            Close(svfile as BPTR);
        } else {
            sverr = IoErr();
        }
        if sverr != 0 {
            print(if svflag != 0 {
                c"ERROR: Couldn't save game.".as_ptr()
            } else {
                c"ERROR: Couldn't load game.".as_ptr()
            });
        }
        if on_floppy {
            loop {
                flock = Lock(c"df0:winpic".as_ptr(), ACCESS_READ);
                if flock != 0 {
                    UnLock(flock);
                    break;
                }
                print(c"Please insert GAME disk.".as_ptr());
                waitnewdisk();
            }
        }
        if svflag == 0 {
            wt = 0;
            encounter_number = 0;
            shape_read();
            set_options();
            viewstatus = VIEW_CORRUPT;
            prq(PRINT_VITALITY);
            prq(PRINT_STATS);
            print(c"".as_ptr());
            print(c"".as_ptr());
            print(c"".as_ptr());
            encounter_type = 0;
            actors_loading = 0;
        }
        SetFont(rp, afont);
    }
}

/// Writes or reads `length` bytes of the save file, remembering an error.
pub fn saveload(buffer: *mut c_void, length: i32) {
    unsafe {
        let result = if svflag != 0 {
            Write(svfile as BPTR, buffer, length)
        } else {
            Read(svfile as BPTR, buffer.cast(), length)
        };
        // the original kept the result in a 16 bit variable
        if (result as i16) < 0 {
            sverr = IoErr();
        }
    }
}

/// Saves or loads one byte.
pub fn sl_u8(v: *mut u8) {
    saveload(v.cast(), 1);
}

/// Saves or loads a 16-bit number, always stored big-endian (as the Amiga's 68000 kept it).
pub fn sl_u16(v: *mut u16) {
    unsafe {
        let mut bytes = [0u8; 2];
        if svflag != 0 {
            bytes = (*v).to_be_bytes();
            saveload(bytes.as_mut_ptr().cast(), 2);
        } else {
            saveload(bytes.as_mut_ptr().cast(), 2);
            *v = u16::from_be_bytes(bytes);
        }
    }
}

/// The 40 variables the original saved as one 80 byte block, in that order: the map and hero
/// positions, the stats (bravery, luck, kindness, wealth, hunger, fatigue), which brother is
/// playing, the day/night clock, the magic timers and so on. The order is the file format.
fn save_variables() {
    let variables: [*mut u16; MISC_VARIABLES] = [
        &raw mut map_x,
        &raw mut map_y,
        &raw mut hero_x,
        &raw mut hero_y,
        &raw mut safe_x,
        &raw mut safe_y,
        &raw mut safe_r,
        &raw mut img_x,
        &raw mut img_y,
        (&raw mut cheat1).cast(),
        (&raw mut riding).cast(),
        (&raw mut flying).cast(),
        (&raw mut wcarry).cast(),
        (&raw mut turtleprox).cast(),
        (&raw mut raftprox).cast(),
        (&raw mut brave).cast(),
        (&raw mut luck).cast(),
        (&raw mut kind).cast(),
        (&raw mut wealth).cast(),
        (&raw mut hunger).cast(),
        (&raw mut fatigue).cast(),
        (&raw mut brother).cast(),
        (&raw mut princess).cast(),
        (&raw mut hero_sector).cast(),
        &raw mut hero_place,
        &raw mut daynight,
        &raw mut lightlevel,
        (&raw mut actor_file).cast(),
        (&raw mut set_file).cast(),
        (&raw mut active_carrier).cast(),
        &raw mut xtype,
        (&raw mut leader).cast(),
        (&raw mut secret_timer).cast(),
        (&raw mut light_timer).cast(),
        (&raw mut freeze_timer).cast(),
        (&raw mut cmode).cast(),
        (&raw mut encounter_type).cast(),
        &raw mut pad1,
        &raw mut pad2,
        &raw mut pad3,
    ];
    for v in variables {
        sl_u16(v);
    }
}

/// Saves or loads one actor.
pub fn sl_shape(a: *mut Shape) {
    unsafe {
        let a = &mut *a;
        sl_u16(&mut a.abs_x);
        sl_u16(&mut a.abs_y);
        sl_u16(&mut a.rel_x);
        sl_u16(&mut a.rel_y);
        sl_u8((&mut a.kind as *mut i8).cast());
        sl_u8(&mut a.race);
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
            sl_u8((byte as *mut i8).cast());
        }
        sl_u16((&mut a.vitality as *mut i16).cast());
        sl_u8((&mut a.vel_x as *mut i8).cast());
        sl_u8((&mut a.vel_y as *mut i8).cast());
    }
}

/// Saves or loads one extent (a special rectangle of the world).
pub fn sl_extent(e: *mut Extent) {
    unsafe {
        let e = &mut *e;
        sl_u16(&mut e.x1);
        sl_u16(&mut e.y1);
        sl_u16(&mut e.x2);
        sl_u16(&mut e.y2);
        sl_u8(&mut e.etype);
        sl_u8(&mut e.v1);
        sl_u8(&mut e.v2);
        sl_u8(&mut e.v3);
    }
}

/// Saves or loads one object (an item or person in the world).
pub fn sl_object(o: *mut WorldObject) {
    unsafe {
        let o = &mut *o;
        sl_u16(&mut o.xc);
        sl_u16(&mut o.yc);
        sl_u8((&mut o.ob_id as *mut i8).cast());
        sl_u8((&mut o.ob_stat as *mut i8).cast());
    }
}

/// Saves or loads one missile (arrow, rock, fireball). Called from `mod1save` in csrc/fmain.c.
pub fn sl_missile(m: *mut Missile) {
    unsafe {
        let m = &mut *m;
        sl_u16(&mut m.abs_x);
        sl_u16(&mut m.abs_y);
        for byte in [
            &mut m.missile_type,
            &mut m.time_of_flight,
            &mut m.speed,
            &mut m.direction,
            &mut m.archer,
        ] {
            sl_u8((byte as *mut i8).cast());
        }
    }
}

/// Size of one brother's item list.
const STUFF_SIZE: i32 = 35;

/// Saves or loads the three brothers' item lists and the 6 missiles (the count `mdex` is saved
/// elsewhere). After loading, `stuff` points at the current brother's list.
pub fn mod1save() {
    unsafe {
        saveload((&raw mut julstuff).cast(), STUFF_SIZE);
        saveload((&raw mut philstuff).cast(), STUFF_SIZE);
        saveload((&raw mut kevstuff).cast(), STUFF_SIZE);
        stuff = blist[brother as usize - 1].stuff;
        for i in 0..6 {
            sl_missile((&raw mut missile_list).cast::<Missile>().add(i));
        }
    }
}
