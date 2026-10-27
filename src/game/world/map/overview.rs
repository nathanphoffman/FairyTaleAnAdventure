//! The overview map of the whole world (the magic map spell).

use super::drawing::PLANE_COUNT;
use super::lookup::{SECTOR_BYTES, SECTOR_ROWS_IN_REGION};
use crate::game::data::state::{the_memory, the_screen, the_world};

// The overview map (magic spell): each sector is drawn as 16 x 16 pixels, 18 x 9 sectors.
pub(super) const OVERVIEW_COLUMNS: i16 = 18;
pub(super) const OVERVIEW_ROWS: i16 = 9;

/// Sectors across and down the whole map.
pub(super) const MAP_SECTORS_WIDE: i16 = 64;
pub(super) const MAP_WRAP: i16 = 128;
pub(super) const SECTOR_PIXELS: usize = 16;
pub(super) const SECTOR_TILE_ROWS: usize = 8;

/// Bytes between the two copies of each overview line (it is drawn twice as tall).
pub(super) const OVERVIEW_LINE_COPY: usize = 40;
pub(super) const OVERVIEW_LINE_BYTES: usize = 80;

/// Bytes to skip at the end of each overview row of sectors.
pub(super) const OVERVIEW_ROW_SKIP: usize = 40 * 15 + 4;

/// A sector is 256 pixels (a shift of 8). The overview starts 9 sectors to the left and 4 above the
/// hero (half its size), and its start wraps around the 128 sector map when below 32 or above 96.
const SECTOR_SHIFT: u32 = 8;
const HALF_OVERVIEW_WIDTH: i32 = 9;
const HALF_OVERVIEW_HEIGHT: i32 = 4;
const WRAP_LOW: i16 = 32;
const WRAP_HIGH: i16 = 96;

/// The region map has 128 sectors in each row (a shift of 7).
const MAP_ROW_SHIFT: u32 = 7;

/// Each sector shows 16 x 8 pixels: a pixel row is 16 pixels, a line is two bytes, high byte first.
const BYTE_SHIFT: u32 = 8;
const TERRAIN_ENTRY_BYTES: usize = 4;
const TERRAIN_COLOR_BYTE: usize = 3;
const TILE_ROW_PIXELS: usize = 16;

/// The overview's top left sector for a map position, clamped to the map.
pub(super) fn overview_origin(mx: i32, my: i32, region_x: u16, region_y: u16) -> (i16, i16) {
    let mut x = (
        // Camera pixels to sectors (256 to a sector), then back 9 sectors to the left edge of the overview.
        ((mx as u16 >> SECTOR_SHIFT) as i32 - HALF_OVERVIEW_WIDTH - region_x as i32) as i16,
        // Camera pixels to sectors (256 to a sector), then back 4 sectors to the top edge of the overview.
        ((my as u16 >> SECTOR_SHIFT) as i32 - HALF_OVERVIEW_HEIGHT - region_y as i32) as i16,
    );

    for v in [&mut x.0, &mut x.1] {
        if *v < WRAP_LOW {
            *v += MAP_WRAP;
        }

        if *v > WRAP_HIGH {
            *v -= MAP_WRAP;
        }
    }

    (
        x.0.clamp(0, MAP_SECTORS_WIDE - OVERVIEW_COLUMNS),
        x.1.clamp(0, SECTOR_ROWS_IN_REGION - OVERVIEW_ROWS),
    )
}

/// Draws the overview map (the magic map spell): each sector is one 16 by 8 block of pixels drawn
/// twice as tall, 18 x 9 sectors across the screen. Each pixel's colour is the tile's overview
/// colour (terrain table byte 3). `sector_x` / `sector_y` tell the caller which sector is at the top left.
pub fn draw_overview_map(mx: i32, my: i32) {
    unsafe {
        let (x, y) = overview_origin(mx, my, the_world.origin_x, the_world.origin_y);
        the_world.sector_x = x as i32;
        the_world.sector_y = y as i32;
        let mut offset = 0usize;

        for j in 0..OVERVIEW_ROWS {
            for i in 0..OVERVIEW_COLUMNS {
                let index =
                    // Index into the region map: the sector row times 128 plus the column.
                    the_world.origin_x as i32 + x as i32 + i as i32 + (((y + j) as i32) << MAP_ROW_SHIFT);
                let sec = *the_memory.map_mem.add(index as u16 as usize) as usize;
                let tiles = the_memory.sector_mem.add(sec * SECTOR_BYTES);

                for line in 0..SECTOR_TILE_ROWS {
                    let mut words = [0u32; PLANE_COUNT];

                    for px in 0..SECTOR_PIXELS {
                        let tile = *tiles.add(line * TILE_ROW_PIXELS + px) as usize;
                        let color = *the_memory
                            .terrain_mem
                            .add(tile * TERRAIN_ENTRY_BYTES + TERRAIN_COLOR_BYTE);

                        for (p, word) in words.iter_mut().enumerate() {
                            // Builds one bit per pixel for bitplane `p`: shift the row left and add the plane's bit of this colour.
                            *word = (*word << 1) | ((color >> p) & 1) as u32;
                        }
                    }

                    for (p, word) in words.iter().enumerate() {
                        let d =
                            (*the_screen.planes.add(p)).add(offset + line * OVERVIEW_LINE_BYTES);
                        // A 16 pixel row is two bytes, high byte first.
                        *d = (word >> BYTE_SHIFT) as u8;
                        // The second copy is for the other page.
                        *d.add(OVERVIEW_LINE_COPY) = (word >> BYTE_SHIFT) as u8;
                        *d.add(1) = *word as u8;
                        *d.add(OVERVIEW_LINE_COPY + 1) = *word as u8;
                    }
                }

                offset += 2;
            }

            offset += OVERVIEW_ROW_SKIP;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overview_origin_is_clamped_to_the_map() {
        assert_eq!(overview_origin(0, 0, 0, 0), (0, 0));
        assert_eq!(overview_origin(0x3000, 0x1000, 0, 0), (39, 12));
        assert_eq!(overview_origin(0x5000, 0x2800, 0, 0), (46, 23));

        // positions that wrap around the 128 sector map
        assert_eq!(overview_origin(0x7f00, 0x7f00, 0, 0), (0, 0));
    }
}
