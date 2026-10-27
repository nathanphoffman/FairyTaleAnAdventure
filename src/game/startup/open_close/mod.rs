//! Opening and closing everything the game needs (from csrc/fmain.c): `open_all` sets up the screen,
//! memory, fonts, sound and input, and `close_all` gives it all back.

mod close;

mod open;

pub use self::close::close_all;
pub use self::open::open_all;

use crate::amiga::{
    BitMap, ColorMap, Process, RastPort, SimpleSprite, TextAttr, TextFont, View, ViewPort,
};

use std::ffi::{c_char, c_void};

use crate::game::data::globals::open_flags;

unsafe extern "C" {
    static mut hinor: [u8; 0];
    static mut hivar: [u8; 0];

    // AmigaOS calls the platform layer provides.
    fn OpenLibrary(name: *const c_char, version: u32) -> *mut c_void;

    fn CloseLibrary(lib: *mut c_void);

    fn MakeBitMap(bm: *mut BitMap, depth: i32, width: i32, height: i32) -> i32;

    fn UnMakeBitMap(bm: *mut BitMap);

    fn NewLayerInfo() -> *mut c_void;

    fn DisposeLayerInfo(li: *mut c_void);

    fn InitRastPort(current_rastport: *mut RastPort);

    fn SetBPen(current_rastport: *mut RastPort, pen: u32);

    fn SetDrMd(current_rastport: *mut RastPort, mode: u32);

    fn SetRast(current_rastport: *mut RastPort, pen: u32);

    fn SetFont(current_rastport: *mut RastPort, f: *mut TextFont);

    fn AllocMem(size: u32, flags: u32) -> *mut u8;

    fn FreeMem(mem: *mut c_void, size: u32);

    fn AllocRaster(w: u32, h: u32) -> *mut u8;

    fn FreeRaster(p: *mut u8, w: u32, h: u32);

    fn LoadSeg(name: *const c_char) -> isize;

    fn UnLoadSeg(seg: isize);

    fn OpenFont(attr: *mut TextAttr) -> *mut TextFont;

    fn CloseFont(f: *mut TextFont);

    fn FindTask(name: *const c_char) -> *mut Process;

    fn FreeSprite(n: i32);

    fn GetSprite(s: *mut SimpleSprite, n: i32) -> i32;

    fn ChangeSprite(current_viewport: *mut ViewPort, s: *mut SimpleSprite, data: *mut u8);

    fn InitView(v: *mut View);

    fn InitVPort(current_viewport: *mut ViewPort);

    fn MakeVPort(v: *mut View, current_viewport: *mut ViewPort) -> i32;

    fn InitBitMap(bm: *mut BitMap, depth: i32, w: i32, h: i32);

    fn GetColorMap(n: i32) -> *mut ColorMap;

    fn FreeColorMap(cm: *mut ColorMap);

    fn FreeVPortCopLists(current_viewport: *mut ViewPort);

    fn FreeCprList(c: *mut c_void);

    fn LoadView(view: *mut View);

    fn CreatePort(name: *const c_char, pri: i32) -> *mut c_void;

    fn DeletePort(p: *mut c_void);

    fn CreateExtIO(p: *mut c_void, size: i32) -> *mut c_void;

    fn DeleteExtIO(io: *mut c_void);

    fn OpenDevice(name: *const c_char, unit: u32, io: *mut c_void, flags: u32) -> i32;

    fn CloseDevice(io: *mut c_void);

    fn BeginIO(io: *mut c_void);

    fn WaitIO(io: *mut c_void) -> i32;

    fn Open(name: *const c_char, mode: i32) -> isize;

    fn Read(f: isize, buf: *mut u8, len: i32) -> i32;

    fn Seek(f: isize, pos: i32, mode: i32) -> i32;

    fn Close(f: isize);

    fn CurrentDir(lock: isize) -> isize;

    fn exit(code: i32) -> !;

    // The game's own code that is still C, or lives in other files.
    fn init_music(ins: *const i16, wav: *mut u8, vol: *mut u8);

    fn wrap_music();
}

// Memory the game allocates (csrc/fmain.c).
const PAGE_DEPTH: i32 = 5;
const TEXT_DEPTH: i32 = 4;
const PHANTA_WIDTH: i32 = 320; // two words added for horizontal scrolling
const PAGE_HEIGHT: i16 = 143;
const RAST_HEIGHT: i32 = 200;
const TEXT_HEIGHT: i32 = 57;
const MAXSHAPES: usize = 25;
const MEMF_CLEAR: u32 = 0x10000;
const SPRITES: u16 = 0x4000;
const VP_HIDE: u16 = 0x2000;
const S_WAVBUF: i32 = 128 * 8;
const S_VOLBUF: i32 = 10 * 256;
const VOICE_SZ: u32 = (S_WAVBUF + S_VOLBUF) as u32;
const SCORE_SZ: u32 = 5900;
const IPLAN_SZ: u32 = 16384; // one plane of 256 chars
const IMAGE_SZ: u32 = IPLAN_SZ * 5;
const SHAPE_SZ: u32 = 78000;
const SHADOW_SZ: u32 = 8192 + 4096; // background masks
const SECTOR_SZ: u32 = (128 * 256) + 4096; // 256 sectors, plus the region map
const SECTOR_OFF: usize = 128 * 256;
const SAMPLE_SZ: u32 = 5632;
const TERRA_SZ: u32 = 1024;
const SPRITE_DATA_SZ: i32 = 88;
const NORM_HIGH_SZ: i32 = 16 * 16;
const MODE_OLDFILE: i32 = 1005;
const BITMAP_COUNT: u32 = 5;
const LOWRES_WIDTH: i16 = 288;
const HIRES_WIDTH: i16 = 640;
const PAGE_VIEW_HEIGHT: i16 = 140;
const PAGE_VIEW_X: i16 = 16;
const CMD_RESET: u16 = 1;
const IOF_QUICK: u8 = 1;

/// A text row is 80 bytes (640 pixels, one bit per pixel).
const TEXT_ROW_BYTES: usize = 80;

/// Where the queue of shapes to draw and the first background save sit in the text bitmap's memory.
const BACKSAVE_OFFSET: usize = 962;

// What `open_all` has set up, so `close_all` knows what to undo (`openflags` bits).
const AL_BMAP: u16 = 0x0001;
const AL_GBASE: u16 = 0x0002;
const AL_HANDLE: u16 = 0x0004;
const AL_MUSIC: u16 = 0x0008;
const AL_IMAGE: u16 = 0x0010;
const AL_SECTOR: u16 = 0x0020;
const AL_SHAPE: u16 = 0x0080;
const AL_SHADOW: u16 = 0x0100;
const AL_FONT: u16 = 0x0200;
const AL_SAMPLE: u16 = 0x0400;
const AL_TERR: u16 = 0x4000;

// Screen modes (`view_status`).
/// How `dif_x` wraps: a 10 bit signed value.
unsafe fn set_flag(flag: u16) {
    unsafe { open_flags |= flag };
}

unsafe fn has_flag(flag: u16) -> bool {
    unsafe { open_flags & flag != 0 }
}
