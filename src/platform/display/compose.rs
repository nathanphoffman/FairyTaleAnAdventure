//! Turns the planar bitmaps of a view into RGB pixels.

use super::{HEIGHT, SPRITE, WIDTH};
use crate::amiga::*;

const VP_HIDE: u16 = 0x2000;
const SPRITES: u16 = 0x4000;

const BITS_PER_BYTE: i32 = 8;
const FIRST_BIT: u8 = 0x80;
const NIBBLE_MASK: u16 = 15;
const NIBBLE_TO_BYTE: u32 = 17; // 0xF * 17 = 0xFF
const GREEN_NIBBLE_SHIFT: u16 = 4;
const RED_NIBBLE_SHIFT: u16 = 8;
const RED_BYTE_SHIFT: u32 = 16;
const GREEN_BYTE_SHIFT: u32 = 8;

/// The pointer's colours are 16 + the sprite pixel value in the colour table.
const POINTER_FIRST_COLOR: usize = 16;
const POINTER_WIDTH: i32 = 16;
const POINTER_MAX_LINES: usize = 32;
const HIGH_WORD_SHIFT: u32 = 16;
/// Lores pixels are doubled horizontally.
const LORES_SCALE: i32 = 2;

/// 12 bit Amiga colour (0x0RGB) to 0xRRGGBB.
fn rgb4(c: u16) -> u32 {
    let r = (c >> RED_NIBBLE_SHIFT & NIBBLE_MASK) as u32;
    let g = (c >> GREEN_NIBBLE_SHIFT & NIBBLE_MASK) as u32;
    let b = (c & NIBBLE_MASK) as u32;
    (r * NIBBLE_TO_BYTE) << RED_BYTE_SHIFT
        | (g * NIBBLE_TO_BYTE) << GREEN_BYTE_SHIFT
        | (b * NIBBLE_TO_BYTE)
}

/// Sets a frame pixel if it lies inside the frame.
fn put_pixel(frame: &mut [u32], x: i32, y: i32, color: u32) {
    if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
        frame[y as usize * WIDTH as usize + x as usize] = color;
    }
}

/// Draws one line of the pointer sprite: two 16 bit planes packed in one 32 bit word.
fn draw_pointer_line(frame: &mut [u32], colors: &ColorMap, word: u32, origin: (i32, i32)) {
    let (a, b) = ((word >> HIGH_WORD_SHIFT) as u16, word as u16);
    for bit in 0..POINTER_WIDTH {
        let shift = POINTER_WIDTH - 1 - bit;
        let v = (a >> shift & 1) | (b >> shift & 1) << 1;
        let idx = POINTER_FIRST_COLOR + v as usize;
        if v == 0 || idx >= colors.Count as usize {
            continue;
        }
        let color = rgb4(unsafe { *colors.ColorTable.add(idx) });
        for dx in 0..LORES_SCALE {
            put_pixel(frame, origin.0 + bit * LORES_SCALE + dx, origin.1, color);
        }
    }
}

/// Draws the mouse pointer: a 16 pixel wide sprite, 2 frame pixels per sprite pixel. Each line is
/// two 16 bit planes (the high word of each 32 bit value is plane 0); colour = 16 + value.
fn draw_pointer(frame: &mut [u32], colors: &ColorMap) {
    let data = SPRITE.with(|c| *c.borrow());
    if data.is_null() {
        return;
    }
    let (px, py) = crate::platform::input::pointer();
    for line in 0..POINTER_MAX_LINES {
        let word = unsafe { *data.add(1 + line) };
        if word == 0 {
            break;
        }
        draw_pointer_line(frame, colors, word, (px * LORES_SCALE, py + line as i32));
    }
}

/// The colour-table index of the pixel at (x, y), built from one bit per bitplane.
fn pixel_index(bm: &BitMap, x: i32, y: i32) -> usize {
    let mut index = 0usize;
    for p in 0..bm.Depth as usize {
        let plane = bm.Planes[p];
        if plane.is_null() {
            continue;
        }
        let offset = y as usize * bm.BytesPerRow as usize + (x / BITS_PER_BYTE) as usize;
        let byte = unsafe { *plane.add(offset) };
        if byte & (FIRST_BIT >> (x % BITS_PER_BYTE)) != 0 {
            index |= 1 << p;
        }
    }
    index
}

/// Draws one viewport's rows into the frame.
fn draw_viewport(v: &ViewPort, ri: &RasInfo, frame: &mut [u32]) {
    let bm = unsafe { &*ri.BitMap };
    let cm = unsafe { &*v.ColorMap };
    let hires = v.Modes & HIRES != 0;
    for row in 0..v.DHeight as i32 {
        let sy = ri.RyOffset as i32 + row;
        let oy = v.DyOffset as i32 + row;
        if sy < 0 || sy >= bm.Rows as i32 || oy < 0 || oy >= HEIGHT as i32 {
            continue;
        }
        for col in 0..v.DWidth as i32 {
            let sx = ri.RxOffset as i32 + col;
            if sx < 0 || sx >= bm.BytesPerRow as i32 * BITS_PER_BYTE {
                continue;
            }
            let index = pixel_index(bm, sx, sy);
            let color = if index < cm.Count as usize {
                rgb4(unsafe { *cm.ColorTable.add(index) })
            } else {
                0
            };
            let screen_x = v.DxOffset as i32 + col;
            if hires {
                put_pixel(frame, screen_x, oy, color);
            } else {
                put_pixel(frame, screen_x * LORES_SCALE, oy, color);
                put_pixel(frame, screen_x * LORES_SCALE + 1, oy, color);
            }
        }
    }
}

/// Draws all viewports of the view into the 640x200 frame.
pub(super) fn compose(view: *const View, frame: &mut [u32]) {
    frame.fill(0);
    if view.is_null() {
        return;
    }
    let mut pointer_colors: *const ColorMap = std::ptr::null();
    let mut vp = unsafe { (*view).ViewPort };
    while !vp.is_null() {
        let v = unsafe { &*vp };
        vp = v.Next;
        if v.RasInfo.is_null() || v.ColorMap.is_null() || v.Modes & VP_HIDE != 0 {
            continue;
        }
        if v.Modes & SPRITES != 0 {
            pointer_colors = v.ColorMap;
        }
        let ri = unsafe { &*v.RasInfo };
        if !ri.BitMap.is_null() {
            draw_viewport(v, ri, frame);
        }
    }
    if !pointer_colors.is_null() {
        draw_pointer(frame, unsafe { &*pointer_colors });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_twelve_bit_colour_to_rgb() {
        assert_eq!(rgb4(0x0000), 0x000000);
        assert_eq!(rgb4(0x0FFF), 0xFFFFFF);
        assert_eq!(rgb4(0x0F08), 0xFF0088);
    }

    #[test]
    fn put_pixel_ignores_points_outside_the_frame() {
        let mut frame = vec![0u32; (WIDTH * HEIGHT) as usize];
        put_pixel(&mut frame, -1, 0, 7);
        put_pixel(&mut frame, WIDTH as i32, 0, 7);
        put_pixel(&mut frame, 3, 2, 9);
        assert_eq!(frame.iter().filter(|&&p| p != 0).count(), 1);
        assert_eq!(frame[2 * WIDTH as usize + 3], 9);
    }
}
