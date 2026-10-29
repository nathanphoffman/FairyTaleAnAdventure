//! What is on screen: the view ports (rectangles of bitmaps with their colours) and the mouse
//! pointer, turned into the pixels of a frame.

use super::bitmap::Bitmap;

/// Size of the frame in pixels. The play field is lores (each of its pixels is shown twice as wide);
/// the text panel is hires.
pub const FRAME_WIDTH: usize = 640;
pub const FRAME_HEIGHT: usize = 200;

const LORES_SCALE: i32 = 2;

/// A rectangle of a bitmap shown on screen with its own colours.
pub struct ViewPort<'a> {
    pub bitmap: &'a Bitmap,
    /// The part of the bitmap shown starts here.
    pub scroll: (i32, i32),
    /// Where on screen it goes (in its own pixels) and how big it is.
    pub position: (i32, i32),
    pub size: (i32, i32),
    pub hires: bool,
    /// The colour table: 12 bit colours (0x0RGB).
    pub colors: &'a [u16],
    /// The mouse pointer is drawn with the colours of this view port.
    pub shows_pointer: bool,
}

/// The mouse pointer: a 16 pixel wide sprite.
pub struct Pointer<'a> {
    /// Each line is two 16 bit planes packed in one word; the first word is a control word.
    pub image: &'a [u32],
    /// Position in lores pixels.
    pub position: (i32, i32),
}

/// Everything shown: the view ports from the top one down, and the pointer.
pub struct Scene<'a> {
    pub view_ports: Vec<ViewPort<'a>>,
    pub pointer: Option<Pointer<'a>>,
}

const BITS_PER_BYTE: i32 = 8;
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

/// 12 bit colour (4 bits per channel, 0x0RGB) to 0xRRGGBB.
pub fn rgb4(c: u16) -> u32 {
    let r = ((c >> RED_NIBBLE_SHIFT) & NIBBLE_MASK) as u32;
    let g = ((c >> GREEN_NIBBLE_SHIFT) & NIBBLE_MASK) as u32;
    let b = (c & NIBBLE_MASK) as u32;

    // Puts the three components, each stretched from 4 to 8 bits, side by side in one pixel.
    ((r * NIBBLE_TO_BYTE) << RED_BYTE_SHIFT)
        | ((g * NIBBLE_TO_BYTE) << GREEN_BYTE_SHIFT)
        | (b * NIBBLE_TO_BYTE)
}

/// Sets a frame pixel if it lies inside the frame.
fn put_pixel(frame: &mut [u32], x: i32, y: i32, color: u32) {
    if x >= 0 && x < FRAME_WIDTH as i32 && y >= 0 && y < FRAME_HEIGHT as i32 {
        frame[y as usize * FRAME_WIDTH + x as usize] = color;
    }
}

/// Draws one line of the pointer sprite.
fn draw_pointer_line(frame: &mut [u32], colors: &[u16], word: u32, origin: (i32, i32)) {
    // A 32 bit value holds two 16 pixel rows of a bitplane.
    let (a, b) = ((word >> HIGH_WORD_SHIFT) as u16, word as u16);

    for bit in 0..POINTER_WIDTH {
        let shift = POINTER_WIDTH - 1 - bit;

        // One pixel of the two interleaved planes: plane a is the low bit, plane b the high bit.
        let v = (a >> shift & 1) | (b >> shift & 1) << 1;
        let Some(&c) = colors.get(POINTER_FIRST_COLOR + v as usize) else {
            continue;
        };

        if v == 0 {
            continue;
        }

        for dx in 0..LORES_SCALE {
            put_pixel(frame, origin.0 + bit * LORES_SCALE + dx, origin.1, rgb4(c));
        }
    }
}

