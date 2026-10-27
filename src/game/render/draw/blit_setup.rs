//! Getting a sprite ready for the blitter: cutting it to the screen, where its background is saved,
//! and the blit values.

use super::*;

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
    if sprite.atype == OBJECTS && (sprite.inum & 128) != 0 {
        sprite.inum &= 0x7f;
        clip.yoff = clip.yoff.wrapping_add(8);
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

#[cfg(test)]
mod tests {
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
