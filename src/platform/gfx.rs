//! Amiga graphics library: planar bitmaps, rastports, drawing, text, palette, views.
//!
//! Bitmaps stay planar (one byte array per bitplane, 1 bit per pixel) exactly like the original,
//! because the game code reads and writes them directly. Only the final display step
//! (display.rs) converts them to RGB.

use crate::amiga::*;
use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr::null_mut;

// ---- bitmaps ----

fn bytes_per_row(width: i32) -> usize {
    (((width + 15) / 16) * 2) as usize
}

#[unsafe(no_mangle)]
pub extern "C" fn InitBitMap(bm: *mut BitMap, depth: LONG, width: LONG, height: LONG) {
    let bm = unsafe { &mut *bm };
    bm.BytesPerRow = bytes_per_row(width) as UWORD;
    bm.Rows = height as UWORD;
    bm.Flags = 0;
    bm.Depth = depth as UBYTE;
    bm.pad = 0;
    bm.Planes = [null_mut(); 8];
}

/// Allocates the planes too. Returns 1 on success (this one was assembly in the original).
#[unsafe(no_mangle)]
pub extern "C" fn MakeBitMap(bm: *mut BitMap, depth: LONG, width: LONG, height: LONG) -> LONG {
    InitBitMap(bm, depth, width, height);
    let b = unsafe { &mut *bm };
    for p in 0..depth as usize {
        b.Planes[p] = AllocRaster(width as ULONG, height as ULONG);
        if b.Planes[p].is_null() {
            UnMakeBitMap(bm);
            return 0;
        }
    }
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn UnMakeBitMap(bm: *mut BitMap) {
    let b = unsafe { &mut *bm };
    for p in 0..b.Depth as usize {
        FreeRaster(b.Planes[p], 0, 0);
        b.Planes[p] = null_mut();
    }
}

/// Bitplanes get padding on both sides: the blitter routines (csrc/asm_blit.c) read and write a
/// word or two outside the visible area, as the Amiga's memory allowed.
const RASTER_PAD: usize = 256;

#[unsafe(no_mangle)]
pub extern "C" fn AllocRaster(width: ULONG, height: ULONG) -> PLANEPTR {
    let size = bytes_per_row(width as i32) * height as usize + 2 * RASTER_PAD;
    let base = unsafe { libc::calloc(1, size) as PLANEPTR };
    if base.is_null() {
        base
    } else {
        unsafe { base.add(RASTER_PAD) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn FreeRaster(p: PLANEPTR, _w: ULONG, _h: ULONG) {
    if !p.is_null() {
        unsafe { libc::free(p.sub(RASTER_PAD) as *mut c_void) }
    }
}

/// Returns bit `x` of row `y` in a plane.
unsafe fn get_bit(bm: &BitMap, plane: usize, x: i32, y: i32) -> bool {
    unsafe {
        let base = bm.Planes[plane];
        let off = y as usize * bm.BytesPerRow as usize + (x >> 3) as usize;
        *base.add(off) & (0x80 >> (x & 7)) != 0
    }
}

unsafe fn set_bit(bm: &BitMap, plane: usize, x: i32, y: i32, on: bool) {
    unsafe {
        let base = bm.Planes[plane];
        let off = y as usize * bm.BytesPerRow as usize + (x >> 3) as usize;
        let mask = 0x80u8 >> (x & 7);
        if on {
            *base.add(off) |= mask
        } else {
            *base.add(off) &= !mask
        }
    }
}

fn in_bounds(bm: &BitMap, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && y < bm.Rows as i32 && x < bm.BytesPerRow as i32 * 8
}

/// Draws one pixel with `pen` (or flips it, in COMPLEMENT mode).
unsafe fn plot(bm: &BitMap, x: i32, y: i32, pen: u8, complement: bool) {
    if !in_bounds(bm, x, y) {
        return;
    }
    for p in 0..bm.Depth as usize {
        if bm.Planes[p].is_null() {
            continue;
        }
        unsafe {
            if complement {
                let cur = get_bit(bm, p, x, y);
                set_bit(bm, p, x, y, !cur);
            } else {
                set_bit(bm, p, x, y, pen >> p & 1 != 0);
            }
        }
    }
}

// ---- rastports and drawing ----

#[unsafe(no_mangle)]
pub extern "C" fn InitRastPort(rp: *mut RastPort) {
    let rp = unsafe { &mut *rp };
    rp.Layer = null_mut();
    rp.BitMap = null_mut();
    rp.AreaInfo = null_mut();
    rp.TmpRas = null_mut();
    rp.FgPen = 1;
    rp.BgPen = 0;
    rp.DrawMode = JAM2;
    rp.cp_x = 0;
    rp.cp_y = 0;
    rp.Font = null_mut();
}

#[unsafe(no_mangle)]
pub extern "C" fn SetAPen(rp: *mut RastPort, pen: ULONG) {
    unsafe { (*rp).FgPen = pen as u8 }
}

#[unsafe(no_mangle)]
pub extern "C" fn SetBPen(rp: *mut RastPort, pen: ULONG) {
    unsafe { (*rp).BgPen = pen as u8 }
}

#[unsafe(no_mangle)]
pub extern "C" fn SetDrMd(rp: *mut RastPort, mode: ULONG) {
    unsafe { (*rp).DrawMode = mode as u8 }
}

/// Fills the whole bitmap with one pen.
#[unsafe(no_mangle)]
pub extern "C" fn SetRast(rp: *mut RastPort, pen: ULONG) {
    let rp = unsafe { &*rp };
    if rp.BitMap.is_null() {
        return;
    }
    let bm = unsafe { &*rp.BitMap };
    let size = bm.BytesPerRow as usize * bm.Rows as usize;
    for p in 0..bm.Depth as usize {
        if !bm.Planes[p].is_null() {
            let v = if pen >> p & 1 != 0 { 0xff } else { 0 };
            unsafe { std::ptr::write_bytes(bm.Planes[p], v, size) };
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn RectFill(rp: *mut RastPort, x0: LONG, y0: LONG, x1: LONG, y1: LONG) {
    let rp = unsafe { &*rp };
    if rp.BitMap.is_null() {
        return;
    }
    let bm = unsafe { &*rp.BitMap };
    let complement = rp.DrawMode & COMPLEMENT != 0;
    for y in y0.max(0)..=y1.min(bm.Rows as i32 - 1) {
        for x in x0.max(0)..=x1 {
            unsafe { plot(bm, x, y, rp.FgPen, complement) };
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Move(rp: *mut RastPort, x: LONG, y: LONG) {
    let rp = unsafe { &mut *rp };
    rp.cp_x = x as WORD;
    rp.cp_y = y as WORD;
}

/// Line from the current pen position to (x, y), then moves the pen there.
#[unsafe(no_mangle)]
pub extern "C" fn Draw(rp: *mut RastPort, x: LONG, y: LONG) {
    let rp = unsafe { &mut *rp };
    if !rp.BitMap.is_null() {
        let bm = unsafe { &*rp.BitMap };
        let complement = rp.DrawMode & COMPLEMENT != 0;
        let (mut cx, mut cy) = (rp.cp_x as i32, rp.cp_y as i32);
        let (dx, dy) = ((x - cx).abs(), -(y - cy).abs());
        let (sx, sy) = (if cx < x { 1 } else { -1 }, if cy < y { 1 } else { -1 });
        let mut err = dx + dy;
        loop {
            unsafe { plot(bm, cx, cy, rp.FgPen, complement) };
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
    }
    rp.cp_x = x as WORD;
    rp.cp_y = y as WORD;
}

/// Copies the rectangle from src to dst for the planes selected by `mask`.
/// The game only ever uses minterm 0xC0 (plain copy).
#[unsafe(no_mangle)]
pub extern "C" fn BltBitMap(
    src: *mut BitMap,
    sx: LONG,
    sy: LONG,
    dst: *mut BitMap,
    dx: LONG,
    dy: LONG,
    w: LONG,
    h: LONG,
    minterm: ULONG,
    mask: ULONG,
    _temp: PLANEPTR,
) -> LONG {
    debug_assert_eq!(minterm, 0xC0, "only plain copy is implemented");
    let (src, dst) = unsafe { (&*src, &*dst) };
    let planes = src.Depth.min(dst.Depth) as usize;
    let mut copied = 0;
    let mut buf = vec![false; (w.max(0) * h.max(0)) as usize];
    for p in 0..planes {
        if mask >> p & 1 == 0 || src.Planes[p].is_null() || dst.Planes[p].is_null() {
            continue;
        }
        // read everything first so overlapping copies within one bitmap work
        for y in 0..h {
            for x in 0..w {
                let (px, py) = (sx + x, sy + y);
                buf[(y * w + x) as usize] =
                    in_bounds(src, px, py) && unsafe { get_bit(src, p, px, py) };
            }
        }
        for y in 0..h {
            for x in 0..w {
                let (px, py) = (dx + x, dy + y);
                if in_bounds(dst, px, py) {
                    unsafe { set_bit(dst, p, px, py, buf[(y * w + x) as usize]) };
                }
            }
        }
        copied += 1;
    }
    copied
}

/// Scrolls the rectangle's contents; the exposed strip is filled with the background pen.
#[unsafe(no_mangle)]
pub extern "C" fn ScrollRaster(
    rp: *mut RastPort,
    dx: LONG,
    dy: LONG,
    x0: LONG,
    y0: LONG,
    x1: LONG,
    y1: LONG,
) {
    let rp = unsafe { &*rp };
    if rp.BitMap.is_null() {
        return;
    }
    let bm = unsafe { &*rp.BitMap };
    let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
    let mut pixels = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut v = 0u8;
            for p in 0..bm.Depth as usize {
                if in_bounds(bm, x0 + x, y0 + y) && unsafe { get_bit(bm, p, x0 + x, y0 + y) } {
                    v |= 1 << p;
                }
            }
            pixels[(y * w + x) as usize] = v;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = (x + dx, y + dy);
            let v = if sx >= 0 && sy >= 0 && sx < w && sy < h {
                pixels[(sy * w + sx) as usize]
            } else {
                rp.BgPen
            };
            unsafe { plot(bm, x0 + x, y0 + y, v, false) };
        }
    }
}

// ---- filled polygons (AreaMove / AreaDraw / AreaEnd) ----

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

/// Fills the polygon collected so far (even-odd rule).
#[unsafe(no_mangle)]
pub extern "C" fn AreaEnd(rp: *mut RastPort) -> LONG {
    let rp = unsafe { &*rp };
    let pts = AREA.with(|a| std::mem::take(&mut *a.borrow_mut()));
    if rp.BitMap.is_null() || pts.len() < 3 {
        return 0;
    }
    let bm = unsafe { &*rp.BitMap };
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
        let mut xs = Vec::new();
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            if (a.1 <= y && b.1 > y) || (b.1 <= y && a.1 > y) {
                xs.push(a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1));
            }
        }
        xs.sort();
        for pair in xs.chunks(2) {
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

// ---- text ----

/// A bitmap font in the Amiga's layout: one wide strip of glyph bitmaps (`modulo` bytes per row)
/// with a table giving each glyph's bit offset and width, its advance and its kerning.
struct BitmapFont {
    height: i32,
    baseline: i32,
    lo: u8,
    hi: u8,
    modulo: usize,
    data: Vec<u8>,
    loc: Vec<(u16, u16)>,
    space: Vec<u16>,
    kern: Vec<i16>,
}

enum FontData {
    /// Fixed 8x8 stand-in for the Amiga's ROM font Topaz 8.
    Builtin,
    Bitmap(BitmapFont),
}

/// Reads `N` bytes at offset `o`, or None when they run past the end of the data.
fn be_bytes<const N: usize>(d: &[u8], o: usize) -> Option<[u8; N]> {
    d.get(o..o + N)?.try_into().ok()
}

fn be16(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_be_bytes(be_bytes(d, o)?))
}

fn be32(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_be_bytes(be_bytes(d, o)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_big_endian_values() {
        let data = [0x12, 0x34, 0x56, 0x78];
        assert_eq!(be16(&data, 0), Some(0x1234));
        assert_eq!(be32(&data, 0), Some(0x1234_5678));
    }

    #[test]
    fn reading_past_the_end_gives_none() {
        let data = [1, 2, 3];
        assert_eq!(be16(&data, 2), None);
        assert_eq!(be32(&data, 0), None);
    }
}

/// Reads an Amiga disk font (fonts/Amber/9): an executable file whose only hunk holds a
/// DiskFontHeader. Offsets inside it are relative to the start of that hunk's data.
fn load_disk_font(path: &std::path::Path) -> Option<BitmapFont> {
    let file = std::fs::read(path).ok()?;
    if be32(&file, 0)? != 0x3f3 {
        return None;
    }
    // HUNK_HEADER: 0, table size, first, last, one size per hunk, then HUNK_CODE, size, data
    let table = be32(&file, 8)? as usize;
    let hunk = 20 + 4 * table;
    if be32(&file, hunk)? != 0x3e9 {
        return None;
    }
    let seg = &file[hunk + 8..];
    let tf = 58; // TextFont inside the DiskFontHeader
    let height = be16(seg, tf + 20)? as i32;
    let baseline = be16(seg, tf + 26)? as i32;
    let lo = *seg.get(tf + 32)?;
    let hi = *seg.get(tf + 33)?;
    let chardata = be32(seg, tf + 34)? as usize;
    let modulo = be16(seg, tf + 38)? as usize;
    let charloc = be32(seg, tf + 40)? as usize;
    let charspace = be32(seg, tf + 44)? as usize;
    let charkern = be32(seg, tf + 48)? as usize;
    let count = hi as usize - lo as usize + 2; // plus the "default" glyph at the end
    let data = seg
        .get(chardata..chardata + modulo * height as usize)?
        .to_vec();
    let mut loc = Vec::new();
    let (mut space, mut kern) = (Vec::new(), Vec::new());
    for i in 0..count {
        loc.push((be16(seg, charloc + i * 4)?, be16(seg, charloc + i * 4 + 2)?));
        space.push(be16(seg, charspace + i * 2)?);
        kern.push(be16(seg, charkern + i * 2)? as i16);
    }
    Some(BitmapFont {
        height,
        baseline,
        lo,
        hi,
        modulo,
        data,
        loc,
        space,
        kern,
    })
}

fn new_font(data: FontData, width: i32, height: i32, baseline: i32) -> *mut TextFont {
    let data = Box::into_raw(Box::new(data));
    Box::into_raw(Box::new(TextFont {
        tf_YSize: height as UWORD,
        tf_XSize: width as UWORD,
        tf_Baseline: baseline as UWORD,
        tf_Flags: 0,
        priv_: data as APTR,
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn OpenFont(_attr: *mut TextAttr) -> *mut TextFont {
    new_font(FontData::Builtin, 8, 8, 6)
}

#[unsafe(no_mangle)]
pub extern "C" fn OpenDiskFont(_attr: *mut TextAttr) -> *mut TextFont {
    new_font(FontData::Builtin, 8, 8, 6)
}

#[unsafe(no_mangle)]
pub extern "C" fn CloseFont(_f: *mut TextFont) {}

#[unsafe(no_mangle)]
pub extern "C" fn SetFont(rp: *mut RastPort, f: *mut TextFont) {
    unsafe { (*rp).Font = f }
}

/// The game finds its story font with `font = (seg << 2) + 8` and `afont = &font->dfh_TF`, an
/// Amiga trick for turning a loaded segment into a pointer. We hand back a segment value that
/// makes that arithmetic land on a DiskFontHeader we allocated.
#[unsafe(no_mangle)]
pub extern "C" fn LoadSeg(name: *const c_char) -> BPTR {
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    let Some(path) = super::dos::resolve(&name, false) else {
        return 0;
    };
    let tf = match load_disk_font(&path) {
        Some(font) => {
            let (h, b) = (font.height, font.baseline);
            new_font(FontData::Bitmap(font), 10, h, b)
        }
        None => new_font(FontData::Builtin, 8, 9, 7),
    };
    let header = Box::into_raw(Box::new(DiskFontHeader {
        dfh_TF: unsafe { std::ptr::read(tf) },
    }));
    ((header as isize) - 8) >> 2
}

#[unsafe(no_mangle)]
pub extern "C" fn UnLoadSeg(_seg: BPTR) {}

/// Draws `len` characters at the pen position (the pen sits on the baseline).
#[unsafe(no_mangle)]
pub extern "C" fn Text(rp: *mut RastPort, s: *const u8, len: LONG) -> LONG {
    let rp = unsafe { &mut *rp };
    if rp.BitMap.is_null() || rp.Font.is_null() || len <= 0 {
        return 0;
    }
    let font = unsafe { &*((*rp.Font).priv_ as *const FontData) };
    let bm = unsafe { &*rp.BitMap };
    let text = unsafe { std::slice::from_raw_parts(s, len as usize) };
    let jam2 = rp.DrawMode & JAM2 != 0;
    for &ch in text {
        match font {
            FontData::Builtin => {
                let top = rp.cp_y as i32 - 6;
                let glyph = font8x8::legacy::BASIC_LEGACY[(ch & 0x7f) as usize];
                for row in 0..8 {
                    for col in 0..8 {
                        let on = glyph[row as usize] >> col & 1 != 0; // bit 0 is leftmost
                        let (x, y) = (rp.cp_x as i32 + col, top + row);
                        if on {
                            unsafe { plot(bm, x, y, rp.FgPen, false) };
                        } else if jam2 {
                            unsafe { plot(bm, x, y, rp.BgPen, false) };
                        }
                    }
                }
                rp.cp_x += 8;
            }
            FontData::Bitmap(f) => {
                let idx = if ch >= f.lo && ch <= f.hi {
                    (ch - f.lo) as usize
                } else {
                    (f.hi - f.lo) as usize + 1
                };
                let (bit_off, bit_w) = f.loc[idx];
                let advance = f.space[idx] as i32;
                let kern = f.kern[idx] as i32;
                let top = rp.cp_y as i32 - f.baseline;
                for row in 0..f.height {
                    if jam2 {
                        for x in 0..advance {
                            unsafe { plot(bm, rp.cp_x as i32 + x, top + row, rp.BgPen, false) };
                        }
                    }
                    for col in 0..bit_w as i32 {
                        let bit = bit_off as usize + col as usize;
                        let byte = f.data[row as usize * f.modulo + bit / 8];
                        if byte & (0x80 >> (bit % 8)) != 0 {
                            unsafe {
                                plot(bm, rp.cp_x as i32 + kern + col, top + row, rp.FgPen, false)
                            };
                        }
                    }
                }
                rp.cp_x += advance as WORD;
            }
        }
    }
    0
}

// ---- palette ----

#[unsafe(no_mangle)]
pub extern "C" fn GetColorMap(n: LONG) -> *mut ColorMap {
    unsafe {
        let cm = libc::calloc(1, std::mem::size_of::<ColorMap>()) as *mut ColorMap;
        (*cm).Count = n as UWORD;
        (*cm).ColorTable = libc::calloc(n.max(1) as usize, 2) as *mut UWORD;
        cm
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn FreeColorMap(cm: *mut ColorMap) {
    if !cm.is_null() {
        unsafe {
            libc::free((*cm).ColorTable as *mut c_void);
            libc::free(cm as *mut c_void);
        }
    }
}

/// Colours are 12 bit: 0x0RGB with 4 bits per channel.
#[unsafe(no_mangle)]
pub extern "C" fn SetRGB4(vp: *mut ViewPort, n: LONG, r: ULONG, g: ULONG, b: ULONG) {
    let cm = unsafe { (*vp).ColorMap };
    if cm.is_null() || n < 0 || n >= unsafe { (*cm).Count } as i32 {
        return;
    }
    unsafe {
        *(*cm).ColorTable.add(n as usize) = ((r & 15) << 8 | (g & 15) << 4 | (b & 15)) as UWORD
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn LoadRGB4(vp: *mut ViewPort, colors: *const UWORD, count: LONG) {
    let cm = unsafe { (*vp).ColorMap };
    if cm.is_null() {
        return;
    }
    let n = count.min(unsafe { (*cm).Count } as i32).max(0) as usize;
    unsafe { std::ptr::copy_nonoverlapping(colors, (*cm).ColorTable, n) };
}

#[unsafe(no_mangle)]
pub extern "C" fn GetRGB4(cm: *mut ColorMap, n: LONG) -> ULONG {
    if cm.is_null() || n < 0 || n >= unsafe { (*cm).Count } as i32 {
        return u32::MAX;
    }
    unsafe { *(*cm).ColorTable.add(n as usize) as ULONG }
}

// ---- views (what is on screen) ----

#[unsafe(no_mangle)]
pub extern "C" fn InitView(v: *mut View) {
    unsafe { std::ptr::write_bytes(v, 0, 1) }
}

#[unsafe(no_mangle)]
pub extern "C" fn InitVPort(vp: *mut ViewPort) {
    unsafe { std::ptr::write_bytes(vp, 0, 1) }
}

#[unsafe(no_mangle)]
pub extern "C" fn MakeVPort(_v: *mut View, _vp: *mut ViewPort) -> LONG {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn MrgCop(_v: *mut View) -> LONG {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn FreeVPortCopLists(_vp: *mut ViewPort) {}

#[unsafe(no_mangle)]
pub extern "C" fn FreeCprList(_c: *mut c_void) {}

/// Makes this view the one on screen. The game calls this to flip between its two pages, so
/// it is also where a new frame is shown.
#[unsafe(no_mangle)]
pub extern "C" fn LoadView(v: *mut View) {
    super::display::set_view(v);
    super::display::present();
}

#[unsafe(no_mangle)]
pub extern "C" fn WaitBOVP(_vp: *mut ViewPort) {
    super::display::wait_frame();
}

#[unsafe(no_mangle)]
pub extern "C" fn WaitTOF() {
    super::display::wait_frame();
}

// ---- blitter (no-ops, we are not racing hardware) ----

#[unsafe(no_mangle)]
pub extern "C" fn OwnBlitter() {}
#[unsafe(no_mangle)]
pub extern "C" fn DisownBlitter() {}
#[unsafe(no_mangle)]
pub extern "C" fn WaitBlit() {}

// ---- sprites (the mouse pointer; drawn by the display later) ----

#[unsafe(no_mangle)]
pub extern "C" fn GetSprite(_s: *mut SimpleSprite, _n: LONG) -> LONG {
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn FreeSprite(_n: LONG) {}
#[unsafe(no_mangle)]
pub extern "C" fn ChangeSprite(_vp: *mut ViewPort, _s: *mut SimpleSprite, data: *mut c_void) {
    super::display::set_sprite(data as *const u32);
}
#[unsafe(no_mangle)]
pub extern "C" fn MoveSprite(_vp: *mut ViewPort, _s: *mut SimpleSprite, _x: LONG, _y: LONG) {}
