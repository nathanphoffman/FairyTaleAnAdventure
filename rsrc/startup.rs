//! Starting and stopping the game (from csrc/fmain.c): `open_all` sets up the screen, memory, fonts,
//! sound and input; `close_all` gives it all back; and `fta_main` (the original `main`) shows the
//! title and intro, starts the game and runs the main loop, one pass per frame.

use super::draw::{draw_sprites, repair_map_and_witch};
use super::loading::intro_skip_requested;
use super::menus::print_options;
use super::types::IOAudio;
use crate::amiga::{
    BitMap, ColorMap, DiskFontHeader, HIRES, JAM2, MEMF_CHIP, Process, RastPort, SimpleSprite,
    TextAttr, TextFont, View, ViewPort,
};
use std::ffi::{c_char, c_void};

use super::ai::update_enemy_goals;
use super::asm_blit::SShape;
use super::asm_map::{draw_map, scroll_map};
use super::asm_stubs::WBenchMsg;
use super::asm_stubs::trapper;
use super::asm_text::{placard_text, print_positioned, show_error};
use super::combat::{advance_missiles, queue_missile_sprites, resolve_melee};
use super::display::{flip_pages, install_input_handler, remove_input_handler};
use super::game_tables::{
    black_colors, instrument_waves, menus, pointer_sprite, pointer_sprite_data, text_colors,
    topaz_attr,
};
use super::globals::{
    GfxBase as gfx_base, LayersBase as layers_base, audio_open, audioport, font, ioaudio, li,
    openflags, origDir, panel_font, seg, story_font, thistask, v,
};
use super::hdrive::{
    close_disk, load_track_range, mark_last_read_unused, open_disk, wait_for_last_read,
};
use super::iffsubs::load_picture;
use super::input::{next_key, read_walk_direction};
use super::keys::{handle_key, update_hero};
use super::loading::{
    copy_protection_check, copy_protection_riddle, into_chip, intro_skipped, load_pending_region,
    read_sample, read_score, show_story_page,
};
use super::move_actor::move_actor;
use super::objects::spawn_object_actors;
use super::regions::refresh_minimap;
use super::revive::revive;
use super::screen_fx::{fade_normal, handle_print_request, set_intro_zoom, show_still_screen};
use super::state::{the_actors, the_hero, the_memory, the_screen, the_ui, the_world};
use super::world::{
    check_bed, check_door, day_night_tick, raft_proximity, sort_actors, spawn_encounters,
    speak_nearby, survival_tick, undo_sprites,
};

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
    fn SetAPen(current_rastport: *mut RastPort, pen: u32);
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
    fn Delay(ticks: i32);
    fn SetRGB4(current_viewport: *mut ViewPort, n: i32, r: u32, g: u32, b: u32);
    fn LoadRGB4(current_viewport: *mut ViewPort, colors: *const u16, count: i32);
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

/// Opens everything the game needs. Returns 0, or the error number `show_error` shows.
pub fn open_all() -> i32 {
    match unsafe { open_everything() } {
        Ok(()) => 0,
        Err(error) => error,
    }
}

/// The start-up steps in order; the first one that fails gives the error number.
unsafe fn open_everything() -> Result<(), i32> {
    unsafe {
        openflags = 0;
        open_libraries_and_text_screen()?;
        allocate_bitmap_headers()?;
        match open_disk() {
            0 => {}
            error => return Err(error),
        }
        open_fonts()?;
        install_input()?;
        set_up_view_ports();
        set_up_bitmaps_and_rasinfos();
        allocate_page_planes()?;
        share_text_bitmap_memory();
        open_audio();
        allocate_music_memory()?;
        allocate_game_memory()?;
        load_music_waveforms();
        set_up_pointer_sprite_and_fonts_data();
        Ok(())
    }
}

/// Opens the graphics libraries and sets up the rast ports, with the text rast port as the current
/// one.
unsafe fn open_libraries_and_text_screen() -> Result<(), i32> {
    unsafe {
        gfx_base = OpenLibrary(c"graphics.library".as_ptr(), 0).cast();
        if gfx_base.is_null() {
            return Err(2);
        }
        layers_base = OpenLibrary(c"layers.library".as_ptr(), 0);
        if layers_base.is_null() {
            return Err(2);
        }
        set_flag(AL_GBASE);
        the_screen.old_view = (*gfx_base).ActiView;

        if MakeBitMap(&raw mut the_screen.work_bm, 2, 640, 200) == 0 {
            return Err(2);
        }

        li = NewLayerInfo();
        InitRastPort(&raw mut the_screen.rp_map);
        InitRastPort(&raw mut the_screen.rp_text);
        InitRastPort(&raw mut the_screen.rp_text2);
        the_screen.current_rastport = &raw mut the_screen.rp_text;

        the_screen.wb_bmap = &raw mut the_screen.work_bm;
        the_screen.rp_text.BitMap = the_screen.wb_bmap; // the "workbench screen" (not)
        SetBPen(the_screen.current_rastport, 0);
        SetDrMd(the_screen.current_rastport, JAM2 as u32);
        SetRast(the_screen.current_rastport, 0);
        Ok(())
    }
}

