//! The map routines that were 68000 assembly in fsubs.asm (from csrc/asm_map.c). Behaviour,
//! including 16 bit wrap-around, follows the assembly. See docs/data-formats.md "Map data layout".
//!
//! World coordinates are 15 bit. A tile ("image") is 16 pixels wide and 32 tall; a sector is
//! 16 x 8 tiles; the region map is 128 x 32 sectors, one byte each.

unsafe extern "C" {
    static xreg: u16;
    static yreg: u16;
    static mut map_x: u16;
    static mut map_y: u16;
    static hero_x: u16;
    static hero_y: u16;
    static region_num: u16;
    static mut hero_sector: i16;
    static map_mem: *const u8;
    static sector_mem: *const u8;
    static terra_mem: *const u8;
    static image_mem: *const u8;
    static mut minimap: [i16; 120];
    static planes: *mut *mut u8;
    static mut secx: i32;
    static mut secy: i32;
}

/// One bitplane of 256 tiles, 64 bytes per tile.
const IMAGE_PLANE_SIZE: usize = 16384;
const TILE_BYTES: usize = 64;
const TILE_LINES: usize = 32;
/// Bytes per scan line of the play field bitmaps.
const ROW_BYTES: usize = 40;
const PLANE_COUNT: usize = 5;
/// The play field is this many tiles wide and tall.
const SCREEN_COLUMNS: usize = 19;
const SCREEN_ROWS: usize = 6;
/// Sectors are 16 x 8 tiles of 128 bytes; the region map has 128 sector bytes per row.
const SECTOR_BYTES: usize = 128;
const MAP_ROW_BYTES: usize = 128;
const SECTOR_ROWS_IN_REGION: i16 = 32;

/// World coordinates are 15 bit.
const COORD_MASK: i32 = 0x7fff;
const COORD_SIGN: i32 = 0x8000;

/// Sector column from an image column: values outside 0-63 wrap, as in the assembly.
fn wrap_secx(mut v: u16) -> u16 {
    if v & 64 != 0 {
        v = if v & 32 != 0 { 0 } else { 63 };
    }
    v
}

/// Sector row from an image row; rows outside 0-31 are clamped.
fn clamp_secy(v: i16) -> u16 {
    v.clamp(0, SECTOR_ROWS_IN_REGION - 1) as u16
}

/// The sector byte for a sector column and row.
unsafe fn sector_at(col: u16, row: u16) -> u8 {
    unsafe { *map_mem.add((row as usize * MAP_ROW_BYTES + col as usize + xreg as usize) as u16 as usize) }
}

/// Terrain type (0-15) at a pixel position, 0 if that sub-area is clear.
#[unsafe(no_mangle)]
pub extern "C" fn px_to_im(x: i32, y: i32) -> i32 {
    let (ux, uy) = (x as u16, y as u16);
    let mut bit = 0x80u8;
    if ux & 8 != 0 {
        bit >>= 4;
    }
    if uy & 8 != 0 {
        bit >>= 1;
    }
    if uy & 16 != 0 {
        bit >>= 2;
    }
    let (imx, imy) = (ux >> 4, uy >> 5);
    unsafe {
        let col = wrap_secx((imx >> 4).wrapping_sub(xreg));
        let row = clamp_secy((imy >> 3).wrapping_sub(yreg) as i16);
        let sec = sector_at(col, row) as usize;
        let tile = *sector_mem.add((sec * SECTOR_BYTES + (imy as usize & 7) * 16 + (imx as usize & 15)) as u16 as usize) as usize;
        if *terra_mem.add(tile * 4 + 2) & bit == 0 {
            return 0;
        }
        (*terra_mem.add(tile * 4 + 1) >> 4) as i32
    }
}

/// Collision test with two probes either side of the character. Returns the blocking terrain
/// type, 0 if the way is clear.
#[unsafe(no_mangle)]
pub extern "C" fn prox(x: i32, y: i32) -> i32 {
    let t = px_to_im(x + 4, y + 2);
    if t == 1 || t >= 10 {
        return t;
    }
    let t = px_to_im(x - 4, y + 2);
    if t == 1 || t >= 8 {
        return t;
    }
    0
}

