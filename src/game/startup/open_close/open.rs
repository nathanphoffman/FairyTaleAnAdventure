//! Opening everything the game needs: libraries, fonts, screen, memory, sound and input.

use super::{
    AL_BMAP, AL_FONT, AL_GBASE, AL_HANDLE, AL_IMAGE, AL_MUSIC, AL_SAMPLE, AL_SECTOR, AL_SHADOW,
    AL_SHAPE, AL_TERR, AllocMem, AllocRaster, BACKSAVE_OFFSET, BITMAP_COUNT, BeginIO, CMD_RESET,
    ChangeSprite, Close, CreateExtIO, CreatePort, FindTask, FreeSprite, GetColorMap, GetSprite,
    HIRES_WIDTH, IMAGE_SZ, IOF_QUICK, InitBitMap, InitRastPort, InitVPort, InitView, LOWRES_WIDTH,
    LoadSeg, MAXSHAPES, MEMF_CLEAR, MODE_OLDFILE, MakeBitMap, MakeVPort, NORM_HIGH_SZ,
    NewLayerInfo, Open, OpenDevice, OpenFont, OpenLibrary, PAGE_DEPTH, PAGE_HEIGHT,
    PAGE_VIEW_HEIGHT, PAGE_VIEW_X, PHANTA_WIDTH, RAST_HEIGHT, Read, S_VOLBUF, S_WAVBUF, SAMPLE_SZ,
    SCORE_SZ, SECTOR_OFF, SECTOR_SZ, SHADOW_SZ, SHAPE_SZ, SPRITE_DATA_SZ, SPRITES, Seek, SetBPen,
    SetDrMd, SetFont, SetRast, TERRA_SZ, TEXT_DEPTH, TEXT_HEIGHT, TEXT_ROW_BYTES, VOICE_SZ,
    VP_HIDE, WaitIO, hinor, hivar, init_music, set_flag,
};

use crate::amiga::{BitMap, DiskFontHeader, HIRES, JAM2, MEMF_CHIP};
use crate::game::data::globals::{
    GfxBase as gfx_base, LayersBase as layers_base, audio_is_open, audio_port, audio_request,
    font_header, font_segment, layer_info, main_view, open_flags, panel_font, story_font,
    this_task,
};

use crate::game::data::state::{the_memory, the_screen, the_ui};
use crate::game::data::tables::{
    instrument_waves, pointer_sprite, pointer_sprite_data, topaz_attr,
};

use crate::game::data::types::IOAudio;
use crate::game::files::disk::open_disk;
use crate::game::files::loading::into_chip;
use crate::game::render::blit::SShape;
use crate::game::render::display::install_input_handler;
use crate::game::startup::amiga_leftovers::trapper;
use std::ffi::c_void;

// The error numbers `show_error` shows when a start-up step fails.
const ERROR_NO_MEMORY_FOR_HEADERS: i32 = 1;
const ERROR_NO_GRAPHICS: i32 = 2;
const ERROR_NO_INPUT_HANDLER: i32 = 4;
const ERROR_NO_PAGE1_PLANES: i32 = 4;
const ERROR_NO_PAGE2_PLANES: i32 = 5;
const ERROR_NO_FONT: i32 = 15;
const ERROR_NO_WAVEFORM_MEMORY: i32 = 16;
const ERROR_NO_SCORE_MEMORY: i32 = 17;
const ERROR_NO_IMAGE_MEMORY: i32 = 6;
const ERROR_NO_SECTOR_MEMORY: i32 = 7;
const ERROR_NO_SHAPE_MEMORY: i32 = 8;
const ERROR_NO_SHADOW_MEMORY: i32 = 10;
const ERROR_NO_SAMPLE_MEMORY: i32 = 11;
const ERROR_NO_TERRAIN_MEMORY: i32 = 34;

// The "workbench" bitmap: the text panel's planes borrow its memory.
const WORKBENCH_DEPTH: i32 = 2;
const WORKBENCH_WIDTH: i32 = 640;
const WORKBENCH_HEIGHT: i32 = 200;

// Sizes passed to `InitBitMap` and the colour maps.
const TEXT_BITMAP_WIDTH: i32 = 640;
const LIMIT_BITMAP_DEPTH: i32 = 1;
const PLAY_WIDTH: i32 = 320;
const PLAY_HEIGHT: i32 = 200;
const PLAY_DEPTH: i32 = 5;
const SOURCE_BITMAP_DEPTH: i32 = 3;
const SOURCE_BITMAP_WIDTH: i32 = 64;
const SOURCE_BITMAP_HEIGHT: i32 = 24;
const PAGE_COLOR_COUNT: i32 = 32;
const TEXT_COLOR_COUNT: i32 = 20;

/// Where the pointer sprite starts out (off screen).
const POINTER_START: i16 = 320;

/// The Amiga hands out a BPTR (address / 4); the font header sits 8 bytes into the segment.
const BPTR_SHIFT: u32 = 2;
const FONT_HEADER_OFFSET: isize = 8;

