//! Drawing into a bitmap with a pen: filled rectangles, lines, filled polygons and text.

use super::bitmap::{Bitmap, Rect};
use super::font::Font;

/// How drawing combines with what is there.
pub const JAM1: u8 = 0;
pub const JAM2: u8 = 1;
pub const COMPLEMENT: u8 = 2;

/// The pen: its two colours, its drawing mode and where it is.
#[derive(Clone, Copy, Debug)]
pub struct Pen {
    /// Colour of what is drawn.
    pub fg: u8,
    /// Colour of the background of text (in `JAM2` mode).
    pub bg: u8,
    pub mode: u8,
    pub x: i32,
    pub y: i32,
}

impl Pen {
    pub const fn new() -> Pen {
        Pen {
            fg: 1,
            bg: 0,
            mode: JAM2,
            x: 0,
            y: 0,
        }
    }

    fn complement(&self) -> bool {
        self.mode & COMPLEMENT != 0
    }
}

impl Default for Pen {
    fn default() -> Pen {
        Pen::new()
    }
}

/// Fills the rectangle (corners inclusive) with the pen colour, or flips it in `COMPLEMENT` mode.
/// It is cut to the bitmap.
pub fn rect_fill(
    bm: &mut Bitmap,
    pen: &Pen,
    (x0, y0): (i32, i32),
    (x1, y1): (i32, i32),
    planes: usize,
) {
    for y in y0.max(0)..=y1.min(bm.rows() as i32 - 1) {
        for x in x0.max(0)..=x1 {
            bm.plot(x, y, pen.fg, pen.complement(), planes);
        }
    }
}

/// Draws a line from the pen to (`x`, `y`) and moves the pen there.
pub fn line_to(bm: &mut Bitmap, pen: &mut Pen, (x, y): (i32, i32), planes: usize) {
    let (mut cx, mut cy) = (pen.x, pen.y);
    let (dx, dy) = ((x - cx).abs(), -(y - cy).abs());
    let (sx, sy) = (if cx < x { 1 } else { -1 }, if cy < y { 1 } else { -1 });
    let mut err = dx + dy;

    loop {
        bm.plot(cx, cy, pen.fg, pen.complement(), planes);

        if cx == x && cy == y {
            break;
        }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            cx += sx;
        }

        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }

    pen.x = x;
    pen.y = y;
}

/// The x positions, sorted, where the edges of the polygon cross the pixel row `y`.
fn crossings(points: &[(i32, i32)], y: i32) -> Vec<i32> {
    let mut xs = Vec::new();

    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);

        if (a.1 <= y && b.1 > y) || (b.1 <= y && a.1 > y) {
            xs.push(a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1));
        }
    }

    xs.sort();

    xs
}

/// Fills a polygon (even-odd rule) with the pen colour, or flips it in `COMPLEMENT` mode. Only
/// pixels inside `clip` (corners inclusive) are touched.
pub fn fill_polygon(bm: &mut Bitmap, pen: &Pen, points: &[(i32, i32)], clip: Rect, planes: usize) {
    if points.len() < 3 {
        return;
    }

    let (cx1, cy1) = (clip.x + clip.width - 1, clip.y + clip.height - 1);
    let ymin = points.iter().map(|p| p.1).min().unwrap_or(0).max(clip.y);
    let ymax = points.iter().map(|p| p.1).max().unwrap_or(0).min(cy1);

    for y in ymin..=ymax {
        for pair in crossings(points, y).chunks(2) {
            if let [x0, x1] = pair {
                for x in (*x0).max(clip.x)..=(*x1).min(cx1) {
                    bm.plot(x, y, pen.fg, pen.complement(), planes);
                }
            }
        }
    }
}

/// Draws the bytes of `text` at the pen (which sits on the baseline) and moves the pen along.
pub fn text(bm: &mut Bitmap, pen: &mut Pen, font: &Font, text: &[u8], planes: usize) {
    for &ch in text {
        pen.x += font.draw_glyph(bm, pen, ch, planes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pen(fg: u8) -> Pen {
        Pen { fg, ..Pen::new() }
    }

    #[test]
    fn rectangles_are_cut_to_the_bitmap() {
        let mut bm = Bitmap::new(16, 4, 1);
        rect_fill(&mut bm, &pen(1), (-5, -5), (100, 1), 1);

        assert_eq!(bm.pixel(0, 0), 1);
        assert_eq!(bm.pixel(15, 1), 1);
        assert_eq!(bm.pixel(0, 2), 0);
    }

    #[test]
    fn a_line_moves_the_pen_and_includes_both_ends() {
        let mut bm = Bitmap::new(16, 8, 1);
        let mut p = pen(1);
        p.x = 1;
        p.y = 1;
        line_to(&mut bm, &mut p, (5, 3), 1);

        assert_eq!((p.x, p.y), (5, 3));
        assert_eq!(bm.pixel(1, 1), 1);
        assert_eq!(bm.pixel(5, 3), 1);
    }

    #[test]
    fn polygons_fill_inside_and_stay_in_the_clip() {
        let mut bm = Bitmap::new(32, 16, 1);
        let square = [(2, 2), (10, 2), (10, 10), (2, 10)];
        let clip = Rect {
            x: 0,
            y: 0,
            width: 6,
            height: 16,
        };

        fill_polygon(&mut bm, &pen(1), &square, clip, 1);

        assert_eq!(bm.pixel(3, 5), 1);
        assert_eq!(bm.pixel(8, 5), 0);
    }

    #[test]
    fn complement_mode_draws_twice_to_erase() {
        let mut bm = Bitmap::new(16, 4, 1);
        let mut p = pen(1);
        p.mode = COMPLEMENT;

        rect_fill(&mut bm, &p, (0, 0), (3, 3), 1);
        assert_eq!(bm.pixel(1, 1), 1);
        rect_fill(&mut bm, &p, (0, 0), (3, 3), 1);
        assert_eq!(bm.pixel(1, 1), 0);
    }
}
