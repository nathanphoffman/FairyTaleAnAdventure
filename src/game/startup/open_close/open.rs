//! Opening everything the game needs: the screen, fonts, memory, music and the disk image.

use super::{
    BACKSAVE_OFFSET, IMAGE_SZ, S_VOLBUF, S_WAVBUF, SECTOR_SZ, SHADOW_SZ, SHAPE_SZ, TERRA_SZ,
};
use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::data::state::{Graphics, MapData};
use crate::game::data::story_text::{HINOR, HIVAR};
use crate::game::data::tables::{instrument_waves, pointer_sprite_data};
use crate::game::files::disk::open_disk;
use crate::game::render::blit::{Area, Loc};
use crate::game::render::screen::FontKind;
use crate::game::render::screen::Screen;
use crate::gfx::Font;
use crate::platform::audio::init_music;
use crate::platform::paths::resolve;

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
pub fn open_all(game: &mut Game, video: &mut Video) -> i32 {
    match open_everything(game, video) {
        Ok(()) => 0,
        Err(error) => error,
    }
}

/// The start-up steps in order; the first one that fails gives the error number.
fn open_everything(game: &mut Game, video: &mut Video) -> Result<(), i32> {
    video.screen.set_up([&HINOR, &HIVAR]);

    match open_disk() {
        0 => {}
        error => return Err(error),
    }

    open_fonts(&mut video.screen)?;
    set_up_blitter_memory(&mut video.screen);
    allocate_game_memory(&mut game.map, &mut video.graphics);

    load_music_waveforms();
    video.screen.pointer = Some(&pointer_sprite_data);

    Ok(())
}

/// Loads the story font; the panel uses the built-in one.
fn open_fonts(screen: &mut Screen) -> Result<(), i32> {
    let story = resolve(STORY_FONT_FILE, false).ok_or(ERROR_NO_FONT)?;
    let story = Font::load(&story).unwrap_or(Font::Builtin);

    screen.set_font(FontKind::Panel, Font::Builtin);
    screen.set_font(FontKind::Story, story);

    Ok(())
}

/// A zeroed memory area (with a little slack).
fn area(size: usize) -> Vec<u8> {
    vec![0u8; size + AREA_SLACK]
}

/// Points the saved backgrounds of the two pages into the blitter's work memory.
fn set_up_blitter_memory(screen: &mut Screen) {
    screen.pages[0].backsave = Loc::new(Area::Queue, BACKSAVE_OFFSET);
    screen.pages[1].backsave = Loc::new(Area::Mask, BACKSAVE_OFFSET);
}

/// Makes the picture, sector, shape, shadow-mask and terrain areas.
fn allocate_game_memory(map: &mut MapData, graphics: &mut Graphics) {
    graphics.image = area(IMAGE_SZ);
    map.sector = area(SECTOR_SZ);
    graphics.shape = area(SHAPE_SZ);
    graphics.shadow = area(SHADOW_SZ);
    map.terrain = area(TERRA_SZ);
}

/// Reads the waveforms and volume envelopes for the music from the file `v6` (the waveforms, then as
/// many bytes that are skipped, then the envelopes) and gives them to the music driver with the
/// instrument table.
fn load_music_waveforms() {
    let data = resolve(WAVEFORM_FILE, false)
        .and_then(|path| std::fs::read(path).ok())
        .unwrap_or_default();

    let mut waves = vec![0u8; S_WAVBUF];
    let mut volumes = vec![0u8; S_VOLBUF];
    copy_prefix(&mut waves, &data);
    copy_prefix(&mut volumes, data.get(2 * S_WAVBUF..).unwrap_or(&[]));

    let instruments = { instrument_waves.map(|word| word as u16) };
    init_music(instruments.to_vec(), waves, volumes);
}

/// Copies as much of `from` as fits into `to`.
fn copy_prefix(to: &mut [u8], from: &[u8]) {
    let n = to.len().min(from.len());
    to[..n].copy_from_slice(&from[..n]);
}
