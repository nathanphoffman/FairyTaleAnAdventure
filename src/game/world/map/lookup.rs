//! Looking things up in the map: the sector at a place, the terrain under a point, collision probes and
//! the minimap.

use super::drawing::{SCREEN_COLUMNS, SCREEN_ROWS};
use crate::game::data::object_tables::minimap;
use crate::game::data::state::{the_hero, the_memory, the_world};

/// Sectors are 16 x 8 tiles of 128 bytes; the region map has 128 sector bytes per row.
pub(super) const SECTOR_BYTES: usize = 128;
pub(super) const MAP_ROW_BYTES: usize = 128;
pub(super) const SECTOR_ROWS_IN_REGION: i16 = 32;

/// Sector column from an image column: values outside 0-63 wrap, as in the assembly.
pub(super) fn wrap_secx(mut v: u16) -> u16 {
    if v & 64 != 0 {
        v = if v & 32 != 0 { 0 } else { 63 };
    }

    v
}

/// Sector row from an image row; rows outside 0-31 are clamped.
pub(super) fn clamp_secy(v: i16) -> u16 {
    v.clamp(0, SECTOR_ROWS_IN_REGION - 1) as u16
}

/// The sector number for a sector column and row of the region map.
pub(super) unsafe fn sector_at(col: u16, row: u16) -> u8 {
    unsafe {
        *the_memory.map_mem.add(
            (row as usize * MAP_ROW_BYTES + col as usize + the_world.origin_x as usize) as u16
                as usize,
        )
    }
}

/// Terrain type (0-15) at a pixel position, 0 if that sub-area is clear.
///
/// Each tile is divided into 8 sub-areas (2 across, 4 down); the terrain table has a bit mask
/// saying which of them are solid, so a tile can be partly walkable (a tree trunk in the middle of
/// a clearing, say). The bit is picked from pixel bits 3 (x), 3 and 4 (y).
pub fn terrain_at(x: i32, y: i32) -> i32 {
    let (ux, uy) = (x as u16, y as u16);
    let mut bit = 0x80u8;

    if ux & 8 != 0 {
        // The right half of the tile: the second column of sub-areas.
        bit >>= 4;
    }

    if uy & 8 != 0 {
        // The lower half of a 16 pixel row of sub-areas.
        bit >>= 1;
    }

    if uy & 16 != 0 {
        // The lower two of the four rows of sub-areas.
        bit >>= 2;
    }

    // The tile column and row: tiles are 16 pixels wide and 32 high.
    let (imx, imy) = (ux >> 4, uy >> 5);
    unsafe {
        // A sector is 16 tiles wide.
        let col = wrap_secx((imx >> 4).wrapping_sub(the_world.origin_x));
        // A sector is 8 tiles high.
        let row = clamp_secy((imy >> 3).wrapping_sub(the_world.origin_y) as i16);
        let sec = sector_at(col, row) as usize;
        let tile = *the_memory.sector_mem.add(
            (sec * SECTOR_BYTES + (imy as usize & 7) * 16 + (imx as usize & 15)) as u16 as usize,
        ) as usize;

        if *the_memory.terrain_mem.add(tile * 4 + 2) & bit == 0 {
            return 0;
        }

        // The terrain type is the high nibble of the second byte of the tile's entry.
        (*the_memory.terrain_mem.add(tile * 4 + 1) >> 4) as i32
    }
}

/// Collision test with two probes either side of the character (4 pixels right and left, 2 down).
/// Returns the blocking terrain type, 0 if the way is clear. Type 1 always blocks; the right-hand
/// probe is blocked by 10 and above, the left-hand one by 8 and above.
pub fn probe_collision(x: i32, y: i32) -> i32 {
    let t = terrain_at(x + 4, y + 2);

    if t == 1 || t >= 10 {
        return t;
    }

    let t = terrain_at(x - 4, y + 2);

    if t == 1 || t >= 8 {
        return t;
    }

    0
}

/// Address of the tile byte for a tile position (wraps vertically too).
pub fn tile_address(imx: i32, imy: i32) -> *const u8 {
    let (ux, uy) = (imx as u16, imy as u16);
    unsafe {
        // Pixel column to sector column: 16 pixels to a tile, 16 tiles to a sector.
        let col = wrap_secx((ux >> 4).wrapping_sub(the_world.origin_x));
        // Pixel row to sector row: 8 tiles to a sector, so 8 times 32 divided by 32.
        let mut row = (uy >> 3).wrapping_sub(the_world.origin_y);

        if row & 32 != 0 {
            row = if row & 16 != 0 { 0 } else { 31 };
        }

        let sec = sector_at(col, row) as usize;

        the_memory
            .sector_mem
            .add((sec * SECTOR_BYTES + (uy as usize & 7) * 16 + (ux as usize & 15)) as u16 as usize)
    }
}

/// Fills `minimap` (19 columns of 6 tile numbers, column by column) for the area starting at
/// tile (tile_x, tile_y), and works out which sector the hero is in.
pub fn fill_minimap(tile_x: i32, tile_y: i32) {
    unsafe {
        let mut x = tile_x as u16;
        let mut next = 0;

        for _ in 0..SCREEN_COLUMNS {
            x &= 0x07ff;
            // Tile column to sector column: 16 tiles to a sector.
            let col = wrap_secx((x >> 4).wrapping_sub(the_world.origin_x))
                .wrapping_add(the_world.origin_x);

            for j in 0..SCREEN_ROWS as i32 {
                let y = (tile_y + j) as u16 & 0x7fff;
                // Tile row to sector row: 8 tiles to a sector.
                let row = clamp_secy((y >> 3).wrapping_sub(the_world.origin_y) as i16);
                let sec = *the_memory
                    .map_mem
                    // The region map has 128 sectors in each row.
                    .add(((row as usize) << 7).wrapping_add(col as usize) as u16 as usize)
                    as usize;
                let tile = *the_memory.sector_mem.add(
                    (sec * SECTOR_BYTES + (y as usize & 7) * 16 + (x as usize & 15)) as u16
                        as usize,
                );

                minimap[next] = tile as i16;
                next += 1;
            }

            x += 1;
        }

        // the assembly read the high byte of hero_x and hero_y (68000 is big-endian)
        let hx = (the_hero.x >> 8) as i32;
        // The hero's sector row: 256 pixels to a sector.
        let hy = ((the_hero.y >> 8).wrapping_sub(the_world.origin_y)) as i16 as i32;
        let mut sec = *the_memory
            .map_mem
            // The region map has 128 sectors in each row.
            .add(((hy << 7).wrapping_add(hx)) as u16 as usize) as i32;

        if the_world.region & 0xff > 7 {
            sec += 256;
        }

        the_hero.sector = sec as i16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_columns_wrap_outside_the_map() {
        assert_eq!(wrap_secx(5), 5);
        assert_eq!(wrap_secx(64), 63);
        assert_eq!(wrap_secx(96), 0);
    }

    #[test]
    fn sector_rows_are_clamped() {
        assert_eq!(clamp_secy(-3), 0);
        assert_eq!(clamp_secy(40), 31);
        assert_eq!(clamp_secy(7), 7);
    }
}