/// Allocates the five bitmap headers (two pages, text, source, limit) in one block.
unsafe fn allocate_bitmap_headers() -> Result<(), i32> {
    unsafe {
        the_screen.bm_page1 = AllocMem(
            BITMAP_COUNT * size_of::<BitMap>() as u32,
            MEMF_CHIP | MEMF_CLEAR,
        )
        .cast();
        if the_screen.bm_page1.is_null() {
            return Err(1);
        }
        set_flag(AL_BMAP);
        Ok(())
    }
}

/// Loads the story font and opens the panel font, and gives all the rast ports the panel font.
unsafe fn open_fonts() -> Result<(), i32> {
    unsafe {
        seg = LoadSeg(c"fonts/Amber/9".as_ptr());
        if seg == 0 {
            return Err(15);
        }
        // The Amiga hands out a BPTR (address / 4); the font header sits 8 bytes in.
        font = ((seg << 2) + 8) as *mut DiskFontHeader;
        set_flag(AL_FONT);

        panel_font = OpenFont(&raw mut topaz_attr);
        SetFont(&raw mut the_screen.rp_text, panel_font);
        SetFont(&raw mut the_screen.rp_text2, panel_font);
        SetFont(&raw mut the_screen.rp_map, panel_font);
        story_font = &raw mut (*font).dfh_TF;
        Ok(())
    }
}

/// Installs the keyboard and mouse handler, and takes over the pointer sprite.
unsafe fn install_input() -> Result<(), i32> {
    unsafe {
        the_ui.handler_data.xsprite = 320;
        the_ui.handler_data.ysprite = 320;
        the_ui.handler_data.gbase = gfx_base.cast();
        the_ui.handler_data.pbase = std::ptr::null_mut();
        if install_input_handler() == 0 {
            return Err(4);
        }

        thistask = FindTask(std::ptr::null());
        (*thistask).pr_WindowPtr = usize::MAX as *mut c_void; // (APTR)-1: no requesters
        (*thistask).pr_Task.tc_TrapCode = trapper as *mut c_void;

        set_flag(AL_HANDLE);
        FreeSprite(0);
        GetSprite(&raw mut pointer_sprite, 0);
        Ok(())
    }
}

/// Sets up the view and its two view ports: the text panel (hi-res) and the play field (lo-res).
unsafe fn set_up_view_ports() {
    unsafe {
        InitView(&raw mut v);
        InitVPort(&raw mut the_screen.vp_page);
        InitVPort(&raw mut the_screen.vp_text);

        v.ViewPort = &raw mut the_screen.vp_text;
        the_screen.vp_text.Next = &raw mut the_screen.vp_page;
        the_screen.vp_page.Next = std::ptr::null_mut();

        the_screen.vp_page.DWidth = LOWRES_WIDTH;
        the_screen.vp_text.DWidth = HIRES_WIDTH;

        the_screen.vp_page.DxOffset = PAGE_VIEW_X;
        the_screen.vp_page.DyOffset = 0;
        the_screen.vp_text.DxOffset = 0;
        the_screen.vp_page.DHeight = PAGE_VIEW_HEIGHT;

        the_screen.vp_text.DyOffset = PAGE_HEIGHT;
        the_screen.vp_text.DHeight = TEXT_HEIGHT as i16;

        the_screen.vp_text.Modes = HIRES | SPRITES | VP_HIDE;
    }
}

/// Splits the bitmap block into its five bitmaps, and sets up the rast infos and the two pages.
unsafe fn set_up_bitmaps_and_rasinfos() {
    unsafe {
        the_screen.bm_page2 = the_screen.bm_page1.add(1);
        the_screen.bm_text = the_screen.bm_page1.add(2);
        the_screen.bm_source = the_screen.bm_page1.add(3);
        the_screen.bm_lim = the_screen.bm_page1.add(4);
        InitBitMap(the_screen.bm_page1, PAGE_DEPTH, PHANTA_WIDTH, RAST_HEIGHT);
        InitBitMap(the_screen.bm_page2, PAGE_DEPTH, PHANTA_WIDTH, RAST_HEIGHT);
        InitBitMap(the_screen.bm_text, TEXT_DEPTH, 640, TEXT_HEIGHT);
        InitBitMap(the_screen.bm_lim, 1, 320, 200);
        InitBitMap(&raw mut the_screen.pagea, 5, 320, 200);
        InitBitMap(&raw mut the_screen.pageb, 5, 320, 200);
        InitBitMap(&raw mut the_screen.bm_scroll, 1, 640, TEXT_HEIGHT);
        InitBitMap(the_screen.bm_source, 3, 64, 24);

        the_screen.rp_text2.BitMap = the_screen.bm_text;

        the_screen.ri_page1.BitMap = the_screen.bm_page1;
        the_screen.ri_page2.BitMap = the_screen.bm_page2;
        the_screen.ri_page1.RxOffset = 0;
        the_screen.ri_page2.RxOffset = 0;
        the_screen.ri_page1.RyOffset = 0;
        the_screen.ri_page2.RyOffset = 0;
        the_screen.ri_page1.Next = std::ptr::null_mut();
        the_screen.ri_page2.Next = std::ptr::null_mut();

        the_screen.ri_text.BitMap = the_screen.bm_text;
        the_screen.ri_text.RxOffset = 0;
        the_screen.ri_text.RyOffset = 0;
        the_screen.ri_text.Next = std::ptr::null_mut();

        the_screen.fp_page1.savecop = std::ptr::null_mut();
        the_screen.fp_page2.savecop = std::ptr::null_mut();
        the_screen.fp_page1.ri_page = &raw mut the_screen.ri_page1;
        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page1;
        the_screen.drawing_page = &raw mut the_screen.fp_page1;
        the_screen.viewing_page = &raw mut the_screen.fp_page2;

        the_screen.vp_page.ColorMap = GetColorMap(32);
        the_screen.vp_text.ColorMap = GetColorMap(20);
    }
}

