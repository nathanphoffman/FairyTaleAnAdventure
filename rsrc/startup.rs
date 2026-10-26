//! Starting and stopping the game (from csrc/fmain.c): `open_all` sets up the screen, memory, fonts,
//! sound and input; `close_all` gives it all back; and `fta_main` (the original `main`) shows the
//! title and intro, starts the game and runs the main loop, one pass per frame.

use super::draw::{draw_sprites, repair_map_and_witch};
use super::loading::skipp;
use super::menus::print_options;
use super::types::IOAudio;
use crate::amiga::{
    BitMap, ColorMap, DiskFontHeader, HIRES, JAM2, MEMF_CHIP, Process, RastPort,
    SimpleSprite, TextAttr, TextFont, View, ViewPort,
};
use std::ffi::{c_char, c_void};

use super::asm_blit::SShape;
use super::ai::{actor_ai};
use super::asm_map::{map_draw, scrollmap};
use super::asm_stubs::{trapper};
use super::asm_text::{do_error, placard_text, ssp};
use super::combat::{melee_hits, move_missiles, queue_missiles};
use super::display::{add_device, pagechange, wrap_device};
use super::hdrive::{AllocDiskIO, FreeDiskIO, InvalidLastDiskIO, WaitLastDiskIO, load_track_range};
use super::iffsubs::{unpackbrush};
use super::input::{decode_mouse, getkey};
use super::keys::{handle_key, update_hero};
use super::loading::{copy_protect_junk, copypage, into_chip, load_next, read_sample, read_score, seekn, skipint};
use super::move_actor::{move_actor};
use super::objects::{do_objects};
use super::regions::{gen_mini};
use super::revive::{revive};
use super::screen_fx::{fade_normal, ppick, screen_size, stillscreen};
use super::world::{check_bed, check_door, day_night_tick, raft_proximity, sort_actors, spawn_encounters, speak_nearby, survival_tick, undo_sprites};
use super::asm_stubs::{WBenchMsg};
use super::game_tables::{_sprite_data, blackcolors, menus, new_wave, pointer, textcolors, topaz_ta};
use super::globals::{GfxBase as gfx_base, LayersBase as layers_base, afont, anim_list, anix, anix2, audio_open, audioport, bm_lim, bm_page1, bm_page2, bm_scroll, bm_source, bm_text, bmask_mem, cheat1, cmode, flasher, font, fp_drawing, fp_page1, fp_page2, fp_viewing, freeze_timer, handler_data, hero_x, hero_y, image_mem, img_x, img_y, ioaudio, li, light_timer, map_mem, map_x, map_y, new_region, nhinor, nhivar, oldview, openflags, origDir, pagea, pageb, queue_mem, quitflag, ri_page1, ri_page2, ri_text, rp, rp_map, rp_text, rp_text2, sample_mem, scoremem, sector_mem, seg, shadow_mem, shape_mem, sprite_data, terra_mem, tfont, thistask, track, v, viewstatus, volmem, vp, vp_page, vp_text, wavmem, wb_bmap, work_bm};


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
    fn InitRastPort(rp: *mut RastPort);
    fn SetBPen(rp: *mut RastPort, pen: u32);
    fn SetAPen(rp: *mut RastPort, pen: u32);
    fn SetDrMd(rp: *mut RastPort, mode: u32);
    fn SetRast(rp: *mut RastPort, pen: u32);
    fn SetFont(rp: *mut RastPort, f: *mut TextFont);
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
    fn ChangeSprite(vp: *mut ViewPort, s: *mut SimpleSprite, data: *mut u8);
    fn InitView(v: *mut View);
    fn InitVPort(vp: *mut ViewPort);
    fn MakeVPort(v: *mut View, vp: *mut ViewPort) -> i32;
    fn InitBitMap(bm: *mut BitMap, depth: i32, w: i32, h: i32);
    fn GetColorMap(n: i32) -> *mut ColorMap;
    fn FreeColorMap(cm: *mut ColorMap);
    fn FreeVPortCopLists(vp: *mut ViewPort);
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
    fn Delay(ticks: i32);
    fn SetRGB4(vp: *mut ViewPort, n: i32, r: u32, g: u32, b: u32);
    fn LoadRGB4(vp: *mut ViewPort, colors: *const u16, count: i32);
    fn BltBitMap(
        src: *mut BitMap,
        sx: i32,
        sy: i32,
        dst: *mut BitMap,
        dx: i32,
        dy: i32,
        w: i32,
        h: i32,
        minterm: u32,
        mask: u32,
        temp: *mut u8,
    ) -> i32;
    fn WaitBlit();
    fn exit(code: i32) -> !;
    // The game's own code that is still C, or lives in other files.
    fn init_music(ins: *const i16, wav: *mut u8, vol: *mut u8);
    fn wrap_music();
    fn stopscore();
}

