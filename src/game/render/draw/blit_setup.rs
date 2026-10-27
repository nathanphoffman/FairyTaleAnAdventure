//! Getting a sprite ready for the blitter: cutting it to the screen, where its background is saved,
//! and the blit values.

use super::images::sprite_sheet;
use super::{Frame, OBJECTS, Sprite};
use crate::game::data::globals::current_sshape;
use crate::game::data::globals::{
    last_word_mask, shadow_mask_offset, sprite_mask_modulo, sprite_mask_offset, sprite_x_shift,
};
use crate::game::data::state::{the_memory, the_screen};
use crate::game::render::blit::SShape;

// ---- draw_sprites ----
/// Screen size in pixels the sprites are clipped to (the play field is 320 by 174).
pub(super) const SCREEN_RIGHT: i16 = 319;
pub(super) const SCREEN_BOTTOM: i16 = 173;

/// Bytes in a page row, and the first row of the spare bit planes used as quick background saves.
const ROW_BYTES: usize = 40;
const SPARE_PLANE_OFFSET: usize = 192 * ROW_BYTES;
const SPARE_SAVES: u8 = 5;

/// A saved background smaller than this many bytes (per plane) goes in a spare plane.
const SMALL_SAVE: i16 = 64;
pub(super) const PLANES: i32 = 5;

/// The background-save buffer holds this many bytes; running out stops drawing for the frame.
const SAVE_LIMIT: i32 = 74 * 80;

/// The part of a sprite that is on the screen.
pub(super) struct Clip {
    pub(super) xstart: i16,
    pub(super) ystart: i16,
    pub(super) xstop: i16,
    pub(super) ystop: i16,
    /// How many pixels of the left edge, and lines of the top, are cut off.
    pub(super) xoff: i8,
    pub(super) yoff: i8,
}

/// Bit 7 of an object's image number means it is drawn from the middle of its picture (8 lines in).
const FROM_MIDDLE_BIT: i16 = 128;
const MIDDLE_START: i8 = 8;
const IMAGE_NUMBER_MASK: i16 = 0x7f;

/// The blitter's size word has the height in the bits above the 6 bit width.
const BLIT_HEIGHT_SHIFT: u32 = 6;

/// A sprite is 16 pixels to a word (2 bytes); the shadow mask repeats every 32 lines; the page is
/// 40 bytes wide.
const WORD_PIXELS_MASK: i16 = 15;
const WORD_PIXELS_SHIFT: u32 = 4;
const WORD_BYTES: i32 = 2;
const SHADOW_LINES_MASK: i16 = 31;
const PAGE_ROW_BYTES: i32 = 40;

/// Cuts the sprite to the screen. Also notes in the image number whether the sprite is drawn from
/// the middle of its picture (bit 7).
pub(super) fn clip_to_screen(sprite: &mut Sprite) -> Clip {
    let xstart = sprite.xstart.max(0);
    let ystart = sprite.ystart.max(0);
    let mut clip = Clip {
        xstart,
        ystart,
        xstop: sprite.xstop.min(SCREEN_RIGHT),
        ystop: sprite.ystop.min(SCREEN_BOTTOM),
        xoff: (xstart - sprite.xstart) as i8,
        yoff: (ystart - sprite.ystart) as i8,
    };

    if sprite.atype == OBJECTS && (sprite.inum & FROM_MIDDLE_BIT) != 0 {
        sprite.inum &= IMAGE_NUMBER_MASK;
        clip.yoff = clip.yoff.wrapping_add(MIDDLE_START);
    }

    clip
}

/// Reserves room to save the background under the sprite: a spare bit plane while they last (for
/// small sprites), otherwise the page's save buffer. Returns false when the buffer is full.
pub(super) unsafe fn assign_background_save(frame: &mut Frame) -> bool {
    unsafe {
        let page = the_screen.drawing_page;

        if (*current_sshape).savesize < SMALL_SAVE && frame.crack < SPARE_SAVES {
            (*current_sshape).backsave =
                (*the_screen.planes.add(frame.crack as usize)).add(SPARE_PLANE_OFFSET);
            frame.crack += 1;

            return true;
        }

        (*current_sshape).backsave = frame.backalloc;
        let bytes = PLANES * (*current_sshape).savesize as i32;
        frame.backalloc = frame.backalloc.offset(bytes as isize);
        (*page).saveused += bytes;

        (*page).saveused < SAVE_LIMIT
    }
}