/// Allocates the five bit planes of each of the two pages.
unsafe fn allocate_page_planes() -> Result<(), i32> {
    unsafe {
        for i in 0..PAGE_DEPTH as usize {
            (*the_screen.bm_page1).Planes[i] = std::ptr::null_mut();
            (*the_screen.bm_page2).Planes[i] = std::ptr::null_mut();
        }
        for i in 0..PAGE_DEPTH as usize {
            (*the_screen.bm_page1).Planes[i] = AllocRaster(PHANTA_WIDTH as u32, RAST_HEIGHT as u32);
            if (*the_screen.bm_page1).Planes[i].is_null() {
                return Err(4);
            }
            (*the_screen.bm_page2).Planes[i] = AllocRaster(PHANTA_WIDTH as u32, RAST_HEIGHT as u32);
            if (*the_screen.bm_page2).Planes[i].is_null() {
                return Err(5);
            }
        }
        Ok(())
    }
}

/// The text bitmap borrows the "workbench" planes; the shape queue and the mask buffer sit behind
/// them.
unsafe fn share_text_bitmap_memory() {
    unsafe {
        let text_plane_bytes = TEXT_HEIGHT as usize * TEXT_ROW_BYTES;
        (*the_screen.bm_text).Planes[0] = (*the_screen.wb_bmap).Planes[0];
        (*the_screen.bm_text).Planes[1] = (*the_screen.wb_bmap).Planes[1];
        (*the_screen.bm_text).Planes[2] = (*the_screen.bm_text).Planes[0].add(text_plane_bytes);
        (*the_screen.bm_text).Planes[3] = (*the_screen.bm_text).Planes[1].add(text_plane_bytes);

        the_memory.queue_mem = (*the_screen.bm_text).Planes[2].add(text_plane_bytes);
        the_memory.bmask_mem = (*the_screen.bm_text).Planes[3].add(text_plane_bytes);

        the_screen.fp_page1.backsave = the_memory.queue_mem.add(BACKSAVE_OFFSET);
        the_screen.fp_page2.backsave = the_memory.bmask_mem.add(BACKSAVE_OFFSET);

        the_screen.fp_page1.shape_queue = the_memory.queue_mem.cast();
        the_screen.fp_page2.shape_queue = the_memory
            .queue_mem
            .add(size_of::<SShape>() * MAXSHAPES)
            .cast();

        the_screen.vp_text.RasInfo = &raw mut the_screen.ri_text;
        MakeVPort(&raw mut v, &raw mut the_screen.vp_text);
    }
}

/// Allocates the waveform/volume buffers and the score memory, and starts the music player.
unsafe fn allocate_music_memory() -> Result<(), i32> {
    unsafe {
        the_memory.wavmem = AllocMem(VOICE_SZ, MEMF_CHIP);
        if the_memory.wavmem.is_null() {
            return Err(16);
        }
        the_memory.scoremem = AllocMem(SCORE_SZ, 0);
        if the_memory.scoremem.is_null() {
            return Err(17);
        }
        the_memory.volmem = the_memory.wavmem.add(S_WAVBUF as usize);
        init_music(
            (&raw const instrument_waves).cast(),
            the_memory.wavmem,
            the_memory.volmem,
        );
        set_flag(AL_MUSIC);
        Ok(())
    }
}

/// Allocates one chip memory area; `error` is the number shown if there is no memory.
unsafe fn allocate_chip_area(size: u32, error: i32) -> Result<*mut u8, i32> {
    unsafe {
        let area = AllocMem(size, MEMF_CHIP);
        if area.is_null() { Err(error) } else { Ok(area) }
    }
}

