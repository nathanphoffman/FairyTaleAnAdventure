//! Opening doors with keys (from csrc/fmain.c).
//!
//! A door on the map is a few tiles that look like a door. Walking into one with a key (or none
//! needed) swaps those tiles for the "open" ones. `open_list` says, per door and per map set,
//! which tiles to swap in and which key is needed.

use super::asm_map::{mapxy, px_to_im};
use super::messages::print;
use super::game_tables::{current_loads};
use super::globals::{keydir, viewstatus};


/// Set after a locked door was bumped, so "It's locked." is only said once until the hero moves on.
pub static mut bumped: i8 = 0;

/// The terrain type that marks a door.
const DOOR_TERRAIN: i32 = 15;
/// Value of `viewstatus` that makes the whole map redraw.
const REDRAW_MAP: i8 = 99;

// Key types (`enum ky` in csrc/fmain.c).
const NOKEY: u8 = 0;
const GOLD: u8 = 1;
const GREEN: u8 = 2;
const KBLUE: u8 = 3;
const RED: u8 = 4;
const GREY: u8 = 5;
const WHITE: u8 = 6;

/// How the second new tile is placed, relative to the first.
const ABOVE: u8 = 1;
const SIDE: u8 = 2;
const BACK: u8 = 3;
const CABINET_LAYOUT: u8 = 4;

/// One kind of door: `door_id` is the tile it starts as, in the map set `map_id`; it turns into
/// `new1` (and `new2` next to it, placed by `above`); `keytype` is the key that opens it.
struct DoorOpen {
    door_id: u8,
    map_id: u16,
    new1: u8,
    new2: u8,
    above: u8,
    keytype: u8,
}

const fn door(door_id: u8, map_id: u16, new1: u8, new2: u8, above: u8, keytype: u8) -> DoorOpen {
    DoorOpen {
        door_id,
        map_id,
        new1,
        new2,
        above,
        keytype,
    }
}

const OPEN_LIST: [DoorOpen; 17] = [
    door(64, 360, 123, 124, SIDE, GREEN),          // HSTONE
    door(120, 360, 125, 126, SIDE, NOKEY),         // HWOOD
    door(122, 360, 127, 0, 0, NOKEY),              // VWOOD
    door(64, 280, 124, 125, SIDE, GREY),           // HSTONE2
    door(77, 280, 126, 0, 0, GREY),                // VSTONE2
    door(82, 480, 84, 85, SIDE, KBLUE),            // CRYST
    door(64, 480, 105, 106, SIDE, GREEN),          // OASIS
    door(128, 240, 154, 155, ABOVE, WHITE),        // MARBLE
    door(39, 680, 41, 42, SIDE, GOLD),             // HGATE
    door(25, 680, 27, 26, BACK, GOLD),             // VGATE
    door(114, 760, 116, 117, ABOVE, RED),          // SECRET
    door(118, 760, 116, 117, ABOVE, GREY),         // TUNNEL
    door(136, 800, 133, 134, 135, GOLD),           // GOLDEN
    door(187, 800, 76, 77, SIDE, NOKEY),           // HSTON3
    door(73, 720, 75, 0, 0, NOKEY),                // VSTON3
    door(165, 800, 85, 86, CABINET_LAYOUT, GREEN), // CABINET
    door(210, 840, 208, 209, SIDE, NOKEY),         // BLUE
];

/// The tile byte at tile position (x, y) of the map, to change it.
fn tile(x: i32, y: i32) -> *mut u8 {
    mapxy(x, y).cast_mut()
}

/// Tries to open a door near the pixel (x, y) with a key of type `keytype` (0 when walking into it
/// without a key). Looks 4 pixels either side for the door terrain, then walks left and down to
/// the door's top-left tile. Returns 1 if a door opened, 0 if not ("It's locked." is said the
/// first time).
pub fn doorfind(x: u16, y: u16, keytype: u32) -> i32 {
    unsafe {
        let mut x = x;
        let mut y = y;
        let mut found = px_to_im(x as i32, y as i32) == DOOR_TERRAIN;
        if !found {
            x = x.wrapping_add(4);
            found = px_to_im(x as i32, y as i32) == DOOR_TERRAIN;
        }
        if !found {
            x = x.wrapping_sub(8);
            found = px_to_im(x as i32, y as i32) == DOOR_TERRAIN;
        }
        if !found {
            return 0;
        }

        if px_to_im(x as i32 - 16, y as i32) == DOOR_TERRAIN {
            x = x.wrapping_sub(16);
        }
        if px_to_im(x as i32 - 16, y as i32) == DOOR_TERRAIN {
            x = x.wrapping_sub(16);
        }
        if px_to_im(x as i32, y as i32 + 32) == DOOR_TERRAIN {
            y = y.wrapping_add(32);
        }
        let x = (x >> 4) as i32;
        let y = (y >> 5) as i32;

        // we know where the door starts; now which type is it?
        let sector_id = *tile(x, y);
        let map_id = current_loads.image[(sector_id >> 6) as usize];
        for open in OPEN_LIST.iter() {
            if open.map_id != map_id || open.door_id != sector_id {
                continue;
            }
            if open.keytype == NOKEY || open.keytype as u32 == keytype {
                *tile(x, y) = open.new1;
                let second = open.new2;
                if second != 0 {
                    match open.above {
                        ABOVE => *tile(x, y - 1) = second,
                        BACK => *tile(x - 1, y) = second,
                        CABINET_LAYOUT => {
                            *tile(x, y - 1) = 87;
                            *tile(x + 1, y) = 86;
                            *tile(x + 1, y - 1) = 88;
                        }
                        layout => {
                            *tile(x + 1, y) = second;
                            if layout != SIDE {
                                *tile(x + 2, y) = layout;
                            }
                        }
                    }
                }
                viewstatus = REDRAW_MAP;
                keydir = 0;
                print(c"It opened.".as_ptr());
                return 1;
            }
        }
        if bumped == 0 && keytype == 0 {
            print(c"It's locked.".as_ptr());
        }
        bumped = 1;
        0
    }
}
