//! Which part of the world is loaded (from csrc/fmain.c): working out the region the camera is in
//! and loading its map, tiles and terrain from disk.
//!
//! The world is split into 8 outdoor regions (a 2 x 4 grid of 64 x 32 sector blocks) plus two
//! indoor ones (8: buildings, 9: dungeons and caves). `region_num` is the region now loaded;
//! `new_region` is the one being loaded, or `NO_REGION` when nothing is waiting to load.

use super::hdrive::{CheckDiskIO, InvalidDiskIO, IsReadDiskIO, WaitDiskIO, load_track_range, motor_off};
use super::types::Need;
use std::ffi::c_void;

unsafe extern "C" {
    static map_x: u16;
    static map_y: u16;
    static img_x: u16;
    static img_y: u16;
    static mut xreg: u16;
    static mut yreg: u16;
    static mut new_region: u16;
    static mut lregion: u16;
    static mut region_num: u16;
    static mut current_loads: Need;
    static file_index: [Need; 10];
    static image_mem: *mut u8;
    static sector_mem: *mut u8;
    static map_mem: *mut u8;
    static terra_mem: *mut u8;
    static stuff: *mut u8;

    fn genmini(img_x: i32, img_y: i32);
}

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

/// Works out which region the camera is in and starts loading it if it changed; sets `xreg` and
/// `yreg` (where the loaded region starts, in sectors); then fills `minimap` for the screen.
///
/// Outdoors the region comes from the middle of the screen: bit 6 of the sector column gives
/// west or east, and two bits of the sector row give which of four rows.
#[unsafe(no_mangle)]
pub extern "C" fn gen_mini() {
    unsafe {
        if region_num < FIRST_INDOOR_REGION {
            if new_region < NO_REGION {
                let xs = map_x as u32 & 0x3fff;
                let ys = map_y as u32 & 0x1fff;
                if (ys > 109 && ys < 7936) && (xs > 16156 || xs < 16128) {
                    load_all();
                }
            }
            let xs = (map_x as u32 + 151) >> 8;
            let ys = (map_y as u32 + 64) >> 8;
            let xr = (xs >> 6) & 1;
            let yr = (ys >> 5) & 3;
            lregion = (xr + yr + yr) as u16;
            if lregion != region_num {
                new_region = lregion;
                load_all();
            }
        } else {
            lregion = region_num;
        }

        if (CheckDiskIO(2) != 0 && CheckDiskIO(0) != 0) || new_region >= NO_REGION {
            let yr = lregion >> 1;
            let xr = if lregion > 7 { 0 } else { lregion & 1 };
            xreg = xr << 6;
            yreg = yr << 5;
        }

        genmini(img_x as i32, img_y as i32);
    }
}

/// Loads the pending region completely (`load_new_region` does one piece per call).
#[unsafe(no_mangle)]
pub extern "C" fn load_all() {
    unsafe {
        while new_region < NO_REGION {
            load_new_region();
        }
    }
}

/// Loads the next missing piece of the region in `new_region`: sector data, region map, the two
/// terrain tables, then one image bank per call. Each piece is only read if the loaded one is
/// different (`current_loads`). When everything is there, waits for the reads, turns the drive
/// off, and `new_region` becomes the current region.
#[unsafe(no_mangle)]
pub extern "C" fn load_new_region() {
    unsafe {
        if new_region >= NO_REGION {
            return;
        }
        let need = &file_index[new_region as usize];
        if need.sector != current_loads.sector {
            load_track_range(need.sector as i16, SECTOR_BLOCKS, sector_mem.cast::<c_void>(), REQ_SECTOR);
            current_loads.sector = need.sector;
        }
        if need.region != current_loads.region {
            load_track_range(need.region as i16, REGION_MAP_BLOCKS, map_mem.cast::<c_void>(), REQ_MAP);
            current_loads.region = need.region;
        }
        if need.terra1 != current_loads.terra1 {
            load_track_range(TERRA_BLOCK + need.terra1 as i16, 1, terra_mem.cast::<c_void>(), REQ_TERRA1);
            current_loads.terra1 = need.terra1;
        }
        if need.terra2 != current_loads.terra2 {
            let dest = terra_mem.add(TERRA_TABLE_SIZE).cast::<c_void>();
            load_track_range(TERRA_BLOCK + need.terra2 as i16, 1, dest, REQ_TERRA2);
            current_loads.terra2 = need.terra2;
        }
        let mut bank_mem = image_mem;
        for bank in 0..4 {
            if need.image[bank] != current_loads.image[bank] {
                let first = need.image[bank] as i16;
                for plane in 0..5 {
                    let dest = bank_mem.add(plane * IMAGE_PLANE_SIZE).cast::<c_void>();
                    let block = first + plane as i16 * IMAGE_PLANE_BLOCKS;
                    load_track_range(block, IMAGE_PLANE_BLOCKS, dest, REQ_IMAGE_FIRST + plane as i16);
                }
                current_loads.image[bank] = need.image[bank];
                return;
            }
            bank_mem = bank_mem.add(QUARTER_PLANE_SIZE);
        }

        if new_region == DESERT_REGION && *stuff.add(STATUE_SLOT) < STATUES_NEEDED {
            for offset in [0, 1, 128, 129] {
                *map_mem.add(DESERT_DOOR_TILE + offset) = BLOCKED_TILE;
            }
        }

        for request in 0..7 {
            if IsReadDiskIO(request) != 0 {
                WaitDiskIO(request);
            }
            InvalidDiskIO(request);
        }

        motor_off();
        region_num = new_region;
        new_region = NO_REGION;
    }
}
