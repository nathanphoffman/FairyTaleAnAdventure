//! Text and dark panels drawn over the finished frame, in true colour. The menu and the "Press ESC to
//! skip" message use it, so they do not depend on the colours of the picture underneath (and do not
//! disturb the bitmaps the game is animating).
//!
//! Positions are in overlay pixels: a 640 x 400 grid, the shape the game screen has once its rows
//! are doubled. A lores game pixel is 2 x 2 overlay pixels, so the overlay can draw text half the
//! size of the game's own: the 8x8 font at scale 1 is 8 overlay pixels tall, at scale 2 it is as
//! big as the classic game text.

use std::sync::Arc;

/// The overlay grid.
pub const WIDTH: i32 = 640;

/// The built-in font is 8x8 and its baseline is on its bottom row.
pub const GLYPH_SIZE: i32 = 8;
const GLYPH_BASELINE: i32 = 7;

pub const WHITE: u32 = 0xffff_ffff;

/// The message in the corner while the intro plays.
const HINT_RIGHT_EDGE: i32 = 632;
const HINT_BASELINE: i32 = 390;
const HINT_PADDING: i32 = 6;
const HINT_DARKNESS: u8 = 230;

/// A rectangle that is darkened: `darkness` 0 leaves the picture alone, 255 makes it black.
#[derive(Clone, Debug, PartialEq)]
pub struct Shade {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub darkness: u8,
}

/// A ready-made picture, `width` x `height` pixels of 0xAARRGGBB, with its top left corner at
/// (`x`, `y`). Pixels with an alpha under 255 let the picture below show through. The pixels are
/// shared, so copying an `Image` is cheap.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub pixels: Arc<[u32]>,
}

/// A line of text with its left end and baseline at (`x`, `y`), an 0xAARRGGBB colour, and a
/// scale: each pixel of the 8x8 font is `scale` x `scale` overlay pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub color: u32,
    pub scale: i32,
}

/// How wide `text` is at `scale`.
pub fn text_width(text: &str, scale: i32) -> i32 {
    text.len() as i32 * GLYPH_SIZE * scale
}

/// Everything to draw over the picture: first the shades, then the pictures, then the text on top.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Overlay {
    pub shades: Vec<Shade>,
    pub images: Vec<Image>,
    pub texts: Vec<Text>,
}

impl Overlay {
    /// A short message on a dark box in the bottom right corner.
    pub fn corner_hint(text: &str) -> Overlay {
        let width = text_width(text, 1);
        let x = HINT_RIGHT_EDGE - width;

        Overlay {
            shades: vec![Shade {
                x: x - HINT_PADDING,
                y: HINT_BASELINE - GLYPH_BASELINE - HINT_PADDING,
                width: width + HINT_PADDING * 2,
                height: GLYPH_SIZE + HINT_PADDING * 2,
                darkness: HINT_DARKNESS,
            }],
            images: vec![],
            texts: vec![Text {
                text: text.to_string(),
                x,
                y: HINT_BASELINE,
                color: WHITE,
                scale: 1,
            }],
        }
    }

    /// Whether there is nothing to draw.
    pub fn is_empty(&self) -> bool {
        self.shades.is_empty() && self.images.is_empty() && self.texts.is_empty()
    }

    /// Draws the overlay on a frame `width` pixels wide (and so many rows tall).
    pub fn draw(&self, frame: &mut [u32], width: usize) {
        if width == 0 {
            return;
        }

        for shade in &self.shades {
            draw_shade(frame, width, shade);
        }

        for image in &self.images {
            draw_image(frame, width, image);
        }

        for text in &self.texts {
            draw_text(frame, width, text);
        }
    }
}

/// Darkens the pixels of a shade.
fn draw_shade(frame: &mut [u32], width: usize, shade: &Shade) {
    let height = frame.len() / width;
    let clip = |value: i32, limit: usize| (value.max(0) as usize).min(limit);
    let (left, right) = (clip(shade.x, width), clip(shade.x + shade.width, width));
    let (top, bottom) = (clip(shade.y, height), clip(shade.y + shade.height, height));

    for row in top..bottom {
        for pixel in &mut frame[row * width + left..row * width + right.max(left)] {
            *pixel = darken(*pixel, shade.darkness);
        }
    }
}

/// Draws a picture: opaque pixels replace what is below, see-through ones are blended with it.
fn draw_image(frame: &mut [u32], width: usize, image: &Image) {
    let height = (frame.len() / width) as i32;

    for row in 0..image.height {
        let y = image.y + row;

        if !(0..height).contains(&y) {
            continue;
        }

        for column in 0..image.width {
            let x = image.x + column;
            let source = image.pixels[(row * image.width + column) as usize];

            if (0..width as i32).contains(&x) {
                let at = y as usize * width + x as usize;

                frame[at] = blend(frame[at], source);
            }
        }
    }
}

/// `top` over `below`, by the alpha of `top`.
fn blend(below: u32, top: u32) -> u32 {
    let alpha = top >> 24;

    match alpha {
        0 => below,
        255 => top,
        _ => {
            let mix = |shift: u32| {
                let (under, over) = (below >> shift & 0xff, top >> shift & 0xff);

                ((under * (255 - alpha) + over * alpha) / 255) << shift
            };

            0xff00_0000 | mix(16) | mix(8) | mix(0)
        }
    }
}

/// A colour with each of its red, green and blue parts scaled down by `darkness` out of 255.
fn darken(color: u32, darkness: u8) -> u32 {
    let keep = 255 - darkness as u32;
    let part = |shift: u32| ((color >> shift & 0xff) * keep / 255) << shift;

    0xff00_0000 | part(16) | part(8) | part(0)
}