/// Allocates the picture, sector, shape, shadow-mask, sample and terrain areas.
unsafe fn allocate_game_memory() -> Result<(), i32> {
    unsafe {
        the_memory.image_mem = allocate_chip_area(IMAGE_SZ, 6)?;
        set_flag(AL_IMAGE);
        the_memory.sector_mem = allocate_chip_area(SECTOR_SZ, 7)?;
        the_memory.map_mem = the_memory.sector_mem.add(SECTOR_OFF);
        set_flag(AL_SECTOR);
        the_memory.shape_mem = allocate_chip_area(SHAPE_SZ, 8)?;
        set_flag(AL_SHAPE);
        the_memory.shadow_mem = allocate_chip_area(SHADOW_SZ, 10)?;
        set_flag(AL_SHADOW);
        the_memory.sample_mem = allocate_chip_area(SAMPLE_SZ, 11)?;
        set_flag(AL_SAMPLE);
        the_memory.terrain_mem = allocate_chip_area(TERRA_SZ, 34)?;
        set_flag(AL_TERR);
        Ok(())
    }
}

/// Reads the waveforms and volume envelopes for the music from the file `v6`.
unsafe fn load_music_waveforms() {
    unsafe {
        let file = Open(c"v6".as_ptr(), MODE_OLDFILE);
        if file != 0 {
            Read(file, the_memory.wavmem, S_WAVBUF);
            Seek(file, S_WAVBUF, 0);
            Read(file, the_memory.volmem, S_VOLBUF);
            Close(file);
        }
    }
}

/// Last set-up: the scrolling text uses the first text plane, the pointer sprite is installed, and
/// the two high-bit font tables are copied to chip memory.
unsafe fn set_up_pointer_sprite_and_fonts_data() {
    unsafe {
        the_screen.bm_scroll.Planes[0] = (*the_screen.bm_text).Planes[0];

        the_memory.sprite_data = into_chip((&raw mut pointer_sprite_data).cast(), SPRITE_DATA_SZ);

        ChangeSprite(
            &raw mut the_screen.vp_text,
            &raw mut pointer_sprite,
            the_memory.sprite_data,
        );
        the_ui.handler_data.vbase = (&raw mut the_screen.vp_text).cast();

        the_memory.nhinor = into_chip((&raw mut hinor).cast(), NORM_HIGH_SZ);
        the_memory.nhivar = into_chip((&raw mut hivar).cast(), NORM_HIGH_SZ);
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
        free_game_memory();
        free_pointer_and_font_data();
        free_view_resources();
        free_page_planes();
        close_audio();
        close_input_and_fonts();
        close_disk();
        close_libraries_and_bitmaps();
        openflags = 0;

        if origDir != 0 {
            CurrentDir(origDir);
        }

        exit(0);
    }
}

/// Frees the areas `allocate_game_memory` and `allocate_music_memory` made (those that were).
unsafe fn free_game_memory() {
    unsafe {
        let areas = [
            (AL_TERR, the_memory.terrain_mem, TERRA_SZ),
            (AL_SAMPLE, the_memory.sample_mem, SAMPLE_SZ),
            (AL_SHADOW, the_memory.shadow_mem, SHADOW_SZ),
            (AL_SHAPE, the_memory.shape_mem, SHAPE_SZ),
            (AL_SECTOR, the_memory.sector_mem, SECTOR_SZ),
            (AL_IMAGE, the_memory.image_mem, IMAGE_SZ),
        ];
        for (flag, area, size) in areas {
            if has_flag(flag) {
                FreeMem(area.cast(), size);
            }
        }
        if has_flag(AL_MUSIC) {
            wrap_music();
            FreeMem(the_memory.wavmem.cast(), VOICE_SZ);
            FreeMem(the_memory.scoremem.cast(), SCORE_SZ);
        }
    }
}

/// Memory that `into_chip` copied is freed; if it handed back the original, nothing is.
unsafe fn free_pointer_and_font_data() {
    unsafe {
        free_chip(
            the_memory.sprite_data,
            (&raw mut pointer_sprite_data).cast(),
            SPRITE_DATA_SZ as u32,
        );
        free_chip(
            the_memory.nhinor,
            (&raw mut hinor).cast(),
            NORM_HIGH_SZ as u32,
        );
        free_chip(
            the_memory.nhivar,
            (&raw mut hivar).cast(),
            NORM_HIGH_SZ as u32,
        );
    }
}

/// Puts the old view back and frees the view ports' copper lists and colour maps.
unsafe fn free_view_resources() {
    unsafe {
        LoadView(the_screen.old_view);
        FreeVPortCopLists(&raw mut the_screen.vp_page);
        FreeVPortCopLists(&raw mut the_screen.vp_text);
        FreeCprList(the_screen.fp_page1.savecop);
        FreeCprList(the_screen.fp_page2.savecop);
        FreeCprList(v.SHFCprList);

        if !the_screen.vp_page.ColorMap.is_null() {
            FreeColorMap(the_screen.vp_page.ColorMap);
        }
        if !the_screen.vp_text.ColorMap.is_null() {
            FreeColorMap(the_screen.vp_text.ColorMap);
        }
    }
}

