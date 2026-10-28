//! The compass on the text panel: which direction is lit.

use crate::game::data::object_tables::comptable;
use crate::game::render::screen::Screen;
use crate::gfx::bitmap::Rect;

/// The compass is drawn in the third plane of the text panel.
const COMPASS_PLANE: usize = 2;
const COMPASS_X: i32 = 567;
const COMPASS_Y: i32 = 15;
const COMPASS_WIDTH: i32 = 48;
const COMPASS_HEIGHT: i32 = 24;

/// The two compass pictures: the plain one, and the one with every direction lit.
const PLAIN: usize = 0;
const LIT: usize = 1;

/// Compass directions below this are lit; the rest mean "not walking".
const COMPASS_DIRECTIONS: i16 = 9;

/// Redraws the compass with direction `dir` lit. First the unlit compass is copied, then the lit
/// part for that direction (if any) is copied on top from a second image.
pub fn draw_compass(screen: &mut Screen, dir: i16) {
    let part = &comptable[dir as usize];
    let (xr, yr) = (part.xrect as i32, part.yrect as i32);
    let whole = Rect {
        x: 0,
        y: 0,
        width: COMPASS_WIDTH,
        height: COMPASS_HEIGHT,
    };

    screen.copy_compass(PLAIN, whole, (COMPASS_X, COMPASS_Y), COMPASS_PLANE);

    if dir < COMPASS_DIRECTIONS {
        let lit = Rect {
            x: xr,
            y: yr,
            width: part.xsize as i32,
            height: part.ysize as i32,
        };

        screen.copy_compass(LIT, lit, (COMPASS_X + xr, COMPASS_Y + yr), COMPASS_PLANE);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn compass_part_is_four_bytes() {
        assert_eq!(
            size_of::<crate::game::data::object_tables::CompassPart>(),
            4
        );
    }
}