/// Address of the tile byte for a tile position (wraps vertically too).
#[unsafe(no_mangle)]
pub extern "C" fn mapxy(imx: i32, imy: i32) -> *const u8 {
    let (ux, uy) = (imx as u16, imy as u16);
    unsafe {
        let col = wrap_secx((ux >> 4).wrapping_sub(xreg));
        let mut row = (uy >> 3).wrapping_sub(yreg);
        if row & 32 != 0 {
            row = if row & 16 != 0 { 0 } else { 31 };
        }
        let sec = sector_at(col, row) as usize;
        sector_mem.add((sec * SECTOR_BYTES + (uy as usize & 7) * 16 + (ux as usize & 15)) as u16 as usize)
    }
}

/// Fills `minimap` (19 columns of 6 tile numbers, column by column) for the area starting at
/// tile (img_x, img_y), and works out which sector the hero is in.
#[unsafe(no_mangle)]
pub extern "C" fn genmini(img_x: i32, img_y: i32) {
    unsafe {
        let mut x = img_x as u16;
        let mut next = 0;
        for _ in 0..SCREEN_COLUMNS {
            x &= 0x07ff;
            let col = wrap_secx((x >> 4).wrapping_sub(xreg)).wrapping_add(xreg);
            for j in 0..SCREEN_ROWS as i32 {
                let y = (img_y + j) as u16 & 0x7fff;
                let row = clamp_secy((y >> 3).wrapping_sub(yreg) as i16);
                let sec = *map_mem.add(((row as usize) << 7).wrapping_add(col as usize) as u16 as usize) as usize;
                let tile = *sector_mem.add((sec * SECTOR_BYTES + (y as usize & 7) * 16 + (x as usize & 15)) as u16 as usize);
                minimap[next] = tile as i16;
                next += 1;
            }
            x += 1;
        }
        // the assembly read the high byte of hero_x and hero_y (68000 is big-endian)
        let hx = (hero_x >> 8) as i32;
        let hy = ((hero_y >> 8).wrapping_sub(yreg)) as i16 as i32;
        let mut sec = *map_mem.add(((hy << 7).wrapping_add(hx)) as u16 as usize) as i32;
        if region_num & 0xff > 7 {
            sec += 256;
        }
        hero_sector = sec as i16;
    }
}

// Steps for the 8 directions (and two "no move" entries) per half-unit of speed.
const X_STEPS: [i16; 10] = [-2, 0, 2, 3, 2, 0, -2, -3, 0, 0];
const Y_STEPS: [i16; 10] = [-2, -3, -2, 0, 2, 3, 2, 0, 0, 0];
/// Directions above this mean "no move".
const MAX_DIRECTION: i32 = 7;

/// Moves `v` one step; the result stays within 15 bits.
fn step(v: i32, dir: i32, speed: i32, table: &[i16; 10]) -> i32 {
    let product = table[(dir & 0xff) as usize] as i32 * speed as i16 as i32;
    let half = (product as u16 >> 1) as i32;
    (v.wrapping_add(half) as u16 as i32) & COORD_MASK
}

#[unsafe(no_mangle)]
pub extern "C" fn newx(x: i32, dir: i32, speed: i32) -> i32 {
    if dir & 0xff > MAX_DIRECTION {
        return x;
    }
    step(x, dir, speed, &X_STEPS)
}

/// Like `newx`, but keeps bit 15 of y.
#[unsafe(no_mangle)]
pub extern "C" fn newy(y: i32, dir: i32, speed: i32) -> i32 {
    if dir & 0xff > MAX_DIRECTION {
        return y;
    }
    step(y, dir, speed, &Y_STEPS) | (y & COORD_SIGN)
}

/// Copies bit 14 into bit 15, giving a signed 15 bit value.
#[unsafe(no_mangle)]
pub extern "C" fn wrap(x: i32) -> i32 {
    let mut low = x as u32 & 0xffff;
    if low & 0x4000 != 0 {
        low |= 0x8000;
    } else {
        low &= 0x7fff;
    }
    ((x as u32 & 0xffff_0000) | low) as i32
}