/// Frees the bit planes of both pages.
unsafe fn free_page_planes() {
    unsafe {
        for bm in [the_screen.bm_page1, the_screen.bm_page2] {
            for i in 0..PAGE_DEPTH as usize {
                if !(*bm).Planes[i].is_null() {
                    FreeRaster((*bm).Planes[i], PHANTA_WIDTH as u32, RAST_HEIGHT as u32);
                }
            }
        }
    }
}

/// Closes the audio device, its request and its port.
unsafe fn close_audio() {
    unsafe {
        if audio_open != 0 {
            CloseDevice((&raw mut (*ioaudio).ioa_request).cast());
        }
        if !ioaudio.is_null() {
            DeleteExtIO((&raw mut (*ioaudio).ioa_request).cast());
        }
        if !audioport.is_null() {
            DeletePort(audioport);
        }
    }
}

/// Takes the input handler down and closes the fonts.
unsafe fn close_input_and_fonts() {
    unsafe {
        if has_flag(AL_HANDLE) {
            remove_input_handler();
        }
        if has_flag(AL_FONT) {
            UnLoadSeg(seg);
            CloseFont(panel_font);
        }
    }
}

/// Closes the graphics libraries and frees the bitmap headers, the layer info and the sprite.
unsafe fn close_libraries_and_bitmaps() {
    unsafe {
        if has_flag(AL_GBASE) {
            CloseLibrary(gfx_base.cast());
            CloseLibrary(layers_base);
        }
        if has_flag(AL_BMAP) {
            FreeMem(
                the_screen.bm_page1.cast(),
                BITMAP_COUNT * size_of::<BitMap>() as u32,
            );
            UnMakeBitMap(&raw mut the_screen.work_bm);
        }
        DisposeLayerInfo(li);
        FreeSprite(0);
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
}

// Screen modes (`view_status`).
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
/// The scroll directions `scroll_map` takes, by (dif_x, dif_y).
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

        the_world.light_timer = 0;
        let err = open_all();
        if err != 0 {
            show_error(err);
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
        the_screen.rp_text.BitMap = the_screen.wb_bmap;
        SetRast(&raw mut the_screen.rp_text, 0);
        close_all();
    }
}

/// Title, intro, loading and the copy-protection question, up to the first frame of the game.
/// Returns false if the player failed the copy protection.
unsafe fn start_up() -> bool {
    unsafe {
        stopscore();
        show_title_screen();
        prepare_intro_pages_and_music();
        if let Some(dir) = std::env::var_os("FTA_MUSIC_EXPORT") {
            super::mood::export_tunes(std::path::Path::new(&dir));
            return false;
        }

        if intro_skipped() == 0 {
            play_intro();
        }

        copy_protection_check();
        show_play_screen_background();
        if copy_protection_riddle() == 0 {
            return false;
        }
        Delay(PLACARD_WAIT);
        start_first_brother();
        true
    }
}

/// Blanks both pages.
unsafe fn clear_both_pages() {
    unsafe {
        the_screen.rp_map.BitMap = (*(*the_screen.viewing_page).ri_page).BitMap;
        SetRast(&raw mut the_screen.rp_map, 0);
        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
        SetRast(&raw mut the_screen.rp_map, 0);
    }
}

/// The legal title text on a blue page, then the music and the sound effects are read in.
unsafe fn show_title_screen() {
    unsafe {
        the_screen.vp_page.RasInfo = &raw mut the_screen.ri_page1;
        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page2;
        clear_both_pages();
        the_screen.current_rastport = &raw mut the_screen.rp_map;
        set_intro_zoom(ZOOM_NORMAL);
        SetRGB4(&raw mut the_screen.vp_page, 0, 0, 0, 6);
        SetRGB4(&raw mut the_screen.vp_page, 1, 15, 15, 15);

        let pen = (*the_screen.current_rastport).FgPen as u32;
        SetAPen(the_screen.current_rastport, TITLE_PEN);
        print_positioned(titletext.as_ptr());
        SetAPen(the_screen.current_rastport, pen);

        Delay(TITLE_WAIT);

        the_screen.current_rastport = &raw mut the_screen.rp_text;
        the_screen.rp_text.BitMap = &raw mut the_screen.bm_scroll;
        SetFont(the_screen.current_rastport, story_font);
        SetAPen(the_screen.current_rastport, 10);
        SetBPen(the_screen.current_rastport, 11);

        read_score();
        read_sample();

        Delay(TITLE_WAIT);
    }
}

