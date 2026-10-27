//! The witch's spinning beam (from csrc/fmain2.c): a filled quadrilateral drawn in XOR mode
//! in a temporary layer, with the sign of the hero's side of each edge kept for hit tests.
//!
//! The beam is the area between two of 64 directions around the witch (`witchpoints` lists the near
//! and far end of each). `witch_fx` draws it with the exclusive-or drawing mode, so drawing it a
//! second time erases it. It also records, in `s1` and `s2`, on which side of each edge the hero
//! stands, which csrc/fmain.c uses to decide whether the beam hits the hero.

use crate::amiga::{AreaInfo, BitMap, Layer, RastPort, TmpRas};
use crate::game::data::globals::layer_info;
use crate::game::data::object_tables::witchpoints;
use crate::game::data::state::{the_hero, the_screen, the_world};
use crate::game::data::types::FPage;

unsafe extern "C" {
    fn CreateUpfrontLayer(
        li: *mut std::ffi::c_void,
        bm: *mut BitMap,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        flags: i32,
        super_bitmap: *mut BitMap,
    ) -> *mut Layer;
    fn DeleteLayer(dummy: *mut std::ffi::c_void, layer: *mut Layer) -> i32;
    fn AllocRaster(w: u32, h: u32) -> *mut u8;
    fn FreeRaster(p: *mut u8, w: u32, h: u32);
    fn InitTmpRas(t: *mut TmpRas, buf: *mut u8, size: i32) -> *mut TmpRas;
    fn InitArea(a: *mut AreaInfo, buf: *mut i16, max_vectors: i32);
    fn AreaMove(current_rastport: *mut RastPort, x: i32, y: i32) -> i32;
    fn AreaDraw(current_rastport: *mut RastPort, x: i32, y: i32) -> i32;
    fn AreaEnd(current_rastport: *mut RastPort) -> i32;
    fn SetDrMd(current_rastport: *mut RastPort, mode: u32);
}

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
fn raster_bytes(width: u32, height: u32) -> i32 {
    (height * ((width + 15) >> 3 & 0xfffe)) as i32
}

/// The two points of the beam edge for direction `dir`, relative to the witch.
fn edge(dir: i16, fp: &FPage) -> ((i16, i16), (i16, i16)) {
    {
        let start = ((dir as i32 * POINT_BYTES) as u8) as usize;
        let w = &witchpoints[start..start + 4];
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
        let xh = the_hero.x.wrapping_sub(the_world.map_x & TILE_X_MASK) as i16;
        let yh = the_hero.y.wrapping_sub(the_world.map_y & TILE_Y_MASK) as i16;

        LAYER = CreateUpfrontLayer(
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

        SetDrMd(r, COMPLEMENT);
        MY_RAS = AllocRaster(RASTER_SIZE, RASTER_SIZE);
        if !MY_RAS.is_null() {
            let old_tmp = (*r).TmpRas;
            let old_area = (*r).AreaInfo;
            (*r).TmpRas = InitTmpRas(
                &raw mut MY_TMP,
                MY_RAS,
                raster_bytes(RASTER_SIZE, RASTER_SIZE),
            );
            InitArea(
                &raw mut MY_AREA_INFO,
                (&raw mut AREA_BUFFER).cast(),
                AREA_MAX_VECTORS,
            );
            (*r).AreaInfo = &raw mut MY_AREA_INFO;
            fill_beam(r, [p1, p2, p4, p3]);
            FreeRaster(MY_RAS, RASTER_SIZE, RASTER_SIZE);
            (*r).TmpRas = old_tmp;
            (*r).AreaInfo = old_area;
        }
        (*r).Layer = OLD_LAYER;
        DeleteLayer(std::ptr::null_mut(), LAYER);
    }
}

/// Fills the quadrilateral with corners `p` (in drawing order) on the map rastport `r`.
unsafe fn fill_beam(r: *mut RastPort, p: [(i16, i16); 4]) {
    unsafe {
        AreaMove(r, p[0].0 as i32, p[0].1 as i32);
        for corner in &p[1..] {
            AreaDraw(r, corner.0 as i32, corner.1 as i32);
        }
        AreaEnd(r);
    }
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
