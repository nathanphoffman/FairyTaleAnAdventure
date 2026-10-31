//! Wooden planks, made up in code: a board with grain, a lighter top and left edge, a darker bottom
//! and right edge, and a soft shadow under it, as if the board lay on a table. They are made once
//! (they take a moment) and then drawn as ready-made pictures.

/// The shadow reaches this far right of and below the board.
pub const SHADOW: usize = 7;

/// Wood colours, darkest and lightest grain.
const DARK_GRAIN: (f32, f32, f32) = (88.0, 52.0, 24.0);
const LIGHT_GRAIN: (f32, f32, f32) = (176.0, 118.0, 66.0);

/// A hovered board is lit: brighter and a little warmer.
const LIT_BRIGHTNESS: f32 = 1.22;

/// Width of the bevelled edge, and the outline colour.
const BEVEL: usize = 2;
const OUTLINE: u32 = 0xff2a_1608;

/// How dark the shadow is at its darkest (out of 255) and how far its soft edge reaches.
const SHADOW_ALPHA: f32 = 150.0;
const SHADOW_SOFT_EDGE: f32 = 5.0;

/// A board `width` x `height` pixels, with room for its shadow, as 0xAARRGGBB pixels (so
/// `width + SHADOW` wide and `height + SHADOW` tall). `seed` makes each board's grain different.
pub fn plank(width: usize, height: usize, seed: u32, lit: bool) -> Vec<u32> {
    let (full_width, full_height) = (width + SHADOW, height + SHADOW);
    let mut pixels = vec![0u32; full_width * full_height];

    for y in 0..full_height {
        for x in 0..full_width {
            pixels[y * full_width + x] = if x < width && y < height {
                board_pixel((x, y), (width, height), seed, lit)
            } else {
                shadow_pixel((x, y), (width, height))
            };
        }
    }

    pixels
}

/// One pixel of the board: grain, then the bevel and outline.
fn board_pixel(
    (x, y): (usize, usize),
    (width, height): (usize, usize),
    seed: u32,
    lit: bool,
) -> u32 {
    let on_edge = x == 0 || y == 0 || x == width - 1 || y == height - 1;

    if on_edge {
        return OUTLINE;
    }

    let mut color = grain((x, y), seed);

    // Light falls from the top left: those edges are lit, the others in shade.
    let bevel = if y <= BEVEL || x <= BEVEL {
        32.0
    } else if y >= height - 1 - BEVEL || x >= width - 1 - BEVEL {
        -46.0
    } else {
        0.0
    };

    color = (color.0 + bevel, color.1 + bevel, color.2 + bevel);

    if lit {
        color = (
            color.0 * LIT_BRIGHTNESS,
            color.1 * LIT_BRIGHTNESS,
            color.2 * LIT_BRIGHTNESS * 0.92,
        );
    }

    pack(color, 255)
}

/// The colour of the wood at a point: streaks that run along the board and wander a little, with
/// fine fibres on top.
fn grain((x, y): (usize, usize), seed: u32) -> (f32, f32, f32) {
    let (fx, fy) = (x as f32, y as f32);
    let wander = noise(fx * 0.012, fy * 0.15, seed) * 12.0 + 4.0 * (fx * 0.021 + seed as f32).sin();
    let streak =
        0.5 + 0.5 * ((fy + wander) * 0.46 + noise(fx * 0.05, fy * 0.4, seed + 1) * 2.6).sin();
    let fibre = noise(fx * 0.55, fy * 0.9, seed + 2);
    let tone = (streak * 0.62 + fibre * 0.38).clamp(0.0, 1.0);

    (
        DARK_GRAIN.0 + (LIGHT_GRAIN.0 - DARK_GRAIN.0) * tone,
        DARK_GRAIN.1 + (LIGHT_GRAIN.1 - DARK_GRAIN.1) * tone,
        DARK_GRAIN.2 + (LIGHT_GRAIN.2 - DARK_GRAIN.2) * tone,
    )
}

/// One pixel of the shadow: black, see-through, and fading out towards the edge. The shadow is the
/// board's shape moved down and right.
fn shadow_pixel((x, y): (usize, usize), (width, height): (usize, usize)) -> u32 {
    let offset = (SHADOW / 2) as f32;
    let (fx, fy) = (x as f32, y as f32);

    // How far inside the moved board's edges the point is.
    let inside = (fx - offset)
        .min(width as f32 + offset - fx)
        .min(fy - offset)
        .min(height as f32 + offset - fy);

    if inside <= 0.0 {
        return 0;
    }

    let alpha = SHADOW_ALPHA * (inside / SHADOW_SOFT_EDGE).min(1.0);

    (alpha as u32) << 24
}

