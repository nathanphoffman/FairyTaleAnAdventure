//! Which part of the world is loaded (from csrc/fmain.c): working out the region the camera is in
//! and loading its map, tiles and terrain from disk.
//!
//! The world is split into 8 outdoor regions (a 2 x 4 grid of 64 x 32 sector blocks) plus two
//! indoor ones (8: buildings, 9: dungeons and caves). `region_num` is the region now loaded;
//! `new_region` is the one being loaded, or `NO_REGION` when nothing is waiting to load.

use super::asm_map::fill_minimap;
use super::game_tables::{loaded_blocks, region_disk_blocks};
use super::globals::{image_mem, map_mem, sector_mem, terrain_mem};
use super::hdrive::{
    is_read_pending, load_track_range, mark_read_unused, motor_off, read_finished, wait_for_read,
};
use super::state::{the_hero, the_world};
use super::types::Need;
use std::ffi::c_void;

/// `new_region` value meaning no region is waiting to be loaded.
const NO_REGION: u16 = 10;
/// First region number that is indoors.
const FIRST_INDOOR_REGION: u16 = 8;
/// Block on the disk where the terrain tables start.
const TERRA_BLOCK: i16 = 149;
/// Bytes in one bitplane of 256 tiles, and of 64 tiles.
const IMAGE_PLANE_SIZE: usize = 16384;
const QUARTER_PLANE_SIZE: usize = 4096;
/// Blocks for the sector data, the region map and one image plane.
const SECTOR_BLOCKS: i16 = 64;
const REGION_MAP_BLOCKS: i16 = 8;
const IMAGE_PLANE_BLOCKS: i16 = 8;
/// Disk request numbers for each kind of data (see `load_track_range`).
const REQ_SECTOR: i16 = 0;
const REQ_MAP: i16 = 0;
const REQ_TERRA1: i16 = 1;
const REQ_TERRA2: i16 = 2;
const REQ_IMAGE_FIRST: i16 = 3;
/// Terrain tables are one block (512 bytes) each; the second goes after the first.
const TERRA_TABLE_SIZE: usize = 512;

/// The desert region and the inventory slot of the golden statue count. Until the hero has five
/// statues, the door into the desert fort is blocked by four map tiles (254) at this spot.
const DESERT_REGION: u16 = 4;
const STATUE_SLOT: usize = 25;
const STATUES_NEEDED: u8 = 5;
const DESERT_DOOR_TILE: usize = (11 * 128) + 26;
const BLOCKED_TILE: u8 = 254;

/// Works out which region the camera is in and starts loading it if it changed; sets `region_origin_x` and
/// `region_origin_y` (where the loaded region starts, in sectors); then fills `minimap` for the screen.
///
/// Outdoors the region comes from the middle of the screen: bit 6 of the sector column gives
/// west or east, and two bits of the sector row give which of four rows.
pub fn refresh_minimap() {
    unsafe {
        if the_world.region < FIRST_INDOOR_REGION {
            if the_world.new_region < NO_REGION {
                let xs = the_world.map_x as u32 & 0x3fff;
                let ys = the_world.map_y as u32 & 0x1fff;
                if (ys > 109 && ys < 7936) && (xs > 16156 || xs < 16128) {
                    load_region_now();
                }
            }
            let xs = (the_world.map_x as u32 + 151) >> 8;
            let ys = (the_world.map_y as u32 + 64) >> 8;
            let xr = (xs >> 6) & 1;
            let yr = (ys >> 5) & 3;
            the_world.target_region = (xr + yr + yr) as u16;
            if the_world.target_region != the_world.region {
                the_world.new_region = the_world.target_region;
                load_region_now();
            }
        } else {
            the_world.target_region = the_world.region;
        }

        if (read_finished(2) != 0 && read_finished(0) != 0) || the_world.new_region >= NO_REGION {
            let yr = the_world.target_region >> 1;
            let xr = if the_world.target_region > 7 {
                0
            } else {
                the_world.target_region & 1
            };
            the_world.origin_x = xr << 6;
            the_world.origin_y = yr << 5;
        }

        fill_minimap(the_world.tile_x as i32, the_world.tile_y as i32);
    }
}

