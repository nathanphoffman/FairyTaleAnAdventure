//! Closing everything `open_all` set up, in the other order.

use super::{
    AL_BMAP, AL_FONT, AL_GBASE, AL_HANDLE, AL_IMAGE, AL_MUSIC, AL_SAMPLE, AL_SECTOR, AL_SHADOW,
    AL_SHAPE, AL_TERR, BITMAP_COUNT, IMAGE_SZ, NORM_HIGH_SZ, PAGE_DEPTH, PHANTA_WIDTH, RAST_HEIGHT,
    SAMPLE_SZ, SCORE_SZ, SECTOR_SZ, SHADOW_SZ, SHAPE_SZ, SPRITE_DATA_SZ, TERRA_SZ, VOICE_SZ,
    has_flag,
};
use crate::platform::audio::wrap_music;
use crate::platform::dos::set_current_dir;
use crate::platform::exec::{close_device, close_library, delete_ext_io, delete_port, free_mem};
use crate::platform::gfx::{
    close_font, dispose_layer_info, free_color_map, free_cpr_list, free_raster, free_sprite,
    free_vport_cop_lists, load_view, unload_seg, unmake_bit_map,
};

use crate::amiga::BitMap;
use crate::game::data::globals::{
    GfxBase as gfx_base, LayersBase as layers_base, audio_is_open, audio_port, audio_request,
    font_segment, layer_info, main_view, open_flags, original_dir, panel_font,
};

use crate::game::data::state::{the_memory, the_screen};
use crate::game::data::story_text::{HINOR, HIVAR};
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
            set_current_dir(original_dir);
        }

        std::process::exit(0);
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
                free_mem(area.cast(), size);
            }
        }

        if has_flag(AL_MUSIC) {
            wrap_music();
            free_mem(the_memory.wavmem.cast(), VOICE_SZ);
            free_mem(the_memory.scoremem.cast(), SCORE_SZ);
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
            HINOR.as_ptr() as *mut u8,
            NORM_HIGH_SZ as u32,
        );

        free_chip(
            the_memory.nhivar,
            HIVAR.as_ptr() as *mut u8,
            NORM_HIGH_SZ as u32,
        );
    }
}

/// Puts the old view back and frees the view ports' copper lists and colour maps.
pub(super) unsafe fn free_view_resources() {
    unsafe {
        load_view(the_screen.old_view);
        free_vport_cop_lists(&raw mut the_screen.vp_page);
        free_vport_cop_lists(&raw mut the_screen.vp_text);
        free_cpr_list(the_screen.fp_page1.savecop);
        free_cpr_list(the_screen.fp_page2.savecop);
        free_cpr_list(main_view.SHFCprList);

        if !the_screen.vp_page.ColorMap.is_null() {
            free_color_map(the_screen.vp_page.ColorMap);
        }

        if !the_screen.vp_text.ColorMap.is_null() {
            free_color_map(the_screen.vp_text.ColorMap);
        }
    }
}

/// Frees the bit planes of both pages.
pub(super) unsafe fn free_page_planes() {
    unsafe {
        for bm in [the_screen.bm_page1, the_screen.bm_page2] {
            for i in 0..PAGE_DEPTH as usize {
                if !(*bm).Planes[i].is_null() {
                    free_raster((*bm).Planes[i], PHANTA_WIDTH as u32, RAST_HEIGHT as u32);
                }
            }
        }
    }
}

/// Closes the audio device, its request and its port.
pub(super) unsafe fn close_audio() {
    unsafe {
        if audio_is_open != 0 {
            close_device((&raw mut (*audio_request).ioa_request).cast());
        }

        if !audio_request.is_null() {
            delete_ext_io((&raw mut (*audio_request).ioa_request).cast());
        }

        if !audio_port.is_null() {
            delete_port(audio_port);
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
            unload_seg(font_segment);
            close_font(panel_font);
        }
    }
}

/// Closes the graphics libraries and frees the bitmap headers, the layer info and the sprite.
pub(super) unsafe fn close_graphics() {
    unsafe {
        if has_flag(AL_GBASE) {
            close_library(gfx_base.cast());
            close_library(layers_base);
        }

        if has_flag(AL_BMAP) {
            free_mem(
                the_screen.bm_page1.cast(),
                BITMAP_COUNT * size_of::<BitMap>() as u32,
            );

            unmake_bit_map(&raw mut the_screen.work_bm);
        }

        dispose_layer_info(layer_info);
        free_sprite(0);
    }
}

/// Frees `new` unless it is just the original `old`.
pub(super) fn free_chip(new: *mut u8, old: *mut u8, size: u32) {
    if new != old {
        free_mem(new.cast(), size);
    }
}
