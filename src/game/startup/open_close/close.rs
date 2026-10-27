//! Closing everything `open_all` set up, in the other order.

use super::{
    AL_BMAP, AL_FONT, AL_GBASE, AL_HANDLE, AL_IMAGE, AL_MUSIC, AL_SAMPLE, AL_SECTOR, AL_SHADOW,
    AL_SHAPE, AL_TERR, BITMAP_COUNT, CloseDevice, CloseFont, CloseLibrary, CurrentDir, DeleteExtIO,
    DeletePort, DisposeLayerInfo, FreeColorMap, FreeCprList, FreeMem, FreeRaster, FreeSprite,
    FreeVPortCopLists, IMAGE_SZ, LoadView, NORM_HIGH_SZ, PAGE_DEPTH, PHANTA_WIDTH, RAST_HEIGHT,
    SAMPLE_SZ, SCORE_SZ, SECTOR_SZ, SHADOW_SZ, SHAPE_SZ, SPRITE_DATA_SZ, TERRA_SZ, UnLoadSeg,
    UnMakeBitMap, VOICE_SZ, exit, has_flag, hinor, hivar, wrap_music,
};

use crate::amiga::BitMap;
use crate::game::data::globals::{
    GfxBase as gfx_base, LayersBase as layers_base, audio_is_open, audio_port, audio_request,
    font_segment, layer_info, main_view, open_flags, original_dir, panel_font,
};

use crate::game::data::state::{the_memory, the_screen};
use crate::game::data::tables::pointer_sprite_data;
use crate::game::files::disk::close_disk;
use crate::game::render::display::remove_input_handler;

/// Gives back everything `open_all` took, then ends the program.
pub fn close_all() {
    unsafe {
        free_game_memory();
        free_panel_data();
        free_view_resources();
        free_page_planes();
        close_audio();
        close_input_fonts();
        close_disk();
        close_graphics();
        open_flags = 0;

        if original_dir != 0 {
            CurrentDir(original_dir);
        }

        exit(0);
    }
}

/// Frees the areas `allocate_game_memory` and `allocate_music_memory` made (those that were).
pub(super) unsafe fn free_game_memory() {
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
pub(super) unsafe fn free_panel_data() {
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
pub(super) unsafe fn free_view_resources() {
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
pub(super) unsafe fn free_page_planes() {
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
pub(super) unsafe fn close_audio() {
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
pub(super) unsafe fn close_input_fonts() {
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
pub(super) unsafe fn close_graphics() {
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
pub(super) unsafe fn free_chip(new: *mut u8, old: *mut u8, size: u32) {
    unsafe {
        if new != old {
            FreeMem(new.cast(), size);
        }
    }
}
