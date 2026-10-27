//! Looking things up in the map: the sector at a place, the terrain under a point, collision probes and
//! the minimap.

use super::drawing::{SCREEN_COLUMNS, SCREEN_ROWS};
use crate::game::data::object_tables::minimap;
use crate::game::data::state::{the_hero, the_memory, the_world};

/// Sectors are 16 x 8 tiles of 128 bytes; the region map has 128 sector bytes per row.
pub(super) const SECTOR_BYTES: usize = 128;
pub(super) const MAP_ROW_BYTES: usize = 128;
pub(super) const SECTOR_ROWS_IN_REGION: i16 = 32;

/// Tiles are 16 pixels wide and 32 high; a sector is 16 tiles wide and 8 high. These are the shifts
/// and masks that go between pixels, tiles and sectors.
const TILE_WIDTH_SHIFT: u32 = 4;
const TILE_HEIGHT_SHIFT: u32 = 5;
const SECTOR_WIDTH_SHIFT: u32 = 4;
const SECTOR_HEIGHT_SHIFT: u32 = 3;
const TILE_COLUMN_MASK: usize = 15;
const TILE_ROW_MASK: usize = 7;
const SECTOR_ROW_TILES: usize = 16;

/// A sector row of the region map is 128 bytes: the shift that multiplies by it.
const MAP_ROW_SHIFT: u32 = 7;

/// A sector column outside the map: bit 6 set means off the edge, and bit 5 says which edge.
const COLUMN_OFF_MAP: u16 = 64;
const COLUMN_OFF_LOW_SIDE: u16 = 32;
const LAST_SECTOR_COLUMN: u16 = 63;
const ROW_OFF_MAP: u16 = 32;
const ROW_OFF_LOW_SIDE: u16 = 16;
const LAST_SECTOR_ROW: u16 = 31;

/// A terrain table entry is 4 bytes: the second holds the type in its high nibble, the third the
/// mask of solid sub-areas, and the fourth the overview colour.
const TERRAIN_ENTRY_BYTES: usize = 4;
const TERRAIN_TYPE_BYTE: usize = 1;
const TERRAIN_MASK_BYTE: usize = 2;
const TERRAIN_TYPE_SHIFT: u32 = 4;

/// The 8 sub-areas of a tile are picked by one bit of the mask, from the pixel position.
const FIRST_SUBAREA_BIT: u8 = 0x80;
const RIGHT_HALF_PIXEL: u16 = 8;
const LOWER_ROW_PIXEL: u16 = 8;
const LOWER_PAIR_PIXEL: u16 = 16;
const RIGHT_HALF_SHIFT: u32 = 4;

/// Collision probes: how far right/left and down from the actor's position, and which terrain types
/// block on each side.
const PROBE_SIDE: i32 = 4;
const PROBE_DOWN: i32 = 2;
const ALWAYS_BLOCKS: i32 = 1;
const RIGHT_PROBE_BLOCKS_FROM: i32 = 10;
const LEFT_PROBE_BLOCKS_FROM: i32 = 8;

/// The minimap covers the world's 2048 tile columns, and 32768 tile rows.
const TILE_COLUMN_WRAP: u16 = 0x07ff;
const TILE_ROW_WRAP: u16 = 0x7fff;

/// Regions above this number use the second half of the sector table.
const LAST_FIRST_HALF_REGION: u16 = 7;
const REGION_BYTE_MASK: u16 = 0xff;
const SECOND_HALF_OFFSET: i32 = 256;

/// The high byte of the hero's position is the sector.
const SECTOR_POSITION_SHIFT: u32 = 8;