/// Draws the mouse pointer: a 16 pixel wide sprite, 2 frame pixels per sprite pixel.
fn draw_pointer(frame: &mut [u32], colors: &[u16], pointer: &Pointer) {
    let (px, py) = pointer.position;

    for line in 0..POINTER_MAX_LINES {
        let word = pointer.image.get(1 + line).copied().unwrap_or(0);

        if word == 0 {
            break;
        }

        draw_pointer_line(frame, colors, word, (px * LORES_SCALE, py + line as i32));
    }
}

/// Draws one view port's rows into the frame.
fn draw_view_port(v: &ViewPort, frame: &mut [u32]) {
    let bm = v.bitmap;

    for row in 0..v.size.1 {
        let sy = v.scroll.1 + row;
        let oy = v.position.1 + row;

        if sy < 0 || sy >= bm.rows() as i32 || oy < 0 || oy >= FRAME_HEIGHT as i32 {
            continue;
        }

        for col in 0..v.size.0 {
            let sx = v.scroll.0 + col;

            if sx < 0 || sx >= bm.bytes_per_row() as i32 * BITS_PER_BYTE {
                continue;
            }

            let color = v.colors.get(bm.pixel(sx, sy)).map_or(0, |&c| rgb4(c));
            let screen_x = v.position.0 + col;

            if v.hires {
                put_pixel(frame, screen_x, oy, color);
            } else {
                put_pixel(frame, screen_x * LORES_SCALE, oy, color);
                put_pixel(frame, screen_x * LORES_SCALE + 1, oy, color);
            }
        }
    }
}

/// Draws the whole scene into the frame (`FRAME_WIDTH` x `FRAME_HEIGHT` pixels).
pub fn compose(scene: &Scene, frame: &mut [u32]) {
    frame.fill(0);

    for v in &scene.view_ports {
        draw_view_port(v, frame);
    }

    let pointer_colors = scene.view_ports.iter().find(|v| v.shows_pointer);

    if let (Some(pointer), Some(v)) = (&scene.pointer, pointer_colors) {
        draw_pointer(frame, v.colors, pointer);
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
        let mut frame = vec![0u32; FRAME_WIDTH * FRAME_HEIGHT];
        put_pixel(&mut frame, -1, 0, 7);
        put_pixel(&mut frame, FRAME_WIDTH as i32, 0, 7);
        put_pixel(&mut frame, 3, 2, 9);
        assert_eq!(frame.iter().filter(|&&p| p != 0).count(), 1);
        assert_eq!(frame[2 * FRAME_WIDTH + 3], 9);
    }

    #[test]
    fn a_lores_view_port_doubles_its_pixels() {
        let mut bm = Bitmap::new(16, 2, 1);
        bm.plot(0, 0, 1, false, 1);
        let colors = [0x000, 0xF00];
        let scene = Scene {
            view_ports: vec![ViewPort {
                bitmap: &bm,
                scroll: (0, 0),
                position: (1, 0),
                size: (16, 2),
                hires: false,
                colors: &colors,
                shows_pointer: false,
            }],
            pointer: None,
        };
        let mut frame = vec![0u32; FRAME_WIDTH * FRAME_HEIGHT];

        compose(&scene, &mut frame);

        assert_eq!(frame[2], 0xFF0000);
        assert_eq!(frame[3], 0xFF0000);
        assert_eq!(frame[4], 0);
    }

    #[test]
    fn colour_numbers_past_the_table_are_black() {
        let mut bm = Bitmap::new(16, 1, 2);
        bm.plot(0, 0, 3, false, 2);
        let colors = [0x000, 0xFFF];
        let scene = Scene {
            view_ports: vec![ViewPort {
                bitmap: &bm,
                scroll: (0, 0),
                position: (0, 0),
                size: (1, 1),
                hires: true,
                colors: &colors,
                shows_pointer: false,
            }],
            pointer: None,
        };
        let mut frame = vec![5u32; FRAME_WIDTH * FRAME_HEIGHT];

        compose(&scene, &mut frame);

        assert_eq!(frame[0], 0);
    }
}
