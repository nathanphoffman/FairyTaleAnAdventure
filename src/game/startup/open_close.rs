//! Opening and closing everything the game needs (from csrc/fmain.c): `open_all` sets up the screen,
//! memory, fonts, sound and input, and `close_all` gives it all back.

use crate::amiga::{
    BitMap, ColorMap, DiskFontHeader, HIRES, JAM2, MEMF_CHIP, Process, RastPort, SimpleSprite,
    TextAttr, TextFont, View, ViewPort,
};
use crate::game::data::types::IOAudio;
use std::ffi::{c_char, c_void};

use crate::game::data::globals::{
    GfxBase as gfx_base, LayersBase as layers_base, audio_is_open, audio_port, audio_request,
    font_header, font_segment, layer_info, main_view, open_flags, original_dir, panel_font,
    story_font, this_task,
};
use crate::game::data::state::{the_memory, the_screen, the_ui};
use crate::game::data::tables::{
    instrument_waves, pointer_sprite, pointer_sprite_data, topaz_attr,
};
use crate::game::files::disk::{close_disk, open_disk};
use crate::game::files::loading::into_chip;
use crate::game::render::blit::SShape;
use crate::game::render::display::{install_input_handler, remove_input_handler};
use crate::game::startup::amiga_leftovers::trapper;

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

// ---- fta_main ----
unsafe extern "C" {}

// Screen modes (`view_status`).
/// The game menu entry that pauses the game; bit 0 set means paused.
/// Intro: how big the picture zooms (half the width in pixels), and the step.
/// A region number below this means the next region is still being loaded.
/// Colours `LoadRGB4` sets for the page and the text.
/// Pens: the title text, and the colour of the flashing pointer.
/// Waits, in fiftieths of a second.
/// The placard shown while the copy protection question is asked.
/// Where the picture planes in `image_mem` start: five planes, 8000 bytes apart.
/// Disk track range holding the shadow masks.
/// How `dif_x` wraps: a 10 bit signed value.
unsafe fn set_flag(flag: u16) {
    unsafe { open_flags |= flag };
}

unsafe fn has_flag(flag: u16) -> bool {
    unsafe { open_flags & flag != 0 }
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
        open_flags = 0;
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

        layer_info = NewLayerInfo();
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
        font_segment = LoadSeg(c"fonts/Amber/9".as_ptr());
        if font_segment == 0 {
            return Err(15);
        }
        // The Amiga hands out a BPTR (address / 4); the font header sits 8 bytes in.
        font_header = ((font_segment << 2) + 8) as *mut DiskFontHeader;
        set_flag(AL_FONT);

        panel_font = OpenFont(&raw mut topaz_attr);
        SetFont(&raw mut the_screen.rp_text, panel_font);
        SetFont(&raw mut the_screen.rp_text2, panel_font);
        SetFont(&raw mut the_screen.rp_map, panel_font);
        story_font = &raw mut (*font_header).dfh_TF;
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

        this_task = FindTask(std::ptr::null());
        (*this_task).pr_WindowPtr = usize::MAX as *mut c_void; // (APTR)-1: no requesters
        (*this_task).pr_Task.tc_TrapCode = trapper as *mut c_void;

        set_flag(AL_HANDLE);
        FreeSprite(0);
        GetSprite(&raw mut pointer_sprite, 0);
        Ok(())
    }
}

/// Sets up the view and its two view ports: the text panel (hi-res) and the play field (lo-res).
unsafe fn set_up_view_ports() {
    unsafe {
        InitView(&raw mut main_view);
        InitVPort(&raw mut the_screen.vp_page);
        InitVPort(&raw mut the_screen.vp_text);

        main_view.ViewPort = &raw mut the_screen.vp_text;
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
        MakeVPort(&raw mut main_view, &raw mut the_screen.vp_text);
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
        audio_port = CreatePort(std::ptr::null(), 0);
        if audio_port.is_null() {
            return;
        }
        audio_request = CreateExtIO(audio_port, size_of::<IOAudio>() as i32).cast();
        if audio_request.is_null() {
            return;
        }
        let mut data: u8 = 0x0f;
        (*audio_request).ioa_data = &mut data;
        (*audio_request).ioa_length = 1;
        let request = &raw mut (*audio_request).ioa_request;
        if OpenDevice(c"audio.device".as_ptr(), 0, request.cast(), 0) == 0 {
            (*request).io_command = CMD_RESET;
            (*request).io_flags = IOF_QUICK;
            BeginIO(request.cast());
            if (*request).io_flags & IOF_QUICK == 0 {
                WaitIO(request.cast());
            }
            audio_is_open = 1;
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
        open_flags = 0;

        if original_dir != 0 {
            CurrentDir(original_dir);
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
        FreeCprList(main_view.SHFCprList);

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
        if audio_is_open != 0 {
            CloseDevice((&raw mut (*audio_request).ioa_request).cast());
        }
        if !audio_request.is_null() {
            DeleteExtIO((&raw mut (*audio_request).ioa_request).cast());
        }
        if !audio_port.is_null() {
            DeletePort(audio_port);
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
            UnLoadSeg(font_segment);
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
        DisposeLayerInfo(layer_info);
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