/// Points the picture loader at the page memory, starts the intro tune, and flips the pages twice
/// so both show the same thing.
unsafe fn prepare_intro_pages_and_music() {
    unsafe {
        the_screen.current_viewport = &raw mut the_screen.vp_page; // so that the IFF routines can communicate
        the_screen.vp_page.RasInfo = &raw mut the_screen.ri_page1;

        for i in 0..5 {
            the_screen.pageb.Planes[i] = the_memory.image_mem.add(i * IMAGE_PLANE_BYTES);
            the_screen.pagea.Planes[i] = the_screen.pageb.Planes[i].add(PAGE_HALF_BYTES);
        }
        (*the_screen.bm_lim).Planes[0] = the_memory.sector_mem;

        playscore(
            the_memory.track[12],
            the_memory.track[13],
            the_memory.track[14],
            the_memory.track[15],
        );

        LoadRGB4(
            &raw mut the_screen.vp_text,
            (&raw const black_colors).cast(),
            PAGE_COLORS,
        );

        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page2;
        set_intro_zoom(0);

        flip_pages(); // "prime the pump"
        flip_pages();
    }
}

/// Switches to the normal playing screen: loads the shadow masks and the panel picture, and shows
/// the copyright placard.
unsafe fn show_play_screen_background() {
    unsafe {
        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page2;
        clear_both_pages();

        LoadRGB4(
            &raw mut the_screen.vp_text,
            (&raw const black_colors).cast(),
            PAGE_COLORS,
        );
        set_intro_zoom(ZOOM_NORMAL);
        SetRGB4(&raw mut the_screen.vp_page, 0, 0, 0, 3);
        load_track_range(
            SHADOW_BLOCK.0,
            SHADOW_BLOCK.1,
            the_memory.shadow_mem.cast(),
            0,
        );
        wait_for_last_read();
        mark_last_read_unused();

        SetRGB4(&raw mut the_screen.vp_page, 0, 0, 0, 6);
        load_picture(c"hiscreen".as_ptr(), the_screen.bm_text, 0, 0);

        SetRGB4(&raw mut the_screen.vp_page, 1, 15, 15, 15);

        the_screen.current_rastport = &raw mut the_screen.rp_map;
        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
        show_still_screen();
        SetAPen(the_screen.current_rastport, TITLE_PEN);
        placard_text(PLACARD_COPYRIGHT);
        the_ui.handler_data.laydown = 0;
        the_ui.handler_data.pickup = 0;
    }
}

/// Starts the first brother and sets up the text page and the colours for play.
unsafe fn start_first_brother() {
    unsafe {
        the_screen.ri_page1.RxOffset = 0;
        the_screen.ri_page2.RxOffset = 0;
        the_screen.ri_page1.RyOffset = 0;
        the_screen.ri_page2.RyOffset = 0;

        stopscore();
        revive(1);
        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;

        the_screen.vp_text.DyOffset = PAGE_HEIGHT;
        the_screen.vp_text.DHeight = TEXT_HEIGHT as i16;
        the_screen.vp_page.DHeight = PAGE_VIEW_HEIGHT;
        the_screen.vp_page.DxOffset = PAGE_VIEW_X;
        the_screen.vp_page.DWidth = LOWRES_WIDTH;
        the_screen.vp_page.DyOffset = 0;
        MakeVPort(&raw mut v, &raw mut the_screen.vp_text);
        LoadRGB4(
            &raw mut the_screen.vp_text,
            (&raw const text_colors).cast(),
            TEXT_COLORS,
        );
        the_ui.handler_data.pbase = (&raw mut pointer_sprite).cast();

        the_screen.fp_page1.obcount = 0;
        the_screen.fp_page2.obcount = 0;
        the_screen.fp_page1.wflag = 0;
        the_screen.fp_page2.wflag = 0;

        the_screen.view_status = VIEW_REDRAW;

        the_ui.current_menu = 0;
        print_options();
    }
}

/// The story pictures with the zoom in and out (skipped if the player presses Space).
unsafe fn play_intro() {
    unsafe {
        load_picture(c"page0".as_ptr(), &raw mut the_screen.pageb, 0, 0);
        copy_title_picture_to(the_screen.bm_page1);
        copy_title_picture_to(the_screen.bm_page2);

        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page1;
        for x in (0..=ZOOM_FULL).step_by(ZOOM_STEP) {
            set_intro_zoom(x);
        }
        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page2;

        if intro_skipped() == 0 {
            show_story_page(c"p1a".as_ptr(), c"p1b".as_ptr(), 21, 29);
            show_story_page(c"p2a".as_ptr(), c"p2b".as_ptr(), 20, 29);
            show_story_page(c"p3a".as_ptr(), c"p3b".as_ptr(), 20, 33);
            if intro_skip_requested == 0 {
                Delay(INTRO_PAGES_WAIT);
            }
        }
        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page1;
        for x in (0..=ZOOM_NORMAL).rev().step_by(ZOOM_STEP) {
            set_intro_zoom(x);
        }
    }
}

/// Copies the whole title picture (loaded into `pageb`) onto the bitmap of a display page.
unsafe fn copy_title_picture_to(bitmap: *mut BitMap) {
    unsafe {
        BltBitMap(
            &raw mut the_screen.pageb,
            0,
            0,
            bitmap,
            0,
            0,
            320,
            200,
            0xC0,
            0x1f,
            std::ptr::null_mut(),
        );
    }
}