// The hero may wander this far from the camera centre before it moves.
const CAMERA_CENTER_X: i32 = 144;
const CAMERA_CENTER_Y: i32 = 70;
const CAMERA_JUMP_X: i16 = 70;
const CAMERA_CREEP_X: i16 = 20;
const CAMERA_JUMP_UP: i16 = 24;
const CAMERA_JUMP_DOWN: i16 = 44;
const CAMERA_CREEP_Y: i16 = 10;

/// Sign-extends a 15 bit difference.
fn sign_extend_15(v: i16) -> i16 {
    ((v as i32) << 1) as i16 >> 1
}

/// Where the camera goes for a hero position: it creeps along once the hero is 20 pixels (10
/// vertically) off, and jumps once the hero is 70 (24 up, 44 down) off.
fn camera_after(cam_x: u16, cam_y: u16, x: i32, y: i32) -> (u16, u16) {
    let cx = (x - CAMERA_CENTER_X) as i16;
    let cy = (y - CAMERA_CENTER_Y) as i16;
    let dx = sign_extend_15((cx as i32 - cam_x as i32) as i16);
    let dy = sign_extend_15((cy as i32 - cam_y as i32) as i16);

    let new_x = if dx < -CAMERA_JUMP_X {
        (cx as i32 + CAMERA_JUMP_X as i32) as u16
    } else if dx > CAMERA_JUMP_X {
        (cx as i32 - CAMERA_JUMP_X as i32) as u16
    } else if dx < -CAMERA_CREEP_X {
        cam_x.wrapping_sub(1)
    } else if dx > CAMERA_CREEP_X {
        cam_x.wrapping_add(1)
    } else {
        cam_x
    };
    let new_y = if dy < -CAMERA_JUMP_UP {
        (cy as i32 + CAMERA_JUMP_UP as i32) as u16
    } else if dy > CAMERA_JUMP_DOWN {
        (cy as i32 - CAMERA_JUMP_DOWN as i32) as u16
    } else if dy < -CAMERA_CREEP_Y {
        cam_y.wrapping_sub(1)
    } else if dy > CAMERA_CREEP_Y {
        cam_y.wrapping_add(1)
    } else {
        cam_y
    };
    (new_x, new_y)
}

/// Moves the camera (`map_x`, `map_y`) after the hero.
#[unsafe(no_mangle)]
pub extern "C" fn map_adjust(x: i32, y: i32) {
    unsafe { (map_x, map_y) = camera_after(map_x, map_y, x, y) };
}

/// Small lookup used by the page-turn animation.
#[unsafe(no_mangle)]
pub extern "C" fn page_det(x: i32) -> i32 {
    const SMALL: [i32; 11] = [9, 9, 8, 7, 6, 5, 5, 5, 4, 4, 4];
    match x {
        ..=10 => SMALL[x as usize],
        137.. => 10,
        136 => 7,
        124..=135 => 6,
        99..=123 => 5,
        72..=98 => 4,
        _ => 3,
    }
}

// ---- drawing the map into the play field bitmaps (planes[0..4]) ----

/// Draws one tile at byte offset `dst` in each of the five planes.
unsafe fn draw_tile(tile: i32, dst: usize) {
    unsafe {
        for p in 0..PLANE_COUNT {
            let mut src = image_mem.add(p * IMAGE_PLANE_SIZE).offset(tile as isize * TILE_BYTES as isize);
            let mut d = (*planes.add(p)).add(dst);
            for _ in 0..TILE_LINES {
                *d = *src;
                *d.add(1) = *src.add(1);
                src = src.add(2);
                d = d.add(ROW_BYTES);
            }
        }
    }
}

/// Draws the tile at minimap position (col, row) to its place on screen.
unsafe fn draw_minimap_tile(col: usize, row: usize) {
    unsafe { draw_tile(minimap[col * SCREEN_ROWS + row] as i32, row * TILE_LINES * ROW_BYTES + col * 2) };
}

