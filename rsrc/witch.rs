//! The witch's spinning beam (from csrc/fmain2.c): a filled quadrilateral drawn in XOR mode
//! in a temporary layer, with the sign of the hero's side of each edge kept for hit tests.
//! The work areas it uses are still defined in csrc/fmain2.c.

use super::types::FPage;
use crate::amiga::{AreaInfo, BitMap, Layer, RastPort, TmpRas};

unsafe extern "C" {
    static hero_x: u16;
    static hero_y: u16;
    static map_x: u16;
    static map_y: u16;
    static mut rp_map: RastPort;
    static li: *mut std::ffi::c_void;
    static witchpoints: [i8; 256];
    static mut s1: i16;
    static mut s2: i16;
    static mut sg1: i16;
    static mut sg2: i16;
    static mut mytmp: TmpRas;
    static mut myAreaInfo: AreaInfo;
    static mut areabuffer: [i16; 20];
    static mut myras: *mut u8;
    static mut layer: *mut Layer;
    static mut oldlayer: *mut Layer;

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
    fn AreaMove(rp: *mut RastPort, x: i32, y: i32) -> i32;
    fn AreaDraw(rp: *mut RastPort, x: i32, y: i32) -> i32;
    fn AreaEnd(rp: *mut RastPort) -> i32;
    fn SetDrMd(rp: *mut RastPort, mode: u32);
}

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
unsafe fn edge(dir: i16, fp: &FPage) -> ((i16, i16), (i16, i16)) {
    unsafe {
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
#[unsafe(no_mangle)]
pub extern "C" fn witch_fx(fp: *mut FPage) {
    unsafe {
        let fp = &*fp;
        let xh = hero_x.wrapping_sub(map_x & TILE_X_MASK) as i16;
        let yh = hero_y.wrapping_sub(map_y & TILE_Y_MASK) as i16;

        layer = CreateUpfrontLayer(li, rp_map.BitMap, 0, 0, LAYER_WIDTH, LAYER_HEIGHT, LAYER_SIMPLE, std::ptr::null_mut());
        let r = &raw mut rp_map;
        oldlayer = (*r).Layer;
        (*r).Layer = layer;

        let (p1, p2) = edge(fp.witchdir.wrapping_add(DIRECTIONS_BEFORE as i16), fp);
        let (p3, p4) = edge(fp.witchdir.wrapping_add(DIRECTIONS_AFTER as i16), fp);
        (s1, sg1) = side(p1, p2, xh, yh);
        (s2, sg2) = side(p3, p4, xh, yh);

        SetDrMd(r, COMPLEMENT);
        myras = AllocRaster(RASTER_SIZE, RASTER_SIZE);
        if !myras.is_null() {
            let old_tmp = (*r).TmpRas;
            let old_area = (*r).AreaInfo;
            (*r).TmpRas = InitTmpRas(&raw mut mytmp, myras, raster_bytes(RASTER_SIZE, RASTER_SIZE));
            InitArea(&raw mut myAreaInfo, (&raw mut areabuffer).cast(), AREA_MAX_VECTORS);
            (*r).AreaInfo = &raw mut myAreaInfo;
            AreaMove(r, p1.0 as i32, p1.1 as i32);
            AreaDraw(r, p2.0 as i32, p2.1 as i32);
            AreaDraw(r, p4.0 as i32, p4.1 as i32);
            AreaDraw(r, p3.0 as i32, p3.1 as i32);
            AreaEnd(r);
            FreeRaster(myras, RASTER_SIZE, RASTER_SIZE);
            (*r).TmpRas = old_tmp;
            (*r).AreaInfo = old_area;
        }
        (*r).Layer = oldlayer;
        DeleteLayer(std::ptr::null_mut(), layer);
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
