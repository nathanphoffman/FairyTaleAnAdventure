//! Opening doors with keys (from csrc/fmain.c).
//!
//! A door on the map is a few tiles that look like a door. Walking into one with a key (or none
//! needed) swaps those tiles for the "open" ones. `open_list` says, per door and per map set,
//! which tiles to swap in and which key is needed.

use crate::game::data::state::{the_screen, the_ui};
use crate::game::data::tables::loaded_blocks;
use crate::game::ui::messages::print;
use crate::game::world::map::{terrain_at, tile_address};

/// Set after a locked door was bumped, so "It's locked." is only said once until the hero moves on.
pub static mut bumped_locked_door: i8 = 0;

/// The terrain type that marks a door.
const DOOR_TERRAIN: i32 = 15;

/// Value of `view_status` that makes the whole map redraw.
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

/// A door is looked for this far (pixels) to the right and left, and a tile is 16 pixels wide and
/// 32 high (shifts 4 and 5). Each map set covers 64 sector numbers (a shift of 6).
const DOOR_SEARCH_SIDE: u16 = 4;
const TILE_WIDTH: u16 = 16;
const TILE_HEIGHT: u16 = 32;
const TILE_WIDTH_SHIFT: u32 = 4;
const TILE_HEIGHT_SHIFT: u32 = 5;
const MAP_SET_SHIFT: u32 = 6;

/// A door can be up to three tiles wide, so at most two steps to its left edge.
const DOOR_MAX_LEFT_STEPS: usize = 2;

/// The tiles of an opened cabinet: the open door's top, and the three around it.
const CABINET_TILES: [u8; 3] = [87, 86, 88];

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
    tile_address(x, y).cast_mut()
}

/// Tries to open a door near the pixel (x, y) with a key of type `keytype` (0 when walking into it
/// without a key). Looks 4 pixels either side for the door terrain, then walks left and down to
/// the door's top-left tile. Returns 1 if a door opened, 0 if not ("It's locked." is said the
/// first time).
pub fn open_door_near(x: u16, y: u16, keytype: u32) -> i32 {
    unsafe {
        let Some((x, y)) = find_door_corner(x, y) else {
            return 0;
        };

        // we know where the door starts; now which type is it?
        let sector_id = *tile(x, y);
        let map_id = loaded_blocks.image[(sector_id >> MAP_SET_SHIFT) as usize];

        if let Some(open) = find_open(sector_id, map_id, keytype) {
            swap_in_open_door_tiles(open, x, y);
            the_screen.view_status = REDRAW_MAP;
            the_ui.held_direction = 0;
            print(c"It opened.".as_ptr());

            return 1;
        }

        if bumped_locked_door == 0 && keytype == 0 {
            print(c"It's locked.".as_ptr());
        }

        bumped_locked_door = 1;

        0
    }
}

/// Looks for a door tile at the point, 4 pixels right of it, or 4 pixels left of it, then moves to
/// the door's top-left tile. Returns that tile's column and row, or `None` if there is no door.
fn find_door_corner(x: u16, y: u16) -> Option<(i32, i32)> {
    let mut x = x;
    let mut y = y;
    let mut found = terrain_at(x as i32, y as i32) == DOOR_TERRAIN;

    if !found {
        x = x.wrapping_add(DOOR_SEARCH_SIDE);
        found = terrain_at(x as i32, y as i32) == DOOR_TERRAIN;
    }

    if !found {
        x = x.wrapping_sub(2 * DOOR_SEARCH_SIDE);
        found = terrain_at(x as i32, y as i32) == DOOR_TERRAIN;
    }

    if !found {
        return None;
    }

    // A door can be up to three tiles wide; step to its left edge.
    for _ in 0..DOOR_MAX_LEFT_STEPS {
        if terrain_at(x as i32 - TILE_WIDTH as i32, y as i32) == DOOR_TERRAIN {
            x = x.wrapping_sub(TILE_WIDTH);
        }
    }

    if terrain_at(x as i32, y as i32 + TILE_HEIGHT as i32) == DOOR_TERRAIN {
        y = y.wrapping_add(TILE_HEIGHT);
    }

    Some((
        (x >> TILE_WIDTH_SHIFT) as i32,
        (y >> TILE_HEIGHT_SHIFT) as i32,
    ))
}