/// The whole screen, 19 x 6 tiles.
#[unsafe(no_mangle)]
pub extern "C" fn map_draw() {
    for col in 0..SCREEN_COLUMNS {
        for row in 0..SCREEN_ROWS {
            unsafe { draw_minimap_tile(col, row) };
        }
    }
}

/// One column of tiles; `x` is the byte offset on screen (0 or 36).
#[unsafe(no_mangle)]
pub extern "C" fn strip_draw(x: i32) {
    for row in 0..SCREEN_ROWS {
        unsafe { draw_minimap_tile(x as usize / 2, row) };
    }
}

/// One row of tiles; `r` is twice the row number (0 or 10).
#[unsafe(no_mangle)]
pub extern "C" fn row_draw(r: i32) {
    for col in 0..SCREEN_COLUMNS {
        unsafe { draw_minimap_tile(col, r as usize / 2) };
    }
}

/// One way to scroll: source x (bytes), source line, destination x, destination line, width in
/// bytes and number of lines.
struct ScrollMode {
    src_x: usize,
    src_line: usize,
    dst_x: usize,
    dst_line: usize,
    width: usize,
    lines: usize,
}

const fn mode(src_x: usize, src_line: usize, dst_x: usize, dst_line: usize, width: usize, lines: usize) -> ScrollMode {
    ScrollMode { src_x, src_line, dst_x, dst_line, width, lines }
}

/// Directions are numbered by how the picture moves:
/// 0 right, 1 down+right, 2 down, 3 down+left, 4 left, 5 up+left, 6 up, 7 up+right.
/// The new edge is filled in by `strip_draw` / `row_draw` afterwards.
const SCROLL_MODES: [ScrollMode; 8] = [
    mode(0, 0, 2, 0, 36, 192),
    mode(0, 0, 2, 32, 36, 160),
    mode(0, 0, 0, 32, 38, 160),
    mode(2, 0, 0, 32, 36, 160),
    mode(2, 0, 0, 0, 36, 192),
    mode(2, 32, 0, 0, 36, 160),
    mode(0, 32, 0, 0, 38, 160),
    mode(0, 32, 2, 0, 36, 160),
];

/// Shifts the map one tile in one of 8 directions (the blitter did this on the Amiga).
#[unsafe(no_mangle)]
pub extern "C" fn scrollmap(dir: i32) {
    let m = &SCROLL_MODES[dir as usize & 7];
    unsafe {
        for p in 0..PLANE_COUNT {
            let plane = *planes.add(p);
            for i in 0..m.lines {
                // copy bottom-up when moving down so rows are not overwritten before being read
                let line = if m.dst_line > m.src_line { m.lines - 1 - i } else { i };
                std::ptr::copy(
                    plane.add((m.src_line + line) * ROW_BYTES + m.src_x),
                    plane.add((m.dst_line + line) * ROW_BYTES + m.dst_x),
                    m.width,
                );
            }
        }
    }
}

// The overview map (magic spell): each sector is drawn as 16 x 16 pixels, 18 x 9 sectors.
const OVERVIEW_COLUMNS: i16 = 18;
const OVERVIEW_ROWS: i16 = 9;
/// Sectors across and down the whole map.
const MAP_SECTORS_WIDE: i16 = 64;
const MAP_WRAP: i16 = 128;
const SECTOR_PIXELS: usize = 16;
const SECTOR_TILE_ROWS: usize = 8;
/// Bytes between the two copies of each overview line (it is drawn twice as tall).
const OVERVIEW_LINE_COPY: usize = 40;
const OVERVIEW_LINE_BYTES: usize = 80;
/// Bytes to skip at the end of each overview row of sectors.
const OVERVIEW_ROW_SKIP: usize = 40 * 15 + 4;

/// The overview's top left sector for a map position, clamped to the map.
fn overview_origin(mx: i32, my: i32, region_x: u16, region_y: u16) -> (i16, i16) {
    let mut x = (((mx as u16 >> 8) as i32 - 9 - region_x as i32) as i16, ((my as u16 >> 8) as i32 - 4 - region_y as i32) as i16);
    for v in [&mut x.0, &mut x.1] {
        if *v < 32 {
            *v += MAP_WRAP;
        }
        if *v > 96 {
            *v -= MAP_WRAP;
        }
    }
    (
        x.0.clamp(0, MAP_SECTORS_WIDE - OVERVIEW_COLUMNS),
        x.1.clamp(0, SECTOR_ROWS_IN_REGION - OVERVIEW_ROWS),
    )
}