/// Draws a line of text with the built-in font.
fn draw_text(frame: &mut [u32], width: usize, text: &Text) {
    let height = (frame.len() / width) as i32;
    let top = text.y - GLYPH_BASELINE * text.scale;

    for (index, byte) in text.text.bytes().enumerate() {
        let glyph = font8x8::legacy::BASIC_LEGACY[(byte & 0x7f) as usize];
        let left = text.x + index as i32 * GLYPH_SIZE * text.scale;

        for (row, bits) in glyph.iter().enumerate() {
            for column in 0..GLYPH_SIZE {
                // Bit 0 of each row is the leftmost pixel.
                if bits >> column & 1 != 0 {
                    let at = (left + column * text.scale, top + row as i32 * text.scale);

                    fill_block(frame, (width as i32, height), at, text);
                }
            }
        }
    }
}

/// Fills a square of `text.scale` pixels at `at` with the text colour, where it is on the frame.
fn fill_block(frame: &mut [u32], (width, height): (i32, i32), at: (i32, i32), text: &Text) {
    for y in at.1.max(0)..(at.1 + text.scale).min(height) {
        let start = at.0.max(0).min(width);
        let end = (at.0 + text.scale).clamp(start, width);

        frame[(y * width + start) as usize..(y * width + end) as usize].fill(text.color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROWS: usize = 400;
    const COLUMNS: usize = WIDTH as usize;

    fn text_at(x: i32, y: i32, scale: i32) -> Text {
        Text {
            text: "A".to_string(),
            x,
            y,
            color: 0xffff_0000,
            scale,
        }
    }

    #[test]
    fn a_shade_darkens_only_its_own_area() {
        let mut frame = vec![0xff80_8080u32; COLUMNS * ROWS];
        let overlay = Overlay {
            shades: vec![Shade {
                x: 10,
                y: 10,
                width: 10,
                height: 10,
                darkness: 255,
            }],
            ..Default::default()
        };

        overlay.draw(&mut frame, COLUMNS);

        assert_eq!(frame[10 * COLUMNS + 10], 0xff00_0000);
        assert_eq!(frame[9 * COLUMNS + 10], 0xff80_8080);
        assert_eq!(frame[10 * COLUMNS + 9], 0xff80_8080);
        assert_eq!(frame[10 * COLUMNS + 20], 0xff80_8080);
    }

    #[test]
    fn half_darkness_halves_the_colour() {
        assert_eq!(darken(0xffff_ffff, 128) & 0xff, 127);
        assert_eq!(darken(0xffff_ffff, 0), 0xffff_ffff);
    }

    #[test]
    fn text_lights_pixels_in_its_colour() {
        let mut frame = vec![0u32; COLUMNS * ROWS];
        let overlay = Overlay {
            texts: vec![text_at(100, 100, 1)],
            ..Default::default()
        };

        overlay.draw(&mut frame, COLUMNS);

        assert!(frame.contains(&0xffff_0000));
        assert!(frame[..92 * COLUMNS].iter().all(|&p| p == 0));
    }

    #[test]
    fn double_scale_text_is_twice_as_big() {
        let count = |scale| {
            let mut frame = vec![0u32; COLUMNS * ROWS];

            text_at(100, 100, scale).draw_into(&mut frame);

            frame.iter().filter(|&&p| p != 0).count()
        };

        assert_eq!(count(2), count(1) * 4);
    }

    #[test]
    fn the_corner_hint_lands_in_the_bottom_right() {
        let mut frame = vec![0xff80_8080u32; COLUMNS * ROWS];

        Overlay::corner_hint("Press ESC to skip").draw(&mut frame, COLUMNS);

        assert!(
            frame[..COLUMNS * (ROWS / 2)]
                .iter()
                .all(|&p| p == 0xff80_8080)
        );

        assert!(frame.contains(&WHITE));
        assert!(frame.contains(&darken(0xff80_8080, HINT_DARKNESS)));
    }

    #[test]
    fn drawing_off_the_frame_or_on_an_empty_one_is_harmless() {
        let overlay = Overlay::corner_hint("Press ESC to skip");

        overlay.draw(&mut [], COLUMNS);
        overlay.draw(&mut [0; 4], 0);
        overlay.draw(&mut vec![0; 10 * 10], 10);
    }

    #[test]
    fn an_image_covers_where_opaque_and_blends_where_not() {
        let mut frame = vec![0xff00_00ffu32; COLUMNS * ROWS];
        let overlay = Overlay {
            images: vec![Image {
                x: 5,
                y: 5,
                width: 2,
                height: 1,
                pixels: Arc::from(vec![0xffff_0000u32, 0x80ff_0000]),
            }],
            ..Default::default()
        };

        overlay.draw(&mut frame, COLUMNS);

        assert_eq!(frame[5 * COLUMNS + 5], 0xffff_0000);
        let blended = frame[5 * COLUMNS + 6];
        assert!(
            blended >> 16 & 0xff > 100 && blended & 0xff > 100,
            "{blended:x}"
        );

        assert_eq!(frame[5 * COLUMNS + 7], 0xff00_00ff);
    }

    #[test]
    fn an_empty_overlay_says_so() {
        assert!(Overlay::default().is_empty());
        assert!(!Overlay::corner_hint("x").is_empty());
    }
}

#[cfg(test)]
impl Text {
    /// How wide the text is.
    pub fn width(&self) -> i32 {
        text_width(&self.text, self.scale)
    }

    /// Draws just this text on a frame of `WIDTH` columns.
    fn draw_into(&self, frame: &mut [u32]) {
        draw_text(frame, WIDTH as usize, self);
    }
}