/// How the map moved since the last frame, in sectors (x wraps at 10 bits).
unsafe fn scroll_step() -> (i16, i16) {
    unsafe {
        the_world.tile_x = the_world.map_x >> 4;
        the_world.tile_y = the_world.map_y >> 5;
        let dif_x = (the_world.tile_x as i32 - (*the_screen.drawing_page).isv_x) as i16;
        let dif_y = (the_world.tile_y as i32 - (*the_screen.drawing_page).isv_y) as i16;
        (wrap_scroll_difference(dif_x), dif_y)
    }
}

/// The map's horizontal step as a 10 bit signed number (the map wraps round at the edges).
fn wrap_scroll_difference(difference: i16) -> i16 {
    if difference as i32 & DIF_SIGN_BIT != 0 {
        (difference as i32 | DIF_SIGN_EXTEND) as i16
    } else {
        (difference as i32 & DIF_MASK) as i16
    }
}

/// What the frame loop carries from one frame to the next.
struct FrameState {
    /// The frame counter (the C left it uninitialised).
    cycle: i16,
    /// The terrain of the last actor moved. It carries over to the next actor (a dead actor reads
    /// it without setting it, see `move_actor`), and from the drawing loop to the next frame.
    last_terrain: i16,
}

/// The main loop: one pass per frame until the player quits.
unsafe fn main_loop() {
    unsafe {
        let mut state = FrameState {
            cycle: 0,
            last_terrain: 0,
        };
        the_ui.cheat_enabled = 0;
        the_ui.quit_requested = 0;
        while the_ui.quit_requested == 0 {
            run_frame(&mut state);
        }
    }
}

/// One frame: input, moving everything, updating the map and the world, then drawing.
unsafe fn run_frame(state: &mut FrameState) {
    unsafe {
        state.cycle = state.cycle.wrapping_add(1);
        the_screen.flasher = the_screen.flasher.wrapping_add(1);
        super::statedump::run_pending_command(); // test commands, does nothing normally

        read_keyboard();
        if handle_view_status() {
            return;
        }

        read_walk_direction();
        if menus[GAME_MENU].enabled[PAUSE_ENTRY] & 1 != 0 {
            Delay(1);
            return;
        }

        update_hero(state.cycle);
        move_everything(state);
        check_bed();
        check_door();

        the_hero.x = the_actors.list[0].abs_x;
        the_hero.y = the_actors.list[0].abs_y;

        undo_sprites();
        let (scroll_x, scroll_y, world_update) = update_map_view();
        if world_update {
            run_world_update();
        }
        draw_frame(state, scroll_x, scroll_y);
    }
}

/// Reads one key and handles it.
unsafe fn read_keyboard() {
    unsafe {
        let key = next_key() as i8;
        let notpause = menus[GAME_MENU].enabled[PAUSE_ENTRY] & 1 == 0;
        if key != 0 {
            handle_key(key as i32, notpause as i32);
        }
    }
}

/// The screen modes that stop a normal frame. Returns true if this frame ends here (the item list
/// and the like only flash the pointer and handle the print queue).
unsafe fn handle_view_status() -> bool {
    unsafe {
        if the_screen.view_status == VIEW_PAUSED {
            Delay(PAUSE_WAIT);
            the_screen.view_status = VIEW_REDRAW;
        } else if the_screen.view_status == VIEW_BIG || the_screen.view_status == VIEW_BIG_STILL {
            if the_screen.flasher & 16 != 0 && the_screen.view_status == VIEW_BIG {
                SetRGB4(
                    &raw mut the_screen.vp_page,
                    31,
                    FLASH_COLOR,
                    FLASH_COLOR,
                    FLASH_COLOR,
                );
            } else {
                SetRGB4(&raw mut the_screen.vp_page, 31, 0, 0, 0);
            }
            handle_print_request();
            return true;
        }
        false
    }
}

/// Moves the hero, the raft and every monster and person (the actor count is read each time round,
/// as in the original).
unsafe fn move_everything(state: &mut FrameState) {
    unsafe {
        raft_proximity();
        let mut i = 0;
        while i < the_actors.active as i32 {
            move_actor(i, state.cycle, &mut state.last_terrain);
            i += 1;
        }
    }
}

/// The scroll direction `scroll_map` takes for the map moving one tile by (x, y), if it can scroll.
fn scroll_direction(scroll_x: i16, scroll_y: i16) -> Option<i32> {
    match (scroll_x, scroll_y) {
        (1, 1) => Some(SCROLL_UP_LEFT),
        (1, 0) => Some(SCROLL_LEFT),
        (1, -1) => Some(SCROLL_DOWN_LEFT),
        (0, 1) => Some(SCROLL_UP),
        (0, -1) => Some(SCROLL_DOWN),
        (-1, 1) => Some(SCROLL_UP_RIGHT),
        (-1, 0) => Some(SCROLL_RIGHT),
        (-1, -1) => Some(SCROLL_DOWN_RIGHT),
        _ => None,
    }
}