/// Draws the overview map. Each pixel's colour is the tile's overview colour (terra byte 3).
/// `secx` / `secy` tell the caller which sector is at the top left.
#[unsafe(no_mangle)]
pub extern "C" fn bigdraw(mx: i32, my: i32) {
    unsafe {
        let (x, y) = overview_origin(mx, my, xreg, yreg);
        secx = x as i32;
        secy = y as i32;
        let mut offset = 0usize;
        for j in 0..OVERVIEW_ROWS {
            for i in 0..OVERVIEW_COLUMNS {
                let index = xreg as i32 + x as i32 + i as i32 + (((y + j) as i32) << 7);
                let sec = *map_mem.add(index as u16 as usize) as usize;
                let tiles = sector_mem.add(sec * SECTOR_BYTES);
                for line in 0..SECTOR_TILE_ROWS {
                    let mut words = [0u32; PLANE_COUNT];
                    for px in 0..SECTOR_PIXELS {
                        let tile = *tiles.add(line * 16 + px) as usize;
                        let color = *terra_mem.add(tile * 4 + 3);
                        for (p, word) in words.iter_mut().enumerate() {
                            *word = (*word << 1) | ((color >> p) & 1) as u32;
                        }
                    }
                    for (p, word) in words.iter().enumerate() {
                        let d = (*planes.add(p)).add(offset + line * OVERVIEW_LINE_BYTES);
                        *d = (word >> 8) as u8;
                        *d.add(OVERVIEW_LINE_COPY) = (word >> 8) as u8;
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

    #[test]
    fn steps_follow_the_direction_tables() {
        assert_eq!(newx(100, 2, 4), 104); // east, speed 4: +2 * 4 / 2
        assert_eq!(newx(100, 6, 4), 96);
        assert_eq!(newx(100, 9, 4), 100); // no move
        assert_eq!(newy(100, 1, 2), 97); // north: the negative step wraps in 16 bits, then stays 15 bit
    }

    #[test]
    fn newy_keeps_bit_15() {
        assert_eq!(newy(0x8000 | 100, 4, 2) & 0x8000, 0x8000);
    }

    #[test]
    fn wrap_copies_bit_14_to_bit_15() {
        assert_eq!(wrap(0x4000), 0xc000);
        assert_eq!(wrap(0x1234), 0x1234);
        assert_eq!(wrap(0x9234), 0x1234);
    }

    #[test]
    fn page_det_covers_every_range() {
        assert_eq!(page_det(0), 9);
        assert_eq!(page_det(10), 4);
        assert_eq!(page_det(11), 3);
        assert_eq!(page_det(72), 4);
        assert_eq!(page_det(99), 5);
        assert_eq!(page_det(124), 6);
        assert_eq!(page_det(136), 7);
        assert_eq!(page_det(137), 10);
    }

    #[test]
    fn camera_stays_while_the_hero_is_near() {
        assert_eq!(camera_after(100, 100, 244 + 10, 170 + 5), (100, 100));
    }

    #[test]
    fn camera_creeps_then_jumps() {
        assert_eq!(camera_after(100, 100, 144 + 100 + 30, 170), (101, 100));
        assert_eq!(camera_after(100, 100, 144 + 100 + 80, 170), (110, 100));
        assert_eq!(camera_after(100, 100, 244, 70 + 100 + 50), (100, 106));
    }

    #[test]
    fn overview_origin_is_clamped_to_the_map() {
        assert_eq!(overview_origin(0, 0, 0, 0), (0, 0));
        assert_eq!(overview_origin(0x3000, 0x1000, 0, 0), (39, 12));
        assert_eq!(overview_origin(0x5000, 0x2800, 0, 0), (46, 23));
        // positions that wrap around the 128 sector map
        assert_eq!(overview_origin(0x7f00, 0x7f00, 0, 0), (0, 0));
    }
}
