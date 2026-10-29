//! Planar bitmaps: the bit planes of a picture, each pixel's colour number made of one bit from
//! each plane.

/// Extra bytes before and after each plane. The sprite blitter reads and writes a word or two outside
/// the visible area (as the original's memory allowed), so the planes have room for that.
pub const PAD: usize = 256;

/// 8 pixels fit in a byte, and the leftmost pixel of a byte is its top bit.
const PIXELS_PER_BYTE: usize = 8;
const LEFTMOST_PIXEL: u8 = 0x80;

/// Rows are a whole number of 16 pixel words (2 bytes each).
const WORD_PIXELS: usize = 16;
const WORD_BYTES: usize = 2;

/// Bytes in a row of `width` pixels.
pub fn bytes_per_row(width: usize) -> usize {
    width.div_ceil(WORD_PIXELS) * WORD_BYTES
}

/// A picture in bit planes. Plane `p` holds bit `p` of every pixel's colour number.
#[derive(Clone, Default)]
pub struct Bitmap {
    bytes_per_row: usize,
    rows: usize,
    planes: Vec<Vec<u8>>,
}

impl Bitmap {
    /// No planes at all; the state before a bitmap is set up.
    pub const fn empty() -> Bitmap {
        Bitmap {
            bytes_per_row: 0,
            rows: 0,
            planes: Vec::new(),
        }
    }

    /// A blank bitmap of `width` by `rows` pixels with `depth` planes.
    pub fn new(width: usize, rows: usize, depth: usize) -> Bitmap {
        let bytes_per_row = bytes_per_row(width);
        let plane = vec![0; bytes_per_row * rows + 2 * PAD];

        Bitmap {
            bytes_per_row,
            rows,
            planes: vec![plane; depth],
        }
    }

    /// A one plane bitmap whose rows are the first bytes of `data` (the rest of the plane is blank).
    pub fn from_plane(width: usize, rows: usize, data: &[u8]) -> Bitmap {
        let mut bitmap = Bitmap::new(width, rows, 1);
        let plane = bitmap.plane_mut(0);
        let n = plane.len().min(data.len());
        plane[..n].copy_from_slice(&data[..n]);

        bitmap
    }