/// Brings the map in line with the camera: loads a waiting region, then redraws the whole map or
/// scrolls it by one tile. Returns the scroll step used and whether the world gets its turn this
/// frame (it does when the map is redrawn from scratch, or did not move at all).
unsafe fn update_map_view() -> (i16, i16, bool) {
    unsafe {
        let (mut scroll_x, mut scroll_y) = scroll_step();

        if the_world.new_region < NO_REGION {
            load_pending_region();
        }

        if the_screen.view_status == VIEW_REDRAW
            || the_screen.view_status == VIEW_REDRAW_2
            || the_screen.view_status == VIEW_CHANGED
        {
            refresh_minimap();
            draw_map();
            if the_screen.view_status == VIEW_REDRAW {
                the_screen.view_status = VIEW_REDRAW_2;
            } else if the_screen.view_status == VIEW_REDRAW_2 {
                the_screen.view_status = VIEW_NORMAL;
            }
            return (0, 0, true);
        }

        if scroll_x != 0 || scroll_y != 0 {
            refresh_minimap();
        }
        if let Some(direction) = scroll_direction(scroll_x, scroll_y) {
            scroll_map(direction);
            return (scroll_x, scroll_y, false);
        }
        if scroll_x == 0 && scroll_y == 0 {
            // no motion: the world gets its turn here
            handle_print_request();
            return (scroll_x, scroll_y, true);
        }
        // moved too far to scroll: redraw the whole map
        scroll_x = 0;
        scroll_y = 0;
        draw_map();
        (scroll_x, scroll_y, false)
    }
}

/// The once-per-frame world update: the clock, then (unless time is frozen) new monsters, talking,
/// what the monsters want to do, and hunger.
unsafe fn run_world_update() {
    unsafe {
        day_night_tick();
        if the_world.freeze_timer == 0 {
            spawn_encounters();
            speak_nearby();
            update_enemy_goals();
            survival_tick();
        }
    }
}

/// Fights, missiles and drawing, then shows the page that was just drawn.
unsafe fn draw_frame(state: &mut FrameState, scroll_x: i16, scroll_y: i16) {
    unsafe {
        (*the_screen.drawing_page).isv_x = the_world.tile_x as i32;
        (*the_screen.drawing_page).isv_y = the_world.tile_y as i32;

        SetAPen(&raw mut the_screen.rp_map, SPRITE_PEN);

        resolve_melee();
        advance_missiles();

        the_actors.drawn = the_actors.active;
        spawn_object_actors();

        // the flying missiles, then sort the actors by position
        queue_missile_sprites();
        sort_actors();

        // now that the scroll blit is done, finish repairing the map
        repair_map_and_witch(scroll_x, scroll_y);

        draw_sprites(state.cycle);
        // The drawing loop used to leave `j` here; the next frame's actor loop reads it.
        state.last_terrain = the_actors.drawn;
        WaitBlit();
        (*(*the_screen.drawing_page).ri_page).RxOffset = (the_world.map_x & 15) as i16; // modulus 16
        (*(*the_screen.drawing_page).ri_page).RyOffset = (the_world.map_y & 31) as i16; // modulus 32
        flip_pages();
        if the_screen.view_status == VIEW_CHANGED {
            fade_normal();
            the_screen.view_status = VIEW_NORMAL;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_scroll_steps_stay_as_they_are() {
        assert_eq!(wrap_scroll_difference(0), 0);
        assert_eq!(wrap_scroll_difference(1), 1);
        assert_eq!(wrap_scroll_difference(-1), -1);
    }

    #[test]
    fn scroll_steps_wrap_at_ten_bits() {
        // 1023 is one step the other way round the wrapped map; 1024 is no step at all
        assert_eq!(wrap_scroll_difference(0x3ff), -1);
        assert_eq!(wrap_scroll_difference(0x400), 0);
        assert_eq!(wrap_scroll_difference(0x201), -511);
    }

    #[test]
    fn the_map_scrolls_one_tile_in_each_direction() {
        assert_eq!(scroll_direction(1, 0), Some(SCROLL_LEFT));
        assert_eq!(scroll_direction(-1, 0), Some(SCROLL_RIGHT));
        assert_eq!(scroll_direction(0, 1), Some(SCROLL_UP));
        assert_eq!(scroll_direction(0, -1), Some(SCROLL_DOWN));
        assert_eq!(scroll_direction(1, 1), Some(SCROLL_UP_LEFT));
        assert_eq!(scroll_direction(-1, -1), Some(SCROLL_DOWN_RIGHT));
    }

    #[test]
    fn no_motion_and_big_jumps_do_not_scroll() {
        assert_eq!(scroll_direction(0, 0), None);
        assert_eq!(scroll_direction(2, 0), None);
        assert_eq!(scroll_direction(0, -3), None);
    }
}
