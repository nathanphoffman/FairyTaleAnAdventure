//! Drawing the actors each frame (from the main loop in csrc/fmain.c): finishing the scrolled map edge
//! and the witch's beam (`repair_map_and_witch`), and drawing every actor and its weapon into the page
//! (`draw_sprites`): clipping to the screen, hiding parts behind scenery, and saving the background
//! under each sprite so it can be put back next frame.

mod blit_setup;

mod images;

mod map_edge;

mod scenery;

mod weapons;

use self::blit_setup::set_up_blit;
use self::scenery::mask_against_scenery;

pub use self::map_edge::repair_map_and_witch;

use self::blit_setup::{Clip, SCREEN_BOTTOM, SCREEN_RIGHT, clip_to_screen};

use self::images::{adjust_for_depth, choose_image, image_height, sprite_sheet};
use self::scenery::{clear_scenery_mask, drawn_in_front};

use self::weapons::weapon_passmode;

use crate::game::data::globals::current_sshape;

use crate::game::data::state::{the_actors, the_screen, the_world};
use crate::game::data::types::Shape;
use crate::game::render::blit::{combine_masks, draw_shape, save_background};

unsafe extern "C" {
    fn OwnBlitter();

    fn DisownBlitter();

    fn WaitBlit();
}

// Actor kinds and states (csrc/ftale.h).
const PHIL: i16 = 0;
const OBJECTS: i16 = 1;
const ENEMY: i16 = 2;
const RAFT: i16 = 3;
const CARRIER: i16 = 5;
const DEAD: i8 = 15;
const DYING: i8 = 14;
const FALL: i8 = 22;
const SHOOT1: i8 = 24;
const RAFT_SLOT: i32 = 1;

/// What is left of the frame's save buffers as the actors are drawn one by one.
struct Frame {
    /// How many spare planes have been used for small background saves.
    crack: u8,
    /// Next free byte of the page's background-save buffer.
    backalloc: *mut u8,
}

/// How one actor's drawing ended.
enum Drawn {
    /// Drawn, or skipped because it is off screen.
    Done,
    /// The save buffer is full: no more actors this frame.
    OutOfRoom,
}

/// Draws every actor, and its weapon, into the page being drawn, and saves the background under each
/// (`backsave`) so it can be undone next frame. `cycle` is the frame counter.
pub fn draw_sprites(cycle: i16) {
    unsafe {
        let page = the_screen.drawing_page;
        (*page).obcount = 0;
        (*page).saveused = 0;
        let mut frame = Frame {
            crack: 0,
            backalloc: (*page).backsave,
        };

        for j in 0..the_actors.drawn {
            let i = the_actors.draw_order[j as usize] as i32;
            let actor = &raw mut the_actors.list[i as usize];
            (*actor).visible = 0;
            let mut pass: u8 = 0;

            loop {
                let passmode = weapon_passmode(actor, &mut pass);

                if let Drawn::OutOfRoom = draw_one(&mut frame, actor, i, cycle, passmode) {
                    return;
                }

                if pass != 1 {
                    break;
                }
            }
        }
    }
}

/// Which image to draw for an actor, and where on the screen it goes.
struct Sprite {
    /// The kind of sprite sheet the image is in (`PHIL`, `OBJECTS`, ...).
    atype: i16,
    /// The image number in that sheet.
    inum: i16,
    xstart: i16,
    ystart: i16,
    xstop: i16,
    ystop: i16,
    /// The line the actor stands on; things hidden behind scenery are cut against it.
    ground: i16,
}

/// Draws one actor (or its weapon) for the current pass.
unsafe fn draw_one(
    frame: &mut Frame,
    actor: *mut Shape,
    i: i32,
    cycle: i16,
    passmode: bool,
) -> Drawn {
    unsafe {
        clear_scenery_mask();

        let Some(mut sprite) = choose_image(actor, i, passmode) else {
            return Drawn::Done;
        };

        let bytes_wide = place_on_screen(&mut sprite);

        if !adjust_for_depth(actor, i, cycle, passmode, &mut sprite) {
            return Drawn::Done;
        }

        if is_off_screen(&sprite) {
            return Drawn::Done;
        }

        let clip = clip_to_screen(&mut sprite);

        blit_sprite(frame, actor, i, &sprite, &clip, bytes_wide)
    }
}