/// Replaces the closed door tile at (`x`, `y`) with its open version, and the neighbouring tiles
/// the door covers (which ones depends on the door's layout).
unsafe fn swap_in_open_door_tiles(open: &DoorOpen, x: i32, y: i32) {
    unsafe {
        let (changes, count) = open_changes(open, x, y);

        for &(tx, ty, value) in &changes[..count] {
            *tile(tx, ty) = value;
        }
    }
}

/// The door kind that `sector_id` (in map set `map_id`) is, if the key `keytype` opens it (a door
/// that needs no key opens for anything). A door of the right kind with the wrong key is skipped.
fn find_open(sector_id: u8, map_id: u16, keytype: u32) -> Option<&'static DoorOpen> {
    OPEN_LIST.iter().find(|open| {
        open.map_id == map_id
            && open.door_id == sector_id
            && (open.keytype == NOKEY || open.keytype as u32 == keytype)
    })
}

/// The tiles an opened door changes, as (x, y, new tile), and how many of them are used. The
/// door's top-left tile is (x, y).
fn open_changes(open: &DoorOpen, x: i32, y: i32) -> ([(i32, i32, u8); 4], usize) {
    let mut changes = [(x, y, open.new1), (0, 0, 0), (0, 0, 0), (0, 0, 0)];
    let mut count = 1;
    let second = open.new2;

    if second != 0 {
        match open.above {
            ABOVE => {
                changes[1] = (x, y - 1, second);
                count = 2;
            }

            BACK => {
                changes[1] = (x - 1, y, second);
                count = 2;
            }

            CABINET_LAYOUT => {
                changes[1] = (x, y - 1, CABINET_TILES[0]);
                changes[2] = (x + 1, y, CABINET_TILES[1]);
                changes[3] = (x + 1, y - 1, CABINET_TILES[2]);
                count = 4;
            }

            layout => {
                changes[1] = (x + 1, y, second);
                count = 2;

                if layout != SIDE {
                    changes[2] = (x + 2, y, layout);
                    count = 3;
                }
            }
        }
    }

    (changes, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_must_match_unless_none_is_needed() {
        assert!(find_open(64, 360, GREEN as u32).is_some());
        assert!(find_open(64, 360, 0).is_none());
        assert!(find_open(64, 360, GOLD as u32).is_none());

        // A wooden door needs no key: walking into it (key 0) opens it.
        assert_eq!(find_open(120, 360, 0).unwrap().new1, 125);

        // The same door tile in another map set is a different door.
        assert_eq!(find_open(64, 280, GREY as u32).unwrap().new1, 124);
        assert!(find_open(64, 999, GREEN as u32).is_none());
    }

    #[test]
    fn open_tile_layouts() {
        let (c, n) = open_changes(find_open(64, 360, GREEN as u32).unwrap(), 10, 5);
        assert_eq!(&c[..n], &[(10, 5, 123), (11, 5, 124)]);
        let (c, n) = open_changes(find_open(122, 360, 0).unwrap(), 10, 5);
        assert_eq!(&c[..n], &[(10, 5, 127)]);
        let (c, n) = open_changes(find_open(128, 240, WHITE as u32).unwrap(), 10, 5);
        assert_eq!(&c[..n], &[(10, 5, 154), (10, 4, 155)]);
        let (c, n) = open_changes(find_open(25, 680, GOLD as u32).unwrap(), 10, 5);
        assert_eq!(&c[..n], &[(10, 5, 27), (9, 5, 26)]);
        let (c, n) = open_changes(find_open(136, 800, GOLD as u32).unwrap(), 10, 5);
        assert_eq!(&c[..n], &[(10, 5, 133), (11, 5, 134), (12, 5, 135)]);
        let (c, n) = open_changes(find_open(165, 800, GREEN as u32).unwrap(), 10, 5);
        assert_eq!(
            &c[..n],
            &[(10, 5, 85), (10, 4, 87), (11, 5, 86), (11, 4, 88)]
        );
    }
}