/// Sector column from an image column: values outside 0-63 wrap, as in the assembly.
pub(super) fn wrap_secx(mut v: u16) -> u16 {
    if v & COLUMN_OFF_MAP != 0 {
        v = if v & COLUMN_OFF_LOW_SIDE != 0 {
            0
        } else {
            LAST_SECTOR_COLUMN
        };
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
    let mut bit = FIRST_SUBAREA_BIT;

    if ux & RIGHT_HALF_PIXEL != 0 {
        // The right half of the tile: the second column of sub-areas.
        bit >>= RIGHT_HALF_SHIFT;
    }

    if uy & LOWER_ROW_PIXEL != 0 {
        // The lower half of a 16 pixel row of sub-areas.
        bit >>= 1;
    }

    if uy & LOWER_PAIR_PIXEL != 0 {
        // The lower two of the four rows of sub-areas.
        bit >>= 2;
    }

    // The tile column and row: tiles are 16 pixels wide and 32 high.
    let (imx, imy) = (ux >> TILE_WIDTH_SHIFT, uy >> TILE_HEIGHT_SHIFT);
    unsafe {
        // A sector is 16 tiles wide.
        let col = wrap_secx((imx >> SECTOR_WIDTH_SHIFT).wrapping_sub(the_world.origin_x));
        // A sector is 8 tiles high.
        let row = clamp_secy((imy >> SECTOR_HEIGHT_SHIFT).wrapping_sub(the_world.origin_y) as i16);
        let sec = sector_at(col, row) as usize;
        let tile = *the_memory.sector_mem.add(
            (sec * SECTOR_BYTES
                + (imy as usize & TILE_ROW_MASK) * SECTOR_ROW_TILES
                + (imx as usize & TILE_COLUMN_MASK)) as u16 as usize,
        ) as usize;

        if *the_memory
            .terrain_mem
            .add(tile * TERRAIN_ENTRY_BYTES + TERRAIN_MASK_BYTE)
            & bit
            == 0
        {
            return 0;
        }

        // The terrain type is the high nibble of the second byte of the tile's entry.
        (*the_memory
            .terrain_mem
            .add(tile * TERRAIN_ENTRY_BYTES + TERRAIN_TYPE_BYTE)
            >> TERRAIN_TYPE_SHIFT) as i32
    }
}

/// Collision test with two probes either side of the character (4 pixels right and left, 2 down).
/// Returns the blocking terrain type, 0 if the way is clear. Type 1 always blocks; the right-hand
/// probe is blocked by 10 and above, the left-hand one by 8 and above.
pub fn probe_collision(x: i32, y: i32) -> i32 {
    let t = terrain_at(x + PROBE_SIDE, y + PROBE_DOWN);

    if t == ALWAYS_BLOCKS || t >= RIGHT_PROBE_BLOCKS_FROM {
        return t;
    }

    let t = terrain_at(x - PROBE_SIDE, y + PROBE_DOWN);

    if t == ALWAYS_BLOCKS || t >= LEFT_PROBE_BLOCKS_FROM {
        return t;
    }

    0
}

/// Address of the tile byte for a tile position (wraps vertically too).
pub fn tile_address(imx: i32, imy: i32) -> *const u8 {
    let (ux, uy) = (imx as u16, imy as u16);
    unsafe {
        // Pixel column to sector column: 16 pixels to a tile, 16 tiles to a sector.
        let col = wrap_secx((ux >> TILE_WIDTH_SHIFT).wrapping_sub(the_world.origin_x));
        // Pixel row to sector row: 8 tiles to a sector, so 8 times 32 divided by 32.
        let mut row = (uy >> SECTOR_HEIGHT_SHIFT).wrapping_sub(the_world.origin_y);

        if row & ROW_OFF_MAP != 0 {
            row = if row & ROW_OFF_LOW_SIDE != 0 {
                0
            } else {
                LAST_SECTOR_ROW
            };
        }

        let sec = sector_at(col, row) as usize;

        the_memory.sector_mem.add(
            (sec * SECTOR_BYTES
                + (uy as usize & TILE_ROW_MASK) * SECTOR_ROW_TILES
                + (ux as usize & TILE_COLUMN_MASK)) as u16 as usize,
        )
    }
}

/// Fills `minimap` (19 columns of 6 tile numbers, column by column) for the area starting at
/// tile (tile_x, tile_y), and works out which sector the hero is in.
pub fn fill_minimap(tile_x: i32, tile_y: i32) {
    unsafe {
        let mut x = tile_x as u16;
        let mut next = 0;

        for _ in 0..SCREEN_COLUMNS {
            x &= TILE_COLUMN_WRAP;
            // Tile column to sector column: 16 tiles to a sector.
            let col = wrap_secx((x >> SECTOR_WIDTH_SHIFT).wrapping_sub(the_world.origin_x))
                .wrapping_add(the_world.origin_x);

            for j in 0..SCREEN_ROWS as i32 {
                let y = (tile_y + j) as u16 & TILE_ROW_WRAP;
                // Tile row to sector row: 8 tiles to a sector.
                let row =
                    clamp_secy((y >> SECTOR_HEIGHT_SHIFT).wrapping_sub(the_world.origin_y) as i16);
                let sec = *the_memory
                    .map_mem
                    // The region map has 128 sectors in each row.
                    .add(
                        ((row as usize) << MAP_ROW_SHIFT).wrapping_add(col as usize) as u16
                            as usize,
                    ) as usize;
                let tile = *the_memory.sector_mem.add(
                    (sec * SECTOR_BYTES
                        + (y as usize & TILE_ROW_MASK) * SECTOR_ROW_TILES
                        + (x as usize & TILE_COLUMN_MASK)) as u16 as usize,
                );

                minimap[next] = tile as i16;
                next += 1;
            }

            x += 1;
        }

        // the assembly read the high byte of hero_x and hero_y (68000 is big-endian)
        let hx = (the_hero.x >> SECTOR_POSITION_SHIFT) as i32;
        // The hero's sector row: 256 pixels to a sector.
        let hy =
            ((the_hero.y >> SECTOR_POSITION_SHIFT).wrapping_sub(the_world.origin_y)) as i16 as i32;
        let mut sec = *the_memory
            .map_mem
            // The region map has 128 sectors in each row.
            .add(((hy << MAP_ROW_SHIFT).wrapping_add(hx)) as u16 as usize)
            as i32;

        if the_world.region & REGION_BYTE_MASK > LAST_FIRST_HALF_REGION {
            sec += SECOND_HALF_OFFSET;
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