/// Moves the sprite from map to screen coordinates and works out its size and bottom right corner.
/// Returns the width of the image in bytes.
/// The map scrolls in whole tiles (16 x 32 pixels), so the part of a tile it is offset by is added.
const TILE_X_MASK: i32 = 15;
const TILE_Y_MASK: i32 = 31;

/// A word is 16 pixels (a shift of 4); a sprite's width in bytes is its width in pixels over 8.
const WORD_SHIFT: u32 = 4;
const PIXELS_PER_BYTE: i8 = 8;
const WORD_PIXELS_MASK: i16 = 15;

unsafe fn place_on_screen(sprite: &mut Sprite) -> i8 {
    unsafe {
        sprite.xstart += (the_world.map_x as i32 & TILE_X_MASK) as i16;
        sprite.ystart += (the_world.map_y as i32 & TILE_Y_MASK) as i16;
        sprite.ground += (the_world.map_y as i32 & TILE_Y_MASK) as i16;

        let seq = sprite_sheet(sprite.atype);
        // Sprite width in pixels: 16 pixels to each word.
        let xsize = (seq.width << WORD_SHIFT) as i8;
        let ysize = image_height(sprite.atype, sprite.inum, seq.height as i8);
        sprite.xstop = sprite.xstart.wrapping_add(xsize as i16).wrapping_sub(1);
        sprite.ystop = sprite.ystart.wrapping_add(ysize as i16).wrapping_sub(1);

        xsize / PIXELS_PER_BYTE
    }
}

/// Whether the sprite is completely outside the screen.
fn is_off_screen(sprite: &Sprite) -> bool {
    sprite.xstart > SCREEN_RIGHT
        || sprite.ystart > SCREEN_BOTTOM
        || sprite.xstop < 0
        || sprite.ystop < 0
}

/// Where the clipped sprite starts and how big it is, in words and lines: the first word column, the
/// width in words (one more when the sprite is shifted into the next word) and the height.
fn blit_extent(clip: &Clip) -> (i8, i8, i8) {
    // Pixel columns to word columns (16 pixels to a word).
    let xbw = (clip.xstart >> WORD_SHIFT) as i8;
    let xew = (clip.xstop >> WORD_SHIFT) as i8;
    let mut blitwide = (xew as i16 - xbw as i16 + 1) as i8;
    let blithigh = (clip.ystop - clip.ystart + 1) as i8;

    if clip.xoff as i16 & WORD_PIXELS_MASK != 0 {
        blitwide = blitwide.wrapping_add(1);
    }

    (xbw, blitwide, blithigh)
}

/// Saves the background under the sprite and draws it through its masks into the page.
unsafe fn blit_sprite(
    frame: &mut Frame,
    actor: *mut Shape,
    i: i32,
    sprite: &Sprite,
    clip: &Clip,
    bytes_wide: i8,
) -> Drawn {
    unsafe {
        let (xbw, blitwide, blithigh) = blit_extent(clip);

        if !set_up_blit(frame, sprite, clip, bytes_wide, xbw, blitwide, blithigh) {
            return Drawn::OutOfRoom; // no more blits if no room
        }

        OwnBlitter();
        save_background((*current_sshape).backsave);

        if !drawn_in_front(actor, i, sprite.atype, sprite.inum) {
            mask_against_scenery(
                actor,
                i,
                sprite.ground,
                xbw,
                clip.ystart,
                clip.ystop,
                blitwide,
            );
        }

        combine_masks();
        draw_shape();
        (*actor).visible = 1;
        DisownBlitter();
        (*the_screen.drawing_page).obcount += 1;

        Drawn::Done
    }
}

/// A sprite of `atype` and `inum` over the given rectangle, for the tests of the pieces.
#[cfg(test)]
fn sprite_at(atype: i16, inum: i16, xstart: i16, ystart: i16, xstop: i16, ystop: i16) -> Sprite {
    Sprite {
        atype,
        inum,
        xstart,
        ystart,
        xstop,
        ystop,
        ground: ystart + 32,
    }
}
