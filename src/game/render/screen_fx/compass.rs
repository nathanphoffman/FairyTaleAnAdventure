//! The compass on the text panel: which direction is lit.

use crate::game::data::object_tables::comptable;
use crate::game::data::state::{the_memory, the_screen};
use crate::platform::gfx::blt_bit_map;

const BLIT_COPY: u32 = 0xC0;

/// The compass only touches the plane the highlight is in.
const COMPASS_PLANE_MASK: u32 = 4;
const COMPASS_PLANE: usize = 2;
const COMPASS_X: i32 = 567;
const COMPASS_Y: i32 = 15;
const COMPASS_WIDTH: i32 = 48;
const COMPASS_HEIGHT: i32 = 24;

/// Compass directions below this are lit; the rest mean "not walking".
const COMPASS_DIRECTIONS: i16 = 9;

/// Redraws the compass with direction `dir` lit. First the unlit compass is copied, then the lit
/// part for that direction (if any) is copied on top from a second image.
pub fn draw_compass(dir: i16) {
    unsafe {
        let part = &comptable[dir as usize];
        let (xr, yr) = (part.xrect as i32, part.yrect as i32);
        (*the_screen.bm_source).Planes[COMPASS_PLANE] = the_memory.nhinor;
        blt_bit_map(
            the_screen.bm_source,
            0,
            0,
            the_screen.bm_text,
            COMPASS_X,
            COMPASS_Y,
            COMPASS_WIDTH,
            COMPASS_HEIGHT,
            BLIT_COPY,
            COMPASS_PLANE_MASK,
            std::ptr::null_mut(),
        );

        if dir < COMPASS_DIRECTIONS {
            (*the_screen.bm_source).Planes[COMPASS_PLANE] = the_memory.nhivar;
            blt_bit_map(
                the_screen.bm_source,
                xr,
                yr,
                the_screen.bm_text,
                COMPASS_X + xr,
                COMPASS_Y + yr,
                part.xsize as i32,
                part.ysize as i32,
                BLIT_COPY,
                COMPASS_PLANE_MASK,
                std::ptr::null_mut(),
            );
        }
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
