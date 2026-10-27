//! Filled polygons (AreaMove / AreaDraw / AreaEnd) and the layers that clip them.

use super::bitmap::plot;
use crate::amiga::*;
use std::cell::RefCell;
use std::os::raw::c_void;

thread_local! {
    static AREA: RefCell<Vec<(i32, i32)>> = const { RefCell::new(Vec::new()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn InitArea(_a: *mut AreaInfo, _buf: *mut c_void, _max: LONG) {}

#[unsafe(no_mangle)]
pub extern "C" fn InitTmpRas(t: *mut TmpRas, _buf: PLANEPTR, _size: LONG) -> *mut TmpRas {
    t
}

#[unsafe(no_mangle)]
pub extern "C" fn AreaMove(_rp: *mut RastPort, x: LONG, y: LONG) -> LONG {
    AREA.with(|a| {
        let mut a = a.borrow_mut();
        a.clear();
        a.push((x, y));
    });

    0
}

#[unsafe(no_mangle)]
pub extern "C" fn AreaDraw(_rp: *mut RastPort, x: LONG, y: LONG) -> LONG {
    AREA.with(|a| a.borrow_mut().push((x, y)));

    0
}

/// The x positions, sorted, where the polygon's edges cross the pixel row `y`.
fn crossings(pts: &[(i32, i32)], y: i32) -> Vec<i32> {
    let mut xs = Vec::new();

    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);

        if (a.1 <= y && b.1 > y) || (b.1 <= y && a.1 > y) {
            xs.push(a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1));
        }
    }

    xs.sort();

    xs
}

/// Fills the polygon collected so far (even-odd rule).
#[unsafe(no_mangle)]
pub extern "C" fn AreaEnd(rp: *mut RastPort) -> LONG {
    let rp = unsafe { &*rp };
    let pts = AREA.with(|a| std::mem::take(&mut *a.borrow_mut()));

    if rp.BitMap.is_null() || pts.len() < 3 {
        return 0;
    }

    let bm = unsafe { &*rp.BitMap };

    // Masks with COMPLEMENT, keeping only the bits that matter here.
    let complement = rp.DrawMode & COMPLEMENT != 0;

    // a layer limits drawing to its rectangle (x1/y1 are the last pixel)
    let (cx0, cy0, cx1, cy1) = match unsafe { rp.Layer.as_ref() } {
        Some(l) => (l.x0, l.y0, l.x1, l.y1),
        None => (i32::MIN, i32::MIN, i32::MAX, i32::MAX),
    };

    let (ymin, ymax) = pts.iter().fold((i32::MAX, i32::MIN), |(lo, hi), p| {
        (lo.min(p.1), hi.max(p.1))
    });

    let (ymin, ymax) = (ymin.max(cy0), ymax.min(cy1));

    for y in ymin..=ymax {
        for pair in crossings(&pts, y).chunks(2) {
            if let [x0, x1] = pair {
                for x in (*x0).max(cx0)..=(*x1).min(cx1) {
                    unsafe { plot(bm, x, y, rp.FgPen, complement) };
                }
            }
        }
    }

    0
}

// ---- layers (only used to clip the witch's polygon to the map area) ----

#[unsafe(no_mangle)]
pub extern "C" fn NewLayerInfo() -> *mut c_void {
    unsafe { libc::calloc(1, 64) }
}

#[unsafe(no_mangle)]
pub extern "C" fn DisposeLayerInfo(li: *mut c_void) {
    unsafe { libc::free(li) }
}

#[unsafe(no_mangle)]
pub extern "C" fn CreateUpfrontLayer(
    _li: *mut c_void,
    bm: *mut BitMap,
    x0: LONG,
    y0: LONG,
    x1: LONG,
    y1: LONG,
    _flags: LONG,
    _super: *mut BitMap,
) -> *mut Layer {
    let layer = unsafe { libc::calloc(1, std::mem::size_of::<Layer>()) } as *mut Layer;
    unsafe {
        *layer = Layer {
            rp_bitmap: bm,
            x0,
            y0,
            x1,
            y1,
        }
    };

    layer
}

#[unsafe(no_mangle)]
pub extern "C" fn DeleteLayer(_dummy: *mut c_void, layer: *mut Layer) -> LONG {
    unsafe { libc::free(layer as *mut c_void) };

    1
}