    pub fn bytes_per_row(&self) -> usize {
        self.bytes_per_row
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn depth(&self) -> usize {
        self.planes.len()
    }

    /// Width in pixels (a whole number of words).
    pub fn width(&self) -> usize {
        self.bytes_per_row * PIXELS_PER_BYTE
    }

    fn visible_len(&self) -> usize {
        self.bytes_per_row * self.rows
    }

    /// The visible bytes of plane `p`, row after row.
    pub fn plane(&self, p: usize) -> &[u8] {
        &self.planes[p][PAD..PAD + self.visible_len()]
    }

    pub fn plane_mut(&mut self, p: usize) -> &mut [u8] {
        let len = self.visible_len();

        &mut self.planes[p][PAD..PAD + len]
    }

    /// Plane `p` with its padding: the visible bytes start `PAD` bytes in. The sprite blitter may
    /// reach a little outside the visible bytes, into the padding.
    pub fn padded_plane(&self, p: usize) -> &[u8] {
        &self.planes[p]
    }

    pub fn padded_plane_mut(&mut self, p: usize) -> &mut [u8] {
        &mut self.planes[p]
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (y as usize) < self.rows && (x as usize) < self.width()
    }

    fn byte_and_mask(&self, x: i32, y: i32) -> (usize, u8) {
        let byte = y as usize * self.bytes_per_row + x as usize / PIXELS_PER_BYTE;

        (byte, LEFTMOST_PIXEL >> (x as usize % PIXELS_PER_BYTE))
    }

    /// Bit `p` of the pixel at (`x`, `y`), which must be inside the bitmap.
    pub fn bit(&self, p: usize, x: i32, y: i32) -> bool {
        let (byte, mask) = self.byte_and_mask(x, y);

        self.plane(p)[byte] & mask != 0
    }

    pub fn set_bit(&mut self, p: usize, x: i32, y: i32, on: bool) {
        let (byte, mask) = self.byte_and_mask(x, y);
        let plane = self.plane_mut(p);

        if on {
            plane[byte] |= mask;
        } else {
            plane[byte] &= !mask;
        }
    }

    /// The colour number of the pixel at (`x`, `y`), which must be inside the bitmap.
    pub fn pixel(&self, x: i32, y: i32) -> usize {
        let (byte, mask) = self.byte_and_mask(x, y);

        (0..self.depth())
            .filter(|&p| self.plane(p)[byte] & mask != 0)
            .fold(0, |index, p| index | 1 << p)
    }

    /// Draws one pixel in colour `pen` on the first `planes` planes, or flips it (`complement`).
    /// Pixels outside the bitmap are ignored.
    pub fn plot(&mut self, x: i32, y: i32, pen: u8, complement: bool, planes: usize) {
        if !self.contains(x, y) {
            return;
        }

        for p in 0..self.depth().min(planes) {
            let on = if complement {
                !self.bit(p, x, y)
            } else {
                pen >> p & 1 != 0
            };

            self.set_bit(p, x, y, on);
        }
    }

    /// Fills the first `planes` planes with colour `pen`.
    pub fn clear(&mut self, pen: u8, planes: usize) {
        for p in 0..self.depth().min(planes) {
            // A plane is all ones if the pen has that bit set, and all zeros otherwise.
            let fill = if pen >> p & 1 != 0 { 0xff } else { 0 };

            self.plane_mut(p).fill(fill);
        }
    }
}

/// A rectangle of pixels.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Copies `rect` of plane `from` of `src` to plane `to` of `dst`, putting its top left corner at
/// (`dst_x`, `dst_y`). Parts outside either bitmap are skipped.
pub fn copy_plane_rect(
    src: &Bitmap,
    from: usize,
    dst: &mut Bitmap,
    to: usize,
    rect: Rect,
    (dst_x, dst_y): (i32, i32),
) {
    // Everything is read first so copies that overlap within one bitmap work.
    let bits = read_rect(src, from, rect);

    write_rect(dst, to, rect, (dst_x, dst_y), &bits);
}

/// The bits of `rect` in plane `p` of `src`, row after row (false outside the bitmap).
fn read_rect(src: &Bitmap, p: usize, rect: Rect) -> Vec<bool> {
    let mut bits = Vec::with_capacity((rect.width.max(0) * rect.height.max(0)) as usize);

    for y in 0..rect.height {
        for x in 0..rect.width {
            let (px, py) = (rect.x + x, rect.y + y);

            bits.push(src.contains(px, py) && src.bit(p, px, py));
        }
    }

    bits
}

fn write_rect(dst: &mut Bitmap, p: usize, rect: Rect, (dst_x, dst_y): (i32, i32), bits: &[bool]) {
    for y in 0..rect.height {
        for x in 0..rect.width {
            let (px, py) = (dst_x + x, dst_y + y);

            if dst.contains(px, py) {
                dst.set_bit(p, px, py, bits[(y * rect.width + x) as usize]);
            }
        }
    }
}

/// Copies `rect` from `src` to `dst` plane by plane for the planes `mask` selects (bit `p` for
/// plane `p`) that both bitmaps have. Returns how many planes were copied.
pub fn copy_rect(src: &Bitmap, dst: &mut Bitmap, rect: Rect, to: (i32, i32), mask: u32) -> usize {
    let planes: Vec<usize> = (0..src.depth().min(dst.depth()))
        .filter(|&p| mask >> p & 1 != 0)
        .collect();

    for &p in &planes {
        copy_plane_rect(src, p, dst, p, rect, to);
    }

    planes.len()
}

/// Scrolls the pixels of `area` (on the first `planes` planes) by (`dx`, `dy`): a pixel moves up and
/// left by that much. The strip that scrolls in at the edge is filled with colour `fill`.
pub fn scroll(bm: &mut Bitmap, area: Rect, (dx, dy): (i32, i32), fill: u8, planes: usize) {
    for p in 0..bm.depth().min(planes) {
        let bits = read_rect(bm, p, area);

        for y in 0..area.height {
            for x in 0..area.width {
                let (sx, sy) = (x + dx, y + dy);
                let inside = (0..area.width).contains(&sx) && (0..area.height).contains(&sy);
                let on = if inside {
                    bits[(sy * area.width + sx) as usize]
                } else {
                    fill >> p & 1 != 0
                };
                let (px, py) = (area.x + x, area.y + y);

                if bm.contains(px, py) {
                    bm.set_bit(p, px, py, on);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_whole_words() {
        assert_eq!(bytes_per_row(320), 40);
        assert_eq!(bytes_per_row(17), 4);
        assert_eq!(bytes_per_row(1), 2);
    }

    #[test]
    fn a_pixel_is_one_bit_from_each_plane() {
        let mut bm = Bitmap::new(32, 4, 5);
        bm.plot(9, 2, 0b10101, false, 5);

        assert_eq!(bm.pixel(9, 2), 0b10101);
        assert_eq!(bm.pixel(8, 2), 0);
        assert_eq!(bm.plane(0)[2 * 4 + 1], 0x40);
    }

    #[test]
    fn plotting_outside_is_ignored_and_complement_flips() {
        let mut bm = Bitmap::new(16, 2, 2);
        bm.plot(-1, 0, 3, false, 2);
        bm.plot(16, 0, 3, false, 2);
        bm.plot(0, 2, 3, false, 2);
        assert_eq!(bm.pixel(0, 0), 0);

        bm.plot(0, 0, 1, false, 2);
        bm.plot(0, 0, 0, true, 2);
        assert_eq!(bm.pixel(0, 0), 0b10);
    }

    #[test]
    fn clearing_fills_each_plane_by_its_pen_bit() {
        let mut bm = Bitmap::new(16, 2, 3);
        bm.clear(0b101, 3);

        assert_eq!(bm.pixel(5, 1), 0b101);
        bm.clear(0, 1);
        assert_eq!(bm.pixel(5, 1), 0b100);
    }

    #[test]
    fn copying_a_rectangle_between_bitmaps() {
        let mut src = Bitmap::new(16, 4, 2);
        src.plot(1, 1, 3, false, 2);
        let mut dst = Bitmap::new(16, 4, 2);
        let rect = Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };

        assert_eq!(copy_rect(&src, &mut dst, rect, (8, 0), 0b11), 2);
        assert_eq!(dst.pixel(9, 1), 3);
        assert_eq!(dst.pixel(1, 1), 0);
    }

    #[test]
    fn scrolling_moves_pixels_up_and_fills_the_bottom() {
        let mut bm = Bitmap::new(16, 4, 1);
        bm.plot(2, 3, 1, false, 1);
        let area = Rect {
            x: 0,
            y: 0,
            width: 16,
            height: 4,
        };

        scroll(&mut bm, area, (0, 1), 1, 1);
        assert_eq!(bm.pixel(2, 2), 1);
        assert_eq!(bm.pixel(2, 3), 1);

        scroll(&mut bm, area, (0, 1), 0, 1);
        assert_eq!(bm.pixel(2, 1), 1);
        assert_eq!(bm.pixel(2, 3), 0);
    }
}