/// Smooth random values between 0 and 1: each whole-number point has a fixed random value and the
/// points between are blended.
fn noise(x: f32, y: f32, seed: u32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (tx, ty) = (smooth(x - ix), smooth(y - iy));
    let corner = |dx: f32, dy: f32| random_at((ix + dx) as i32, (iy + dy) as i32, seed);
    let top = corner(0.0, 0.0) + (corner(1.0, 0.0) - corner(0.0, 0.0)) * tx;
    let bottom = corner(0.0, 1.0) + (corner(1.0, 1.0) - corner(0.0, 1.0)) * tx;

    top + (bottom - top) * ty
}

/// Eases 0 to 1 so the blend has no hard corners.
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// A fixed pseudo-random value between 0 and 1 for a point.
fn random_at(x: i32, y: i32, seed: u32) -> f32 {
    let mut hash = (x as u32).wrapping_mul(0x9e37_79b1)
        ^ (y as u32).wrapping_mul(0x85eb_ca6b)
        ^ seed.wrapping_mul(0xc2b2_ae35);

    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x2c1b_3c6d);
    hash ^= hash >> 12;

    (hash & 0xffff) as f32 / 65535.0
}

/// Red, green and blue (0 to 255, clamped) and an alpha as 0xAARRGGBB.
fn pack((red, green, blue): (f32, f32, f32), alpha: u32) -> u32 {
    let channel = |value: f32| value.clamp(0.0, 255.0) as u32;

    alpha << 24 | channel(red) << 16 | channel(green) << 8 | channel(blue)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn average_brightness(pixels: &[u32], from: usize, to: usize) -> u32 {
        let sum: u32 = pixels[from..to]
            .iter()
            .map(|p| (p >> 16 & 0xff) + (p >> 8 & 0xff) + (p & 0xff))
            .sum();

        sum / (to - from) as u32
    }

    #[test]
    fn the_board_is_solid_and_the_far_corner_is_clear() {
        let (width, height) = (60, 30);
        let pixels = plank(width, height, 1, false);
        let full_width = width + SHADOW;

        assert_eq!(pixels.len(), full_width * (height + SHADOW));
        assert_eq!(pixels[10 * full_width + 10] >> 24, 255);
        assert_eq!(*pixels.last().unwrap(), 0);
    }

    #[test]
    fn the_shadow_is_see_through_and_lies_below_and_to_the_right() {
        let (width, height) = (60, 30);
        let pixels = plank(width, height, 1, false);
        let full_width = width + SHADOW;
        let below = pixels[(height + 2) * full_width + 20] >> 24;
        let right = pixels[15 * full_width + width + 2] >> 24;

        assert!(below > 0 && below < 255, "below: {below}");
        assert!(right > 0 && right < 255, "right: {right}");
        assert_eq!(pixels[(height + 2) * full_width + 2] >> 24, 0);
    }

    #[test]
    fn a_lit_board_is_brighter() {
        let (width, height) = (80, 40);
        let plain = plank(width, height, 3, false);
        let lit = plank(width, height, 3, true);
        let row = 20 * (width + SHADOW);

        assert!(
            average_brightness(&lit, row, row + width)
                > average_brightness(&plain, row, row + width)
        );
    }

    #[test]
    fn boards_are_the_same_every_time_and_differ_by_seed() {
        assert_eq!(plank(50, 20, 4, false), plank(50, 20, 4, false));
        assert_ne!(plank(50, 20, 4, false), plank(50, 20, 5, false));
    }

    #[test]
    fn the_grain_runs_along_the_board() {
        // Along a row the tone changes slowly; down a column it changes quickly.
        let pixels = plank(200, 60, 2, false);
        let full_width = 200 + SHADOW;
        let tone = |x: usize, y: usize| (pixels[y * full_width + x] >> 8 & 0xff) as i32;

        let across: i32 = (10..110)
            .map(|x| (tone(x + 1, 30) - tone(x, 30)).abs())
            .sum();
        let down: i32 = (10..50)
            .map(|y| (tone(60, y + 1) - tone(60, y)).abs())
            .sum::<i32>()
            * 100
            / 40;

        assert!(down > across, "down {down}, across {across}");
    }
}
