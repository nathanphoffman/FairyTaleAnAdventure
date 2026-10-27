//! The witch's spinning beam (from csrc/fmain2.c): a filled quadrilateral drawn in XOR mode
//! in a temporary layer, with the sign of the hero's side of each edge kept for hit tests.
//!
//! The beam is the area between two of 64 directions around the witch (`witchpoints` lists the near
//! and far end of each). `witch_fx` draws it with the exclusive-or drawing mode, so drawing it a
//! second time erases it. It also records, in `s1` and `s2`, on which side of each edge the hero
//! stands, which csrc/fmain.c uses to decide whether the beam hits the hero.

use crate::amiga::{AreaInfo, Layer, RastPort, TmpRas};
use crate::game::data::globals::layer_info;
use crate::game::data::object_tables::witchpoints;
use crate::game::data::state::{the_hero, the_screen, the_world};
use crate::game::data::types::FPage;

use crate::platform::gfx::{
    alloc_raster, area_draw, area_end, area_move, create_upfront_layer, delete_layer, free_raster,
    init_area, init_tmp_ras, set_draw_mode,
};

/// Which side of each beam edge the hero is on (the cross product and whether it is not negative).
/// Read by the witch's steering in csrc/fmain.c under the names `s1` and `s2`.
pub static mut SIDE_1: i16 = 0;
pub static mut SIDE_2: i16 = 0;
static mut SIDE_FLAG_1: i16 = 0;
static mut SIDE_FLAG_2: i16 = 0;

// Work areas for filling the beam.
static mut MY_TMP: TmpRas = TmpRas {
    RasPtr: std::ptr::null_mut(),
    Size: 0,
};

static mut MY_AREA_INFO: AreaInfo = AreaInfo {
    VctrTbl: std::ptr::null_mut(),
    VctrPtr: std::ptr::null_mut(),
    FlagTbl: std::ptr::null_mut(),
    FlagPtr: std::ptr::null_mut(),
    Count: 0,
    MaxCount: 0,
};

static mut AREA_BUFFER: [i16; 20] = [0; 20];
static mut MY_RAS: *mut u8 = std::ptr::null_mut();
static mut LAYER: *mut Layer = std::ptr::null_mut();
static mut OLD_LAYER: *mut Layer = std::ptr::null_mut();

const LAYER_SIMPLE: i32 = 1;
const COMPLEMENT: u32 = 2;

/// The layer covers the 19 x 6 tiles of the play field.
const LAYER_WIDTH: i32 = 16 * 19;
const LAYER_HEIGHT: i32 = 6 * 32;

/// Work raster for filling the beam.
const RASTER_SIZE: u32 = 100;
const AREA_MAX_VECTORS: i32 = 9;

/// The map position is cut to whole tiles (16 wide, 32 tall).
const TILE_X_MASK: u16 = 0xfff0;
const TILE_Y_MASK: u16 = 0xffe0;

/// The beam has 64 directions, 4 bytes (two points) each.
const POINT_BYTES: i32 = 4;
const DIRECTIONS_BEFORE: i32 = 63;
const DIRECTIONS_AFTER: i32 = 1;

/// RASSIZE(w, h) from the Amiga headers: bytes needed for a raster.
/// A raster row is a whole number of 16 bit words: round the pixel width up to a multiple of 16,
/// then divide by 8 to get bytes.
const WORD_ROUNDING: u32 = 15;
const BYTE_SHIFT: u32 = 3;
const EVEN_BYTES_MASK: u32 = 0xfffe;

/// An edge is two (x, y) points of one byte each, so 4 bytes; the beam polygon has 4 corners.
const EDGE_BYTES: usize = 4;
const BEAM_CORNERS: usize = 4;

fn raster_bytes(width: u32, height: u32) -> i32 {
    // Bytes in the raster: the width in pixels rounded up to a whole word, as bytes, times the height.
    (height * ((width + WORD_ROUNDING) >> BYTE_SHIFT & EVEN_BYTES_MASK)) as i32
}

/// The two points of the beam edge for direction `dir`, relative to the witch.
fn edge(dir: i16, fp: &FPage) -> ((i16, i16), (i16, i16)) {
    {
        let start = ((dir as i32 * POINT_BYTES) as u8) as usize;
        let w = &witchpoints[start..start + EDGE_BYTES];

        (
            (w[0] as i16 + fp.witchx, w[1] as i16 + fp.witchy),
            (w[2] as i16 + fp.witchx, w[3] as i16 + fp.witchy),
        )
    }
}