#[repr(C)]
struct WBArg {
    wa_lock: isize,
}

#[repr(C)]
struct WBStartup {
    sm_arglist: *const WBArg,
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

unsafe fn set_flag(flag: u16) {
    unsafe { openflags |= flag };
}

unsafe fn has_flag(flag: u16) -> bool {
    unsafe { openflags & flag != 0 }
}

/// Opens everything the game needs. Returns 0, or the error number `do_error` shows.
pub fn open_all() -> i32 {
    unsafe {
        openflags = 0;

        gfx_base = OpenLibrary(c"graphics.library".as_ptr(), 0).cast();
        if gfx_base.is_null() {
            return 2;
        }
        layers_base = OpenLibrary(c"layers.library".as_ptr(), 0);
        if layers_base.is_null() {
            return 2;
        }
        set_flag(AL_GBASE);
        oldview = (*gfx_base).ActiView;

        if MakeBitMap(&raw mut work_bm, 2, 640, 200) == 0 {
            return 2;
        }

        li = NewLayerInfo();
        InitRastPort(&raw mut rp_map);
        InitRastPort(&raw mut rp_text);
        InitRastPort(&raw mut rp_text2);
        rp = &raw mut rp_text;

        wb_bmap = &raw mut work_bm;
        rp_text.BitMap = wb_bmap; // the "workbench screen" (not)
        SetBPen(rp, 0);
        SetDrMd(rp, JAM2 as u32);
        SetRast(rp, 0);

        bm_page1 = AllocMem(
            BITMAP_COUNT * size_of::<BitMap>() as u32,
            MEMF_CHIP | MEMF_CLEAR,
        )
        .cast();
        if bm_page1.is_null() {
            return 1;
        }
        set_flag(AL_BMAP);

        let err = AllocDiskIO();
        if err != 0 {
            return err;
        }

        seg = LoadSeg(c"fonts/Amber/9".as_ptr());
        if seg == 0 {
            return 15;
        }
        // The Amiga hands out a BPTR (address / 4); the font header sits 8 bytes in.
        font = ((seg << 2) + 8) as *mut DiskFontHeader;
        set_flag(AL_FONT);

        tfont = OpenFont(&raw mut topaz_ta);
        SetFont(&raw mut rp_text, tfont);
        SetFont(&raw mut rp_text2, tfont);
        SetFont(&raw mut rp_map, tfont);
        afont = &raw mut (*font).dfh_TF;

        // the input handler
        handler_data.xsprite = 320;
        handler_data.ysprite = 320;
        handler_data.gbase = gfx_base.cast();
        handler_data.pbase = std::ptr::null_mut();
        if add_device() == 0 {
            return 4;
        }

        thistask = FindTask(std::ptr::null());
        (*thistask).pr_WindowPtr = usize::MAX as *mut c_void; // (APTR)-1: no requesters
        (*thistask).pr_Task.tc_TrapCode = trapper as *mut c_void;

        set_flag(AL_HANDLE);
        FreeSprite(0);
        GetSprite(&raw mut pointer, 0);

        InitView(&raw mut v);
        InitVPort(&raw mut vp_page);
        InitVPort(&raw mut vp_text);

        v.ViewPort = &raw mut vp_text;
        vp_text.Next = &raw mut vp_page;
        vp_page.Next = std::ptr::null_mut();

        vp_page.DWidth = LOWRES_WIDTH;
        vp_text.DWidth = HIRES_WIDTH;

        vp_page.DxOffset = PAGE_VIEW_X;
        vp_page.DyOffset = 0;
        vp_text.DxOffset = 0;
        vp_page.DHeight = PAGE_VIEW_HEIGHT;

        vp_text.DyOffset = PAGE_HEIGHT;
        vp_text.DHeight = TEXT_HEIGHT as i16;

        vp_text.Modes = HIRES | SPRITES | VP_HIDE;

        // the bit maps (for the rasinfo and the rastports)
        bm_page2 = bm_page1.add(1);
        bm_text = bm_page1.add(2);
        bm_source = bm_page1.add(3);
        bm_lim = bm_page1.add(4);
        InitBitMap(bm_page1, PAGE_DEPTH, PHANTA_WIDTH, RAST_HEIGHT);
        InitBitMap(bm_page2, PAGE_DEPTH, PHANTA_WIDTH, RAST_HEIGHT);
        InitBitMap(bm_text, TEXT_DEPTH, 640, TEXT_HEIGHT);
        InitBitMap(bm_lim, 1, 320, 200);
        InitBitMap(&raw mut pagea, 5, 320, 200);
        InitBitMap(&raw mut pageb, 5, 320, 200);
        InitBitMap(&raw mut bm_scroll, 1, 640, TEXT_HEIGHT);
        InitBitMap(bm_source, 3, 64, 24);

        rp_text2.BitMap = bm_text;

        // the RasInfos
        ri_page1.BitMap = bm_page1;
        ri_page2.BitMap = bm_page2;
        ri_page1.RxOffset = 0;
        ri_page2.RxOffset = 0;
        ri_page1.RyOffset = 0;
        ri_page2.RyOffset = 0;
        ri_page1.Next = std::ptr::null_mut();
        ri_page2.Next = std::ptr::null_mut();

        ri_text.BitMap = bm_text;
        ri_text.RxOffset = 0;
        ri_text.RyOffset = 0;
        ri_text.Next = std::ptr::null_mut();

        fp_page1.savecop = std::ptr::null_mut();
        fp_page2.savecop = std::ptr::null_mut();
        fp_page1.ri_page = &raw mut ri_page1;
        fp_page2.ri_page = &raw mut ri_page1;
        fp_drawing = &raw mut fp_page1;
        fp_viewing = &raw mut fp_page2;

        // the colour tables
        vp_page.ColorMap = GetColorMap(32);
        vp_text.ColorMap = GetColorMap(20);

        // reset all plane pointers, then allocate the pages
        for i in 0..PAGE_DEPTH as usize {
            (*bm_page1).Planes[i] = std::ptr::null_mut();
            (*bm_page2).Planes[i] = std::ptr::null_mut();
        }
        for i in 0..PAGE_DEPTH as usize {
            (*bm_page1).Planes[i] = AllocRaster(PHANTA_WIDTH as u32, RAST_HEIGHT as u32);
            if (*bm_page1).Planes[i].is_null() {
                return 4;
            }
            (*bm_page2).Planes[i] = AllocRaster(PHANTA_WIDTH as u32, RAST_HEIGHT as u32);
            if (*bm_page2).Planes[i].is_null() {
                return 5;
            }
        }

        // The text bitmap borrows the "workbench" planes; the shape queue and the mask buffer sit
        // behind them.
        let text_plane_bytes = TEXT_HEIGHT as usize * TEXT_ROW_BYTES;
        (*bm_text).Planes[0] = (*wb_bmap).Planes[0];
        (*bm_text).Planes[1] = (*wb_bmap).Planes[1];
        (*bm_text).Planes[2] = (*bm_text).Planes[0].add(text_plane_bytes);
        (*bm_text).Planes[3] = (*bm_text).Planes[1].add(text_plane_bytes);

        queue_mem = (*bm_text).Planes[2].add(text_plane_bytes);
        bmask_mem = (*bm_text).Planes[3].add(text_plane_bytes);

        fp_page1.backsave = queue_mem.add(BACKSAVE_OFFSET);
        fp_page2.backsave = bmask_mem.add(BACKSAVE_OFFSET);

        fp_page1.shape_queue = queue_mem.cast();
        fp_page2.shape_queue = queue_mem.add(size_of::<SShape>() * MAXSHAPES).cast();

        vp_text.RasInfo = &raw mut ri_text;
        MakeVPort(&raw mut v, &raw mut vp_text);

        open_audio();

        wavmem = AllocMem(VOICE_SZ, MEMF_CHIP);
        if wavmem.is_null() {
            return 16;
        }
        scoremem = AllocMem(SCORE_SZ, 0);
        if scoremem.is_null() {
            return 17;
        }
        volmem = wavmem.add(S_WAVBUF as usize);
        init_music((&raw const new_wave).cast(), wavmem, volmem);
        set_flag(AL_MUSIC);

        image_mem = AllocMem(IMAGE_SZ, MEMF_CHIP);
        if image_mem.is_null() {
            return 6;
        }
        set_flag(AL_IMAGE);
        sector_mem = AllocMem(SECTOR_SZ, MEMF_CHIP);
        if sector_mem.is_null() {
            return 7;
        }
        map_mem = sector_mem.add(SECTOR_OFF);
        set_flag(AL_SECTOR);
        shape_mem = AllocMem(SHAPE_SZ, MEMF_CHIP);
        if shape_mem.is_null() {
            return 8;
        }
        set_flag(AL_SHAPE);
        shadow_mem = AllocMem(SHADOW_SZ, MEMF_CHIP);
        if shadow_mem.is_null() {
            return 10;
        }
        set_flag(AL_SHADOW);
        sample_mem = AllocMem(SAMPLE_SZ, MEMF_CHIP);
        if sample_mem.is_null() {
            return 11;
        }
        set_flag(AL_SAMPLE);
        terra_mem = AllocMem(TERRA_SZ, MEMF_CHIP);
        if terra_mem.is_null() {
            return 34;
        }
        set_flag(AL_TERR);

        // the waveforms and volume envelopes for the music
        let file = Open(c"v6".as_ptr(), MODE_OLDFILE);
        if file != 0 {
            Read(file, wavmem, S_WAVBUF);
            Seek(file, S_WAVBUF, 0);
            Read(file, volmem, S_VOLBUF);
            Close(file);
        }

        bm_scroll.Planes[0] = (*bm_text).Planes[0];

        sprite_data = into_chip((&raw mut _sprite_data).cast(), SPRITE_DATA_SZ);

        ChangeSprite(&raw mut vp_text, &raw mut pointer, sprite_data);
        handler_data.vbase = (&raw mut vp_text).cast();

        nhinor = into_chip((&raw mut hinor).cast(), NORM_HIGH_SZ);
        nhivar = into_chip((&raw mut hivar).cast(), NORM_HIGH_SZ);
        0
    }
}

/// Opens the audio device and resets it. If anything fails the game goes on without it.
unsafe fn open_audio() {
    unsafe {
        audioport = CreatePort(std::ptr::null(), 0);
        if audioport.is_null() {
            return;
        }
        ioaudio = CreateExtIO(audioport, size_of::<IOAudio>() as i32).cast();
        if ioaudio.is_null() {
            return;
        }
        let mut data: u8 = 0x0f;
        (*ioaudio).ioa_data = &mut data;
        (*ioaudio).ioa_length = 1;
        let request = &raw mut (*ioaudio).ioa_request;
        if OpenDevice(c"audio.device".as_ptr(), 0, request.cast(), 0) == 0 {
            (*request).io_command = CMD_RESET;
            (*request).io_flags = IOF_QUICK;
            BeginIO(request.cast());
            if (*request).io_flags & IOF_QUICK == 0 {
                WaitIO(request.cast());
            }
            audio_open = 1;
        }
    }
}

/// Gives back everything `open_all` took, then ends the program.
pub fn close_all() {
    unsafe {
        if has_flag(AL_TERR) {
            FreeMem(terra_mem.cast(), TERRA_SZ);
        }
        if has_flag(AL_SAMPLE) {
            FreeMem(sample_mem.cast(), SAMPLE_SZ);
        }
        if has_flag(AL_SHADOW) {
            FreeMem(shadow_mem.cast(), SHADOW_SZ);
        }
        if has_flag(AL_SHAPE) {
            FreeMem(shape_mem.cast(), SHAPE_SZ);
        }
        if has_flag(AL_SECTOR) {
            FreeMem(sector_mem.cast(), SECTOR_SZ);
        }
        if has_flag(AL_IMAGE) {
            FreeMem(image_mem.cast(), IMAGE_SZ);
        }
        if has_flag(AL_MUSIC) {
            wrap_music();
            FreeMem(wavmem.cast(), VOICE_SZ);
            FreeMem(scoremem.cast(), SCORE_SZ);
        }

        // Memory that `into_chip` copied is freed; if it handed back the original, nothing is.
        free_chip(
            sprite_data,
            (&raw mut _sprite_data).cast(),
            SPRITE_DATA_SZ as u32,
        );
        free_chip(nhinor, (&raw mut hinor).cast(), NORM_HIGH_SZ as u32);
        free_chip(nhivar, (&raw mut hivar).cast(), NORM_HIGH_SZ as u32);
        LoadView(oldview);
        FreeVPortCopLists(&raw mut vp_page);
        FreeVPortCopLists(&raw mut vp_text);
        FreeCprList(fp_page1.savecop);
        FreeCprList(fp_page2.savecop);
        FreeCprList(v.SHFCprList);

        if !vp_page.ColorMap.is_null() {
            FreeColorMap(vp_page.ColorMap);
        }
        if !vp_text.ColorMap.is_null() {
            FreeColorMap(vp_text.ColorMap);
        }
        for bm in [bm_page1, bm_page2] {
            for i in 0..PAGE_DEPTH as usize {
                if !(*bm).Planes[i].is_null() {
                    FreeRaster((*bm).Planes[i], PHANTA_WIDTH as u32, RAST_HEIGHT as u32);
                }
            }
        }

        if audio_open != 0 {
            CloseDevice((&raw mut (*ioaudio).ioa_request).cast());
        }
        if !ioaudio.is_null() {
            DeleteExtIO((&raw mut (*ioaudio).ioa_request).cast());
        }
        if !audioport.is_null() {
            DeletePort(audioport);
        }

        if has_flag(AL_HANDLE) {
            wrap_device();
        }
        if has_flag(AL_FONT) {
            UnLoadSeg(seg);
            CloseFont(tfont);
        }

        FreeDiskIO();

        if has_flag(AL_GBASE) {
            CloseLibrary(gfx_base.cast());
            CloseLibrary(layers_base);
        }
        if has_flag(AL_BMAP) {
            FreeMem(bm_page1.cast(), BITMAP_COUNT * size_of::<BitMap>() as u32);
            UnMakeBitMap(&raw mut work_bm);
        }
        DisposeLayerInfo(li);
        FreeSprite(0);
        openflags = 0;

        if origDir != 0 {
            CurrentDir(origDir);
        }

        exit(0);
    }
}

/// Frees `new` unless it is just the original `old`.
unsafe fn free_chip(new: *mut u8, old: *mut u8, size: u32) {
    unsafe {
        if new != old {
            FreeMem(new.cast(), size);
        }
    }
}

// ---- fta_main ----
unsafe extern "C" {
    static titletext: [u8; 0];
    fn playscore(t1: *const u8, t2: *const u8, t3: *const u8, t4: *const u8);
    fn fta_test_hook();
}

// Screen modes (`viewstatus`).
const VIEW_NORMAL: i8 = 0;
const VIEW_CHANGED: i8 = 3;
const VIEW_BIG: i8 = 1;
const VIEW_BIG_STILL: i8 = 4;
const VIEW_PAUSED: i8 = 2;
const VIEW_REDRAW_2: i8 = 98;
const VIEW_REDRAW: i8 = 99;
/// The game menu entry that pauses the game; bit 0 set means paused.
const GAME_MENU: usize = 4;
const PAUSE_ENTRY: usize = 5;
/// Intro: how big the picture zooms (half the width in pixels), and the step.
const ZOOM_FULL: i32 = 160;
const ZOOM_NORMAL: i32 = 156;
const ZOOM_STEP: usize = 4;
/// A region number below this means the next region is still being loaded.
const NO_REGION: u16 = 10;
/// Colours `LoadRGB4` sets for the page and the text.
const PAGE_COLORS: i32 = 32;
const TEXT_COLORS: i32 = 20;
/// Pens: the title text, and the colour of the flashing pointer.
const TITLE_PEN: u32 = 1;
const FLASH_COLOR: u32 = 15;
const SPRITE_PEN: u32 = 31;
/// Waits, in fiftieths of a second.
const TITLE_WAIT: i32 = 50;
const INTRO_PAGES_WAIT: i32 = 190;
const PLACARD_WAIT: i32 = 20;
const PAUSE_WAIT: i32 = 200;
/// The placard shown while the copy protection question is asked.
const PLACARD_COPYRIGHT: i32 = 19;
/// Where the picture planes in `image_mem` start: five planes, 8000 bytes apart.
const IMAGE_PLANE_BYTES: usize = 8000;
const PAGE_HALF_BYTES: usize = 40000;
/// Disk track range holding the shadow masks.
const SHADOW_BLOCK: (i16, i16) = (896, 24);
/// How `dif_x` wraps: a 10 bit signed value.
const DIF_SIGN_BIT: i32 = 0x200;
const DIF_SIGN_EXTEND: i32 = 0xfc00;
const DIF_MASK: i32 = 0x03ff;
/// The scroll directions `scrollmap` takes, by (dif_x, dif_y).
const SCROLL_UP_LEFT: i32 = 5;
const SCROLL_LEFT: i32 = 4;
const SCROLL_DOWN_LEFT: i32 = 3;
const SCROLL_UP: i32 = 6;
const SCROLL_DOWN: i32 = 2;
const SCROLL_UP_RIGHT: i32 = 7;
const SCROLL_RIGHT: i32 = 0;
const SCROLL_DOWN_RIGHT: i32 = 1;

/// The game: the original `main`. Called once from `src/main.rs` (`argc` 0 would mean "started from
/// Workbench").
#[unsafe(no_mangle)]
pub extern "C" fn fta_main(argc: i32, _argv: *mut c_char) {
    unsafe {
        if argc == 0 {
            origDir = CurrentDir((*(*WBenchMsg.cast::<WBStartup>()).sm_arglist).wa_lock);
        }

        light_timer = 0;
        let err = open_all();
        if err != 0 {
            do_error(err);
            quit_all();
            return;
        }

        if start_up() {
            main_loop();
            stopscore();
        }
        quit_all();
    }
}

/// The end of the program (the `quit_all` label): back to the plain text screen, then `close_all`.
unsafe fn quit_all() {
    unsafe {
        rp_text.BitMap = wb_bmap;
        SetRast(&raw mut rp_text, 0);
        close_all();
    }
}

/// Title, intro, loading and the copy-protection question, up to the first frame of the game.
/// Returns false if the player failed the copy protection.
unsafe fn start_up() -> bool {
    unsafe {
        stopscore();

        vp_page.RasInfo = &raw mut ri_page1;
        fp_page2.ri_page = &raw mut ri_page2;
        rp_map.BitMap = (*(*fp_viewing).ri_page).BitMap;
        SetRast(&raw mut rp_map, 0);
        rp_map.BitMap = (*(*fp_drawing).ri_page).BitMap;
        SetRast(&raw mut rp_map, 0);
        rp = &raw mut rp_map;
        screen_size(ZOOM_NORMAL);
        SetRGB4(&raw mut vp_page, 0, 0, 0, 6);
        SetRGB4(&raw mut vp_page, 1, 15, 15, 15);

        // show the legal text
        let pen = (*rp).FgPen as u32;
        SetAPen(rp, TITLE_PEN);
        ssp(titletext.as_ptr());
        SetAPen(rp, pen);

        Delay(TITLE_WAIT);

        rp = &raw mut rp_text;
        rp_text.BitMap = &raw mut bm_scroll;
        SetFont(rp, afont);
        SetAPen(rp, 10);
        SetBPen(rp, 11);

        read_score();
        read_sample();

        Delay(TITLE_WAIT);

        vp = &raw mut vp_page; // so that the IFF routines can communicate
        vp_page.RasInfo = &raw mut ri_page1;

        for i in 0..5 {
            pageb.Planes[i] = image_mem.add(i * IMAGE_PLANE_BYTES);
            pagea.Planes[i] = pageb.Planes[i].add(PAGE_HALF_BYTES);
        }
        (*bm_lim).Planes[0] = sector_mem;

        playscore(track[12], track[13], track[14], track[15]);

        LoadRGB4(&raw mut vp_text, (&raw const blackcolors).cast(), PAGE_COLORS);

        fp_page2.ri_page = &raw mut ri_page2;
        screen_size(0);

        pagechange(); // "prime the pump"
        pagechange();

        if skipint() == 0 {
            play_intro();
        }

        seekn();
        // go to the normal (playing) screen mode
        fp_page2.ri_page = &raw mut ri_page2;
        rp_map.BitMap = (*(*fp_viewing).ri_page).BitMap;
        SetRast(&raw mut rp_map, 0);
        rp_map.BitMap = (*(*fp_drawing).ri_page).BitMap;
        SetRast(&raw mut rp_map, 0);

        LoadRGB4(&raw mut vp_text, (&raw const blackcolors).cast(), PAGE_COLORS);
        screen_size(ZOOM_NORMAL);
        SetRGB4(&raw mut vp_page, 0, 0, 0, 3);
        load_track_range(SHADOW_BLOCK.0, SHADOW_BLOCK.1, shadow_mem.cast(), 0);
        WaitLastDiskIO();
        InvalidLastDiskIO();

        SetRGB4(&raw mut vp_page, 0, 0, 0, 6);
        unpackbrush(c"hiscreen".as_ptr(), bm_text, 0, 0);

        SetRGB4(&raw mut vp_page, 1, 15, 15, 15);

        rp = &raw mut rp_map;
        rp_map.BitMap = (*(*fp_drawing).ri_page).BitMap;
        stillscreen();
        SetAPen(rp, TITLE_PEN);
        placard_text(PLACARD_COPYRIGHT);
        handler_data.laydown = 0;
        handler_data.pickup = 0;
        if copy_protect_junk() == 0 {
            return false;
        }
        Delay(PLACARD_WAIT);

        ri_page1.RxOffset = 0;
        ri_page2.RxOffset = 0;
        ri_page1.RyOffset = 0;
        ri_page2.RyOffset = 0;

        stopscore();
        revive(1);
        rp_map.BitMap = (*(*fp_drawing).ri_page).BitMap;

        // establish the text page
        vp_text.DyOffset = PAGE_HEIGHT;
        vp_text.DHeight = TEXT_HEIGHT as i16;
        vp_page.DHeight = PAGE_VIEW_HEIGHT;
        vp_page.DxOffset = PAGE_VIEW_X;
        vp_page.DWidth = LOWRES_WIDTH;
        vp_page.DyOffset = 0;
        MakeVPort(&raw mut v, &raw mut vp_text);
        LoadRGB4(&raw mut vp_text, (&raw const textcolors).cast(), TEXT_COLORS);
        handler_data.pbase = (&raw mut pointer).cast();

        fp_page1.obcount = 0;
        fp_page2.obcount = 0;
        fp_page1.wflag = 0;
        fp_page2.wflag = 0;

        viewstatus = VIEW_REDRAW;

        cmode = 0;
        print_options();
        true
    }
}

/// The story pictures with the zoom in and out (skipped if the player presses Space).
unsafe fn play_intro() {
    unsafe {
        unpackbrush(c"page0".as_ptr(), &raw mut pageb, 0, 0);
        BltBitMap(
            &raw mut pageb,
            0,
            0,
            bm_page1,
            0,
            0,
            320,
            200,
            0xC0,
            0x1f,
            std::ptr::null_mut(),
        );
        BltBitMap(
            &raw mut pageb,
            0,
            0,
            bm_page2,
            0,
            0,
            320,
            200,
            0xC0,
            0x1f,
            std::ptr::null_mut(),
        );

        fp_page2.ri_page = &raw mut ri_page1;
        for x in (0..=ZOOM_FULL).step_by(ZOOM_STEP) {
            screen_size(x);
        }
        fp_page2.ri_page = &raw mut ri_page2;

        if skipint() == 0 {
            copypage(c"p1a".as_ptr(), c"p1b".as_ptr(), 21, 29);
            copypage(c"p2a".as_ptr(), c"p2b".as_ptr(), 20, 29);
            copypage(c"p3a".as_ptr(), c"p3b".as_ptr(), 20, 33);
            if skipp == 0 {
                Delay(INTRO_PAGES_WAIT);
            }
        }
        fp_page2.ri_page = &raw mut ri_page1;
        for x in (0..=ZOOM_NORMAL).rev().step_by(ZOOM_STEP) {
            screen_size(x);
        }
    }
}

/// How the map moved since the last frame, in sectors (x wraps at 10 bits).
unsafe fn scroll_step() -> (i16, i16) {
    unsafe {
        img_x = map_x >> 4;
        img_y = map_y >> 5;
        let mut dif_x = (img_x as i32 - (*fp_drawing).isv_x) as i16;
        let dif_y = (img_y as i32 - (*fp_drawing).isv_y) as i16;
        if dif_x as i32 & DIF_SIGN_BIT != 0 {
            dif_x = (dif_x as i32 | DIF_SIGN_EXTEND) as i16;
        } else {
            dif_x = (dif_x as i32 & DIF_MASK) as i16;
        }
        (dif_x, dif_y)
    }
}

/// The main loop: one pass per frame until the player quits.
unsafe fn main_loop() {
    unsafe {
        // `j` is the terrain of the last actor moved; it carries over to the next (see `move_actor`).
        let mut j: i16 = 0;
        let mut cycle: i16 = 0; // the frame counter (the C left it uninitialised)

        cheat1 = 0;
        quitflag = 0;
        while quitflag == 0 {
            cycle = cycle.wrapping_add(1);
            flasher = flasher.wrapping_add(1);
            fta_test_hook(); // test commands, does nothing normally

            let key = getkey() as i8;

            let notpause = menus[GAME_MENU].enabled[PAUSE_ENTRY] & 1 == 0;
            if key != 0 {
                handle_key(key as i32, notpause as i32);
            }

            if viewstatus == VIEW_PAUSED {
                Delay(PAUSE_WAIT);
                viewstatus = VIEW_REDRAW;
            } else if viewstatus == VIEW_BIG || viewstatus == VIEW_BIG_STILL {
                if flasher & 16 != 0 && viewstatus == VIEW_BIG {
                    SetRGB4(&raw mut vp_page, 31, FLASH_COLOR, FLASH_COLOR, FLASH_COLOR);
                } else {
                    SetRGB4(&raw mut vp_page, 31, 0, 0, 0);
                }
                ppick();
                continue; // go no further
            }

            decode_mouse();

            if menus[GAME_MENU].enabled[PAUSE_ENTRY] & 1 != 0 {
                Delay(1);
                continue;
            }

            update_hero(cycle);

            // the monsters' and the hero's movement
            raft_proximity();
            let mut i = 0;
            while i < anix as i32 {
                move_actor(i, cycle, &mut j);
                i += 1;
            }

            check_bed();
            check_door();

            hero_x = anim_list[0].abs_x;
            hero_y = anim_list[0].abs_y;

            // generate the map
            undo_sprites();

            let (mut dif_x, mut dif_y) = scroll_step();

            if new_region < NO_REGION {
                load_next();
            }

            let mut world_update = false;
            if viewstatus == VIEW_REDRAW
                || viewstatus == VIEW_REDRAW_2
                || viewstatus == VIEW_CHANGED
            {
                gen_mini();
                map_draw();
                dif_x = 0;
                dif_y = 0;
                if viewstatus == VIEW_REDRAW {
                    viewstatus = VIEW_REDRAW_2;
                } else if viewstatus == VIEW_REDRAW_2 {
                    viewstatus = VIEW_NORMAL;
                }
                world_update = true;
            } else {
                if dif_x != 0 || dif_y != 0 {
                    gen_mini();
                }
                let scroll = match (dif_x, dif_y) {
                    (1, 1) => Some(SCROLL_UP_LEFT),
                    (1, 0) => Some(SCROLL_LEFT),
                    (1, -1) => Some(SCROLL_DOWN_LEFT),
                    (0, 1) => Some(SCROLL_UP),
                    (0, -1) => Some(SCROLL_DOWN),
                    (-1, 1) => Some(SCROLL_UP_RIGHT),
                    (-1, 0) => Some(SCROLL_RIGHT),
                    (-1, -1) => Some(SCROLL_DOWN_RIGHT),
                    _ => None,
                };
                if let Some(dir) = scroll {
                    scrollmap(dir);
                } else if dif_x == 0 && dif_y == 0 {
                    // no motion: the world gets its turn here
                    ppick();
                    world_update = true;
                } else {
                    // moved too far to scroll: redraw the whole map
                    dif_x = 0;
                    dif_y = 0;
                    map_draw();
                }
            }

            if world_update {
                day_night_tick();
                if freeze_timer == 0 {
                    spawn_encounters();
                    speak_nearby();
                    actor_ai();
                    survival_tick();
                }
            }

            (*fp_drawing).isv_x = img_x as i32;
            (*fp_drawing).isv_y = img_y as i32;

            SetAPen(&raw mut rp_map, SPRITE_PEN);

            melee_hits();
            move_missiles();

            anix2 = anix;
            do_objects();

            // the flying missiles, then sort the actors by position
            queue_missiles();
            sort_actors();

            // now that the scroll blit is done, finish repairing the map
            repair_map_and_witch(dif_x, dif_y);

            draw_sprites(cycle);
            j = anix2; // the drawing loop used to leave j here; the next frame's actor loop reads it
            WaitBlit();
            (*(*fp_drawing).ri_page).RxOffset = (map_x & 15) as i16; // modulus 16
            (*(*fp_drawing).ri_page).RyOffset = (map_y & 31) as i16; // modulus 32
            pagechange();
            if viewstatus == VIEW_CHANGED {
                fade_normal();
                viewstatus = VIEW_NORMAL;
            }
        }
    }
}
