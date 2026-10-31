//! The picture behind the launch menu: a long strip of the real game map (grass, hills and trees
//! near the start) that is built once and then panned slowly from side to side.

use crate::game::data::state::{Game, Graphics, Video};
use crate::game::output::Output;
use crate::game::render::map_tiles::{IMAGE_PLANE_SIZE, PLANE_COUNT, TILE_BYTES, TILE_LINES};
use crate::game::world::map::{SCREEN_COLUMNS, SCREEN_ROWS, fill_minimap};
use crate::game::world::regions::{load_region_now, refresh_minimap};
use crate::gfx::bitmap::{Bitmap, copy_window};

/// Where the strip starts: the top left corner of the map, in pixels. The camera pans east along
/// it, over meadows with trees and a rocky hill, and back again. It is in the region the first
/// brother starts in, so the region is already loaded when the game begins.
const START: (u16, u16) = (18200, 12400);
const REGION: u16 = 3;

/// A tile is 16 pixels wide and `TILE_LINES` (32) high, and 2 bytes wide in a plane.
const TILE_WIDTH: usize = 16;
const TILE_WIDTH_BYTES: usize = 2;
const TILE_WIDTH_SHIFT: u32 = 4;
const TILE_HEIGHT_SHIFT: u32 = 5;
const TILE_HEIGHT_MASK: u16 = 31;

/// The strip is this many screens of tiles wide and this many tile rows high (a screen is 200
/// lines, so 7 rows would leave no room for the start offset).
const STRIP_SCREENS: usize = 20;
const STRIP_ROWS: usize = 8;

/// The part of the strip that is shown is the size of the game screen.
const VIEW_WIDTH: usize = 320;

/// The camera moves one pixel every this many ticks (of 1/50 second).
const TICKS_PER_PIXEL: u32 = 2;

/// The strip of map, with the lines above the part that is shown.
pub(super) struct Background {
    strip: Bitmap,
    top_line: usize,
}

impl Background {
    /// Loads the region and builds the strip. The page colours become the colours of the map.
    pub(super) fn load(game: &mut Game, video: &mut Video) -> Background {
        game.world.map_x = START.0;
        game.world.map_y = START.1;
        game.world.tile_x = START.0 >> TILE_WIDTH_SHIFT;
        game.world.tile_y = START.1 >> TILE_HEIGHT_SHIFT;
        game.world.new_region = REGION;
        load_region_now(game, video);
        refresh_minimap(game, video);

        let strip = build_strip(game, &video.graphics);
        video.load_page_colors(&game.page_colors);

        Background {
            strip,
            top_line: (START.1 & TILE_HEIGHT_MASK) as usize,
        }
    }

    /// Draws the part of the strip the camera sees after `tick` ticks onto the page being drawn.
    pub(super) fn draw(&self, video: &mut Video, tick: u32) {
        // The bitmap that is on screen: after the intro it need not be the one drawn on.
        let screen = &mut video.screen;
        let page = screen
            .vp_page
            .ras
            .map_or(screen.drawing_bitmap_id(), |ras| screen.ras[ras].bitmap);
        let left = pan_position(tick, self.strip.width());

        copy_window(&self.strip, screen.bitmap_mut(page), (left, self.top_line));
    }
}

/// Where the left edge of the view is after `tick` ticks: it goes right to the end of the strip
/// and back again, over and over.
fn pan_position(tick: u32, strip_width: usize) -> usize {
    let travel = strip_width.saturating_sub(VIEW_WIDTH);

    if travel == 0 {
        return 0;
    }

    let along = (tick / TICKS_PER_PIXEL) as usize % (travel * 2);

    if along <= travel {
        along
    } else {
        travel * 2 - along
    }
}

/// Draws the tiles of the map from `START` onwards into a bitmap, one screen of columns at a time
/// (the game's own map lookup gives 19 columns of 6 tiles).
fn build_strip(game: &mut Game, graphics: &Graphics) -> Bitmap {
    let width = STRIP_SCREENS * SCREEN_COLUMNS * TILE_WIDTH;
    let mut strip = Bitmap::new(width, STRIP_ROWS * TILE_LINES, PLANE_COUNT);
    let first_tile = (
        (START.0 >> TILE_WIDTH_SHIFT) as i32,
        (START.1 >> TILE_HEIGHT_SHIFT) as i32,
    );

    for screen in 0..STRIP_SCREENS {
        for band in 0..STRIP_ROWS.div_ceil(SCREEN_ROWS) {
            let column = first_tile.0 + (screen * SCREEN_COLUMNS) as i32;

            fill_minimap(game, column, first_tile.1 + (band * SCREEN_ROWS) as i32);

            for col in 0..SCREEN_COLUMNS {
                for row in 0..SCREEN_ROWS.min(STRIP_ROWS - band * SCREEN_ROWS) {
                    let tile = game.map.minimap[col * SCREEN_ROWS + row];

                    draw_tile(
                        graphics,
                        &mut strip,
                        tile as usize,
                        (screen * SCREEN_COLUMNS + col, band * SCREEN_ROWS + row),
                    );
                }
            }
        }
    }

    strip
}

/// Copies one tile image into the strip at tile column and row `at`.
fn draw_tile(graphics: &Graphics, strip: &mut Bitmap, tile: usize, at: (usize, usize)) {
    let row_bytes = strip.bytes_per_row();

    for plane_number in 0..PLANE_COUNT {
        let image = &graphics.image[plane_number * IMAGE_PLANE_SIZE + tile * TILE_BYTES..];
        let plane = strip.plane_mut(plane_number);
        let start = at.1 * TILE_LINES * row_bytes + at.0 * TILE_WIDTH_BYTES;

        for line in 0..TILE_LINES {
            let to = start + line * row_bytes;
            let from = line * TILE_WIDTH_BYTES;

            plane[to..to + TILE_WIDTH_BYTES].copy_from_slice(&image[from..from + TILE_WIDTH_BYTES]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_camera_goes_to_the_end_and_comes_back() {
        let width = VIEW_WIDTH + 100;

        assert_eq!(pan_position(0, width), 0);
        assert_eq!(pan_position(TICKS_PER_PIXEL * 40, width), 40);
        assert_eq!(pan_position(TICKS_PER_PIXEL * 100, width), 100);
        assert_eq!(pan_position(TICKS_PER_PIXEL * 160, width), 40);
        assert_eq!(pan_position(TICKS_PER_PIXEL * 200, width), 0);
    }

    #[test]
    fn a_strip_no_wider_than_the_view_does_not_pan() {
        assert_eq!(pan_position(12345, VIEW_WIDTH), 0);
        assert_eq!(pan_position(12345, 10), 0);
    }
}