/// Which side of the line (p1, p2) the point (xh, yh) is on: the cross product, as the 16 bit
/// value the original kept, and 1 if it is not negative.
fn side(p1: (i16, i16), p2: (i16, i16), xh: i16, yh: i16) -> (i16, i16) {
    let (dx, dy) = (p2.0.wrapping_sub(p1.0), p2.1.wrapping_sub(p1.1));
    let (dx1, dy1) = (xh.wrapping_sub(p1.0), yh.wrapping_sub(p1.1));
    let cross = (dx as i32 * dy1 as i32 - dy as i32 * dx1 as i32) as i16;

    (cross, (cross >= 0) as i16)
}

/// Draws the witch's beam on page `fp` and records on which side of its edges the hero is.
/// The drawing is clipped to a temporary layer covering just the map area.
pub fn witch_fx(fp: *mut FPage) {
    unsafe {
        let fp = &*fp;

        // Masks with TILE_X_MASK, keeping only the bits that matter here.
        let xh = the_hero.x.wrapping_sub(the_world.map_x & TILE_X_MASK) as i16;

        // Masks with TILE_Y_MASK, keeping only the bits that matter here.
        let yh = the_hero.y.wrapping_sub(the_world.map_y & TILE_Y_MASK) as i16;

        LAYER = create_upfront_layer(
            layer_info,
            the_screen.rp_map.BitMap,
            0,
            0,
            LAYER_WIDTH,
            LAYER_HEIGHT,
            LAYER_SIMPLE,
            std::ptr::null_mut(),
        );

        let r = &raw mut the_screen.rp_map;
        OLD_LAYER = (*r).Layer;
        (*r).Layer = LAYER;

        let (p1, p2) = edge(fp.witchdir.wrapping_add(DIRECTIONS_BEFORE as i16), fp);
        let (p3, p4) = edge(fp.witchdir.wrapping_add(DIRECTIONS_AFTER as i16), fp);
        (SIDE_1, SIDE_FLAG_1) = side(p1, p2, xh, yh);
        (SIDE_2, SIDE_FLAG_2) = side(p3, p4, xh, yh);

        paint_beam(r, [p1, p2, p4, p3]);

        (*r).Layer = OLD_LAYER;
        delete_layer(std::ptr::null_mut(), LAYER);
    }
}

/// Fills the beam's quadrilateral with corners `p` in complement mode, using a temporary raster
/// and area buffer that are given back afterwards.
unsafe fn paint_beam(r: *mut RastPort, p: [(i16, i16); BEAM_CORNERS]) {
    unsafe {
        set_draw_mode(r, COMPLEMENT);
        MY_RAS = alloc_raster(RASTER_SIZE, RASTER_SIZE);

        if !MY_RAS.is_null() {
            let old_tmp = (*r).TmpRas;
            let old_area = (*r).AreaInfo;
            (*r).TmpRas = init_tmp_ras(
                &raw mut MY_TMP,
                MY_RAS,
                raster_bytes(RASTER_SIZE, RASTER_SIZE),
            );

            init_area(
                &raw mut MY_AREA_INFO,
                (&raw mut AREA_BUFFER).cast(),
                AREA_MAX_VECTORS,
            );

            (*r).AreaInfo = &raw mut MY_AREA_INFO;
            fill_beam(r, p);
            free_raster(MY_RAS, RASTER_SIZE, RASTER_SIZE);
            (*r).TmpRas = old_tmp;
            (*r).AreaInfo = old_area;
        }
    }
}

/// Fills the quadrilateral with corners `p` (in drawing order) on the map rastport `r`.
fn fill_beam(r: *mut RastPort, p: [(i16, i16); BEAM_CORNERS]) {
    area_move(r, p[0].0 as i32, p[0].1 as i32);

    for corner in &p[1..] {
        area_draw(r, corner.0 as i32, corner.1 as i32);
    }

    area_end(r);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raster_size_matches_the_amiga_macro() {
        assert_eq!(raster_bytes(100, 100), 1400);
    }

    #[test]
    fn side_tells_which_way_the_hero_is() {
        // line along x; a point below it (positive y) has a positive cross product
        assert_eq!(side((0, 0), (10, 0), 5, 5), (50, 1));
        assert_eq!(side((0, 0), (10, 0), 5, -5), (-50, 0));
    }
}
