//! Opening everything the game needs: the screen, fonts, memory, music and the disk image.

use super::{
    BACKSAVE_OFFSET, IMAGE_SZ, MAXSHAPES, S_VOLBUF, S_WAVBUF, SAMPLE_SZ, SCORE_SZ, SECTOR_OFF,
    SECTOR_SZ, SHADOW_SZ, SHAPE_SZ, TERRA_SZ, VOICE_SZ,
};
use crate::game::data::state::{the_memory, the_screen};
use crate::game::data::story_text::{HINOR, HIVAR};
use crate::game::data::tables::{instrument_waves, pointer_sprite_data};
use crate::game::files::disk::open_disk;
use crate::game::render::blit::SShape;
use crate::game::render::screen::FontKind;
use crate::gfx::Font;
use crate::platform::audio::init_music;
use crate::platform::dos::resolve;

// The error numbers `show_error` shows when a start-up step fails.
const ERROR_NO_FONT: i32 = 15;

/// The story font's file.
const STORY_FONT_FILE: &str = "fonts/Amber/9";

/// The file with the music's waveforms and volume envelopes.
const WAVEFORM_FILE: &str = "v6";

/// Memory areas are given a little slack at the end: the sprite blitter reads a word past some of
/// them.
const AREA_SLACK: usize = 128;

/// Opens everything the game needs. Returns 0, or the error number `show_error` shows.
pub fn open_all() -> i32 {
    match open_everything() {
        Ok(()) => 0,
        Err(error) => error,
    }
}

/// The start-up steps in order; the first one that fails gives the error number.
fn open_everything() -> Result<(), i32> {
    unsafe {
        (*(&raw mut the_screen)).set_up([&HINOR, &HIVAR]);
    }

    match open_disk() {
        0 => {}
        error => return Err(error),
    }

    open_fonts()?;
    unsafe {
        set_up_blitter_memory();
        allocate_music_memory();
        allocate_game_memory();
    }

    load_music_waveforms();
    unsafe { (*(&raw mut the_screen)).pointer = Some(&pointer_sprite_data) };

    Ok(())
}

/// Loads the story font; the panel uses the built-in one.
fn open_fonts() -> Result<(), i32> {
    let story = resolve(STORY_FONT_FILE, false).ok_or(ERROR_NO_FONT)?;
    let story = Font::load(&story).unwrap_or(Font::Builtin);

    unsafe {
        let screen = &mut *(&raw mut the_screen);
        screen.set_font(FontKind::Panel, Font::Builtin);
        screen.set_font(FontKind::Story, story);
    }

    Ok(())
}

/// A memory area for the whole run of the game (zeroed, with a little slack).
fn static_area(size: usize) -> *mut u8 {
    Box::leak(vec![0u8; size + AREA_SLACK].into_boxed_slice()).as_mut_ptr()
}

/// Points the shape queues and the saved backgrounds of the two pages into the blitter's work memory.
unsafe fn set_up_blitter_memory() {
    unsafe {
        let screen = &mut *(&raw mut the_screen);
        let queue = screen.queue_region.as_mut_ptr();
        let mask = screen.mask_region.as_mut_ptr();
        the_memory.queue_mem = queue;
        the_memory.bmask_mem = mask;

        screen.pages[0].backsave = queue.add(BACKSAVE_OFFSET);
        screen.pages[1].backsave = mask.add(BACKSAVE_OFFSET);
        screen.pages[0].shape_queue = queue.cast();
        screen.pages[1].shape_queue = queue.add(size_of::<SShape>() * MAXSHAPES).cast();
    }
}

/// Makes the waveform/volume buffers and the score memory, and gives the music player the
/// instruments.
unsafe fn allocate_music_memory() {
    unsafe {
        the_memory.wavmem = static_area(VOICE_SZ);
        the_memory.scoremem = static_area(SCORE_SZ);
        the_memory.volmem = the_memory.wavmem.add(S_WAVBUF);
        init_music(
            (&raw const instrument_waves).cast(),
            the_memory.wavmem,
            the_memory.volmem,
        );
    }
}

/// Makes the picture, sector, shape, shadow-mask, sample and terrain areas.
unsafe fn allocate_game_memory() {
    unsafe {
        the_memory.image_mem = static_area(IMAGE_SZ);
        the_memory.sector_mem = static_area(SECTOR_SZ);
        the_memory.map_mem = the_memory.sector_mem.add(SECTOR_OFF);
        the_memory.shape_mem = static_area(SHAPE_SZ);
        the_memory.shadow_mem = static_area(SHADOW_SZ);
        the_memory.sample_mem = static_area(SAMPLE_SZ);
        the_memory.terrain_mem = static_area(TERRA_SZ);
    }
}

/// Reads the waveforms and volume envelopes for the music from the file `v6`: the waveforms, then
/// as many bytes that are skipped, then the envelopes.
fn load_music_waveforms() {
    let Some(data) = resolve(WAVEFORM_FILE, false).and_then(|path| std::fs::read(path).ok()) else {
        return;
    };

    let waves = data.get(..S_WAVBUF).unwrap_or(&data);
    let volumes = data.get(2 * S_WAVBUF..).unwrap_or(&[]);
    let volumes = &volumes[..volumes.len().min(S_VOLBUF)];

    unsafe {
        std::ptr::copy_nonoverlapping(waves.as_ptr(), the_memory.wavmem, waves.len());
        std::ptr::copy_nonoverlapping(volumes.as_ptr(), the_memory.volmem, volumes.len());
    }
}