/// The audio request asks for channels 0-3 (one bit each).
const ALL_AUDIO_CHANNELS: u8 = 0x0f;

/// Opens everything the game needs. Returns 0, or the error number `show_error` shows.
pub fn open_all() -> i32 {
    match unsafe { open_everything() } {
        Ok(()) => 0,
        Err(error) => error,
    }
}

/// The start-up steps in order; the first one that fails gives the error number.
pub(super) unsafe fn open_everything() -> Result<(), i32> {
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
pub(super) unsafe fn open_libraries_and_text_screen() -> Result<(), i32> {
    unsafe {
        gfx_base = OpenLibrary(c"graphics.library".as_ptr(), 0).cast();

        if gfx_base.is_null() {
            return Err(ERROR_NO_GRAPHICS);
        }

        layers_base = OpenLibrary(c"layers.library".as_ptr(), 0);

        if layers_base.is_null() {
            return Err(ERROR_NO_GRAPHICS);
        }

        set_flag(AL_GBASE);
        the_screen.old_view = (*gfx_base).ActiView;

        if MakeBitMap(
            &raw mut the_screen.work_bm,
            WORKBENCH_DEPTH,
            WORKBENCH_WIDTH,
            WORKBENCH_HEIGHT,
        ) == 0
        {
            return Err(ERROR_NO_GRAPHICS);
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
pub(super) unsafe fn allocate_bitmap_headers() -> Result<(), i32> {
    unsafe {
        the_screen.bm_page1 = AllocMem(
            BITMAP_COUNT * size_of::<BitMap>() as u32,
            MEMF_CHIP | MEMF_CLEAR,
        )
        .cast();

        if the_screen.bm_page1.is_null() {
            return Err(ERROR_NO_MEMORY_FOR_HEADERS);
        }

        set_flag(AL_BMAP);

        Ok(())
    }
}

/// Loads the story font and opens the panel font, and gives all the rast ports the panel font.
pub(super) unsafe fn open_fonts() -> Result<(), i32> {
    unsafe {
        font_segment = LoadSeg(c"fonts/Amber/9".as_ptr());

        if font_segment == 0 {
            return Err(ERROR_NO_FONT);
        }

        font_header = ((font_segment << BPTR_SHIFT) + FONT_HEADER_OFFSET) as *mut DiskFontHeader;
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
pub(super) unsafe fn install_input() -> Result<(), i32> {
    unsafe {
        the_ui.handler_data.xsprite = POINTER_START;
        the_ui.handler_data.ysprite = POINTER_START;
        the_ui.handler_data.gbase = gfx_base.cast();
        the_ui.handler_data.pbase = std::ptr::null_mut();

        if install_input_handler() == 0 {
            return Err(ERROR_NO_INPUT_HANDLER);
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
pub(super) unsafe fn set_up_view_ports() {
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
pub(super) unsafe fn set_up_bitmaps_and_rasinfos() {
    unsafe {
        split_bitmap_block();
        init_bitmaps();
        set_up_rast_infos();
        set_up_pages();

        the_screen.vp_page.ColorMap = GetColorMap(PAGE_COLOR_COUNT);
        the_screen.vp_text.ColorMap = GetColorMap(TEXT_COLOR_COUNT);
    }
}

/// The five bitmap headers sit one after the other in the block `bm_page1` points to.
unsafe fn split_bitmap_block() {
    unsafe {
        the_screen.bm_page2 = the_screen.bm_page1.add(1);
        the_screen.bm_text = the_screen.bm_page1.add(2);
        the_screen.bm_source = the_screen.bm_page1.add(3);
        the_screen.bm_lim = the_screen.bm_page1.add(4);
    }
}

/// Gives every bitmap its depth and size (the planes are filled in later).
unsafe fn init_bitmaps() {
    unsafe {
        InitBitMap(the_screen.bm_page1, PAGE_DEPTH, PHANTA_WIDTH, RAST_HEIGHT);
        InitBitMap(the_screen.bm_page2, PAGE_DEPTH, PHANTA_WIDTH, RAST_HEIGHT);
        InitBitMap(
            the_screen.bm_text,
            TEXT_DEPTH,
            TEXT_BITMAP_WIDTH,
            TEXT_HEIGHT,
        );
        InitBitMap(
            the_screen.bm_lim,
            LIMIT_BITMAP_DEPTH,
            PLAY_WIDTH,
            PLAY_HEIGHT,
        );
        InitBitMap(
            &raw mut the_screen.pagea,
            PLAY_DEPTH,
            PLAY_WIDTH,
            PLAY_HEIGHT,
        );
        InitBitMap(
            &raw mut the_screen.pageb,
            PLAY_DEPTH,
            PLAY_WIDTH,
            PLAY_HEIGHT,
        );
        InitBitMap(
            &raw mut the_screen.bm_scroll,
            LIMIT_BITMAP_DEPTH,
            TEXT_BITMAP_WIDTH,
            TEXT_HEIGHT,
        );
        InitBitMap(
            the_screen.bm_source,
            SOURCE_BITMAP_DEPTH,
            SOURCE_BITMAP_WIDTH,
            SOURCE_BITMAP_HEIGHT,
        );
    }
}

/// Points each rast info at its bitmap, unscrolled and with nothing after it.
unsafe fn set_up_rast_infos() {
    unsafe {
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
    }
}

/// The two pages the game draws on and shows in turn (both start on the first rast info).
unsafe fn set_up_pages() {
    unsafe {
        the_screen.fp_page1.savecop = std::ptr::null_mut();
        the_screen.fp_page2.savecop = std::ptr::null_mut();
        the_screen.fp_page1.ri_page = &raw mut the_screen.ri_page1;
        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page1;
        the_screen.drawing_page = &raw mut the_screen.fp_page1;
        the_screen.viewing_page = &raw mut the_screen.fp_page2;
    }
}

/// Allocates the five bit planes of each of the two pages.
pub(super) unsafe fn allocate_page_planes() -> Result<(), i32> {
    unsafe {
        for i in 0..PAGE_DEPTH as usize {
            (*the_screen.bm_page1).Planes[i] = std::ptr::null_mut();
            (*the_screen.bm_page2).Planes[i] = std::ptr::null_mut();
        }

        for i in 0..PAGE_DEPTH as usize {
            (*the_screen.bm_page1).Planes[i] = AllocRaster(PHANTA_WIDTH as u32, RAST_HEIGHT as u32);

            if (*the_screen.bm_page1).Planes[i].is_null() {
                return Err(ERROR_NO_PAGE1_PLANES);
            }

            (*the_screen.bm_page2).Planes[i] = AllocRaster(PHANTA_WIDTH as u32, RAST_HEIGHT as u32);

            if (*the_screen.bm_page2).Planes[i].is_null() {
                return Err(ERROR_NO_PAGE2_PLANES);
            }
        }

        Ok(())
    }
}

/// The text bitmap borrows the "workbench" planes; the shape queue and the mask buffer sit behind
/// them.
pub(super) unsafe fn share_text_bitmap_memory() {
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
pub(super) unsafe fn allocate_music_memory() -> Result<(), i32> {
    unsafe {
        the_memory.wavmem = AllocMem(VOICE_SZ, MEMF_CHIP);

        if the_memory.wavmem.is_null() {
            return Err(ERROR_NO_WAVEFORM_MEMORY);
        }

        the_memory.scoremem = AllocMem(SCORE_SZ, 0);

        if the_memory.scoremem.is_null() {
            return Err(ERROR_NO_SCORE_MEMORY);
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
pub(super) unsafe fn allocate_chip_area(size: u32, error: i32) -> Result<*mut u8, i32> {
    unsafe {
        let area = AllocMem(size, MEMF_CHIP);

        if area.is_null() { Err(error) } else { Ok(area) }
    }
}

/// Allocates the picture, sector, shape, shadow-mask, sample and terrain areas.
pub(super) unsafe fn allocate_game_memory() -> Result<(), i32> {
    unsafe {
        the_memory.image_mem = allocate_chip_area(IMAGE_SZ, ERROR_NO_IMAGE_MEMORY)?;
        set_flag(AL_IMAGE);
        the_memory.sector_mem = allocate_chip_area(SECTOR_SZ, ERROR_NO_SECTOR_MEMORY)?;
        the_memory.map_mem = the_memory.sector_mem.add(SECTOR_OFF);
        set_flag(AL_SECTOR);
        the_memory.shape_mem = allocate_chip_area(SHAPE_SZ, ERROR_NO_SHAPE_MEMORY)?;
        set_flag(AL_SHAPE);
        the_memory.shadow_mem = allocate_chip_area(SHADOW_SZ, ERROR_NO_SHADOW_MEMORY)?;
        set_flag(AL_SHADOW);
        the_memory.sample_mem = allocate_chip_area(SAMPLE_SZ, ERROR_NO_SAMPLE_MEMORY)?;
        set_flag(AL_SAMPLE);
        the_memory.terrain_mem = allocate_chip_area(TERRA_SZ, ERROR_NO_TERRAIN_MEMORY)?;
        set_flag(AL_TERR);

        Ok(())
    }
}

/// Reads the waveforms and volume envelopes for the music from the file `v6`.
pub(super) unsafe fn load_music_waveforms() {
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
pub(super) unsafe fn set_up_pointer_sprite_and_fonts_data() {
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
pub(super) unsafe fn open_audio() {
    unsafe {
        audio_port = CreatePort(std::ptr::null(), 0);

        if audio_port.is_null() {
            return;
        }

        audio_request = CreateExtIO(audio_port, size_of::<IOAudio>() as i32).cast();

        if audio_request.is_null() {
            return;
        }

        let mut data: u8 = ALL_AUDIO_CHANNELS;
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
