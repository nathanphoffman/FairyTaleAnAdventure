//! Drawing the map into the play field: single tiles, whole screens, new edges and scrolling.

use crate::game::data::object_tables::minimap;
use crate::game::data::state::Game;

/// One bitplane of 256 tiles, 64 bytes per tile.
pub(super) const IMAGE_PLANE_SIZE: usize = 16384;
pub(super) const TILE_BYTES: usize = 64;
pub(super) const TILE_LINES: usize = 32;

/// Bytes per scan line of the play field bitmaps.
pub(super) const ROW_BYTES: usize = 40;
pub(super) const PLANE_COUNT: usize = 5;

/// The play field is this many tiles wide and tall.
pub(super) const SCREEN_COLUMNS: usize = 19;
pub(super) const SCREEN_ROWS: usize = 6;

/// Draws one tile at byte offset `dst` in each of the five planes (2 bytes wide, 32 lines).
pub(super) fn draw_tile(game: &mut Game, tile: i32, dst: usize) {
    for p in 0..PLANE_COUNT {
        let mut src = p * IMAGE_PLANE_SIZE + tile as usize * TILE_BYTES;
        let image = &game.memory.image;
        let plane = game.screen.draw_bitmap_mut().plane_mut(p);
        let mut d = dst;

        for _ in 0..TILE_LINES {
            plane[d] = image[src];
            plane[d + 1] = image[src + 1];
            src += 2;
            d += ROW_BYTES;
        }
    }
}

/// Draws the tile at minimap position (col, row) to its place on screen.
pub(super) unsafe fn draw_minimap_tile(game: &mut Game, col: usize, row: usize) {
    unsafe {
        draw_tile(
            game,
            minimap[col * SCREEN_ROWS + row] as i32,
            row * TILE_LINES * ROW_BYTES + col * 2,
        )
    };
}

/// The whole screen, 19 x 6 tiles.
pub fn draw_map(game: &mut Game) {
    for col in 0..SCREEN_COLUMNS {
        for row in 0..SCREEN_ROWS {
            unsafe { draw_minimap_tile(game, col, row) };
        }
    }
}

/// One column of tiles; `x` is the byte offset on screen (0 or 36).
pub fn draw_map_column(game: &mut Game, x: i32) {
    for row in 0..SCREEN_ROWS {
        unsafe { draw_minimap_tile(game, x as usize / 2, row) };
    }
}

/// One row of tiles; `r` is twice the row number (0 or 10).
pub fn draw_map_row(game: &mut Game, r: i32) {
    for col in 0..SCREEN_COLUMNS {
        unsafe { draw_minimap_tile(game, col, r as usize / 2) };
    }
}

/// One way to scroll: source x (bytes), source line, destination x, destination line, width in
/// bytes and number of lines.
pub(super) struct ScrollMode {
    pub(super) src_x: usize,
    pub(super) src_line: usize,
    pub(super) dst_x: usize,
    pub(super) dst_line: usize,
    pub(super) width: usize,
    pub(super) lines: usize,
}

pub(super) const fn mode(
    src_x: usize,
    src_line: usize,
    dst_x: usize,
    dst_line: usize,
    width: usize,
    lines: usize,
) -> ScrollMode {
    ScrollMode {
        src_x,
        src_line,
        dst_x,
        dst_line,
        width,
        lines,
    }
}

/// Directions are numbered by how the picture moves:
/// 0 right, 1 down+right, 2 down, 3 down+left, 4 left, 5 up+left, 6 up, 7 up+right.
/// The new edge is filled in by `draw_map_column` / `draw_map_row` afterwards.
pub(super) const SCROLL_MODES: [ScrollMode; 8] = [
    mode(0, 0, 2, 0, 36, 192),
    mode(0, 0, 2, 32, 36, 160),
    mode(0, 0, 0, 32, 38, 160),
    mode(2, 0, 0, 32, 36, 160),
    mode(2, 0, 0, 0, 36, 192),
    mode(2, 32, 0, 0, 36, 160),
    mode(0, 32, 0, 0, 38, 160),
    mode(0, 32, 2, 0, 36, 160),
];

/// There are 8 scroll directions.
const DIRECTION_MASK: usize = 7;

/// Shifts the map one tile in one of 8 directions (the blitter did this on the Amiga).
/// Moving down copies from the bottom up, so lines are not overwritten before they are read.
pub fn scroll_map(game: &mut Game, dir: i32) {
    // Masks with SCROLL_MODES, keeping only the bits that matter here.
    let m = &SCROLL_MODES[dir as usize & DIRECTION_MASK];
    let bitmap = game.screen.draw_bitmap_mut();

    for p in 0..PLANE_COUNT {
        let plane = bitmap.plane_mut(p);

        for i in 0..m.lines {
            // copy bottom-up when moving down so rows are not overwritten before being read
            let line = if m.dst_line > m.src_line {
                m.lines - 1 - i
            } else {
                i
            };

            let from = (m.src_line + line) * ROW_BYTES + m.src_x;
            let to = (m.dst_line + line) * ROW_BYTES + m.dst_x;
            plane.copy_within(from..from + m.width, to);
        }
    }
}