/// Fills in the blitter's registers and the queue entry for this sprite: sizes, where the image,
/// its mask and the saved background are, and the shifts. Returns false when the save buffer is
/// full.
#[allow(clippy::too_many_arguments)] // the blit set-up of the original, kept as one call
pub(super) unsafe fn set_up_blit(
    frame: &mut Frame,
    sprite: &Sprite,
    clip: &Clip,
    cwide: i8,
    xbw: i8,
    blitwide: i8,
    blithigh: i8,
) -> bool {
    unsafe {
        // The sheet is looked up again: a sunk actor is drawn from the objects sheet, but is as wide
        // and high as its own.
        let seq = sprite_sheet(sprite.atype);
        current_sshape = (*the_screen.drawing_page)
            .shape_queue
            .cast::<SShape>()
            .offset((*the_screen.drawing_page).obcount as isize);
        the_screen.planesize = seq.bytes;
        (*current_sshape).savesize = (blitwide as i32 * blithigh as i32 * WORD_BYTES) as i16;
        // Blitter size word: height in the high bits, width in words in the low 6 bits.
        (*current_sshape).blitsize =
            (((blithigh as i32) << BLIT_HEIGHT_SHIFT) + blitwide as i32) as i16;

        the_memory.shapedata = seq
            .location
            .offset(the_screen.planesize as isize * PLANES as isize * sprite.inum as isize);
        the_memory.mask_buffer = seq
            .maskloc
            .offset(the_screen.planesize as isize * sprite.inum as isize);

        if !assign_background_save(frame) {
            return false;
        }

        set_blitter_registers(sprite, clip, cwide, xbw, blitwide);

        true
    }
}

/// Sets the blitter's mask offsets, shift and modulo for the clipped sprite, and where in the page
/// it goes.
unsafe fn set_blitter_registers(sprite: &Sprite, clip: &Clip, cwide: i8, xbw: i8, blitwide: i8) {
    unsafe {
        shadow_mask_offset =
            ((clip.ystart & SHADOW_LINES_MASK) as i32 * blitwide as i32 * WORD_BYTES) as i16;
        // Where the visible part starts in the mask: whole words (2 bytes per 16 pixels) in, and whole rows down.
        sprite_mask_offset = ((clip.xoff >> WORD_PIXELS_SHIFT) as i32 * WORD_BYTES
            + cwide as i32 * clip.yoff as i32) as i16;
        (*current_sshape).coff =
            (xbw as i32 * WORD_BYTES + clip.ystart as i32 * PAGE_ROW_BYTES) as i16;
        sprite_x_shift = sprite.xstart & WORD_PIXELS_MASK;

        if clip.xoff & WORD_PIXELS_MASK as i8 != 0 {
            (*current_sshape).coff -= WORD_BYTES as i16;
            last_word_mask = 0;
        } else {
            last_word_mask = -1;
        }

        sprite_mask_modulo = (cwide as i32 - blitwide as i32 * WORD_BYTES) as i16;
        (*current_sshape).cmod = (PAGE_ROW_BYTES - blitwide as i32 * WORD_BYTES) as i16;
    }
}

#[cfg(test)]
mod tests {
    use super::super::PHIL;
    use super::super::sprite_at;
    use super::*;

    #[test]
    fn a_sprite_inside_the_screen_is_not_cut() {
        let mut sprite = sprite_at(PHIL, 5, 100, 50, 131, 81);
        let clip = clip_to_screen(&mut sprite);
        assert_eq!(
            (clip.xstart, clip.ystart, clip.xstop, clip.ystop),
            (100, 50, 131, 81)
        );

        assert_eq!((clip.xoff, clip.yoff), (0, 0));
    }

    #[test]
    fn a_sprite_over_the_left_and_top_edges_is_cut() {
        let mut sprite = sprite_at(PHIL, 5, -5, -10, 26, 21);
        let clip = clip_to_screen(&mut sprite);
        assert_eq!((clip.xstart, clip.ystart), (0, 0));
        assert_eq!((clip.xoff, clip.yoff), (5, 10));
    }

    #[test]
    fn a_sprite_over_the_right_and_bottom_edges_stops_at_the_screen() {
        let mut sprite = sprite_at(PHIL, 5, 300, 160, 331, 191);
        let clip = clip_to_screen(&mut sprite);
        assert_eq!((clip.xstop, clip.ystop), (SCREEN_RIGHT, SCREEN_BOTTOM));
    }
}