/// Loads the pending region completely (`load_region_piece` does one piece per call).
pub fn load_region_now() {
    unsafe {
        while the_world.new_region < NO_REGION {
            load_region_piece();
        }
    }
}

/// Loads the next missing piece of the region in `new_region`: sector data, region map, the two
/// terrain tables, then one image bank per call. Each piece is only read if the loaded one is
/// different (`loaded_blocks`). When everything is there, waits for the reads, turns the drive
/// off, and `new_region` becomes the current region.
pub fn load_region_piece() {
    unsafe {
        if the_world.new_region >= NO_REGION {
            return;
        }
        let need = &region_disk_blocks[the_world.new_region as usize];
        if need.sector != loaded_blocks.sector {
            load_track_range(
                need.sector as i16,
                SECTOR_BLOCKS,
                sector_mem.cast::<c_void>(),
                REQ_SECTOR,
            );
            loaded_blocks.sector = need.sector;
        }
        if need.region != loaded_blocks.region {
            load_track_range(
                need.region as i16,
                REGION_MAP_BLOCKS,
                map_mem.cast::<c_void>(),
                REQ_MAP,
            );
            loaded_blocks.region = need.region;
        }
        if need.terra1 != loaded_blocks.terra1 {
            load_track_range(
                TERRA_BLOCK + need.terra1 as i16,
                1,
                terrain_mem.cast::<c_void>(),
                REQ_TERRA1,
            );
            loaded_blocks.terra1 = need.terra1;
        }
        if need.terra2 != loaded_blocks.terra2 {
            let dest = terrain_mem.add(TERRA_TABLE_SIZE).cast::<c_void>();
            load_track_range(TERRA_BLOCK + need.terra2 as i16, 1, dest, REQ_TERRA2);
            loaded_blocks.terra2 = need.terra2;
        }
        if load_next_image_bank(need) {
            return;
        }
        block_desert_door_without_statues();
        wait_for_all_reads();
        motor_off();
        the_world.region = the_world.new_region;
        the_world.new_region = NO_REGION;
    }
}

/// Loads the first of the four image banks that differs from the loaded one. Returns true if it
/// loaded one (only one bank per call, so the game keeps running), false if all are up to date.
unsafe fn load_next_image_bank(need: &Need) -> bool {
    unsafe {
        let mut bank_mem = image_mem;
        for bank in 0..4 {
            if need.image[bank] != loaded_blocks.image[bank] {
                let first = need.image[bank] as i16;
                for plane in 0..5 {
                    let dest = bank_mem.add(plane * IMAGE_PLANE_SIZE).cast::<c_void>();
                    let block = first + plane as i16 * IMAGE_PLANE_BLOCKS;
                    load_track_range(
                        block,
                        IMAGE_PLANE_BLOCKS,
                        dest,
                        REQ_IMAGE_FIRST + plane as i16,
                    );
                }
                loaded_blocks.image[bank] = need.image[bank];
                return true;
            }
            bank_mem = bank_mem.add(QUARTER_PLANE_SIZE);
        }
        false
    }
}

/// Until the hero has enough statues, the desert fort door is covered with blocked tiles.
unsafe fn block_desert_door_without_statues() {
    unsafe {
        if the_world.new_region == DESERT_REGION
            && *the_hero.stuff.add(STATUE_SLOT) < STATUES_NEEDED
        {
            for offset in [0, 1, 128, 129] {
                *map_mem.add(DESERT_DOOR_TILE + offset) = BLOCKED_TILE;
            }
        }
    }
}

/// Waits for every loading request still running and marks them all unused.
fn wait_for_all_reads() {
    for request in 0..7 {
        if is_read_pending(request) != 0 {
            wait_for_read(request);
        }
        mark_read_unused(request);
    }
}
