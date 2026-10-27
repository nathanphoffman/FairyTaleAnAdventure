//! Drawing the actors each frame (from the main loop in csrc/fmain.c): finishing the scrolled map edge
//! and the witch's beam (`repair_map_and_witch`), and drawing every actor and its weapon into the page
//! (`draw_sprites`): clipping to the screen, hiding parts behind scenery, and saving the background
//! under each sprite so it can be put back next frame.

mod blit_setup;
mod images;
mod map_edge;
mod scenery;
mod weapons;

use self::blit_setup::*;
use self::images::*;
pub use self::map_edge::repair_map_and_witch;
use self::scenery::*;
use self::weapons::*;

use crate::game::actors::interaction::{actor_distance, hit_actor};
use crate::game::data::globals::{
    current_sshape, last_word_mask, shadow_mask_offset, sprite_mask_modulo, sprite_mask_offset,
    sprite_x_shift,
};
use crate::game::data::object_tables::{bow_x, bow_y, minimap};
use crate::game::data::random::{rand2, rand4};
use crate::game::data::state::{
    the_actors, the_hero, the_memory, the_screen, the_witch, the_world,
};
use crate::game::data::tables::animation_frames;
use crate::game::data::types::{SeqInfo, Shape, StateEntry};
use crate::game::render::blit::{
    SShape, add_scenery_mask, clear_memory, combine_masks, draw_shape, save_background,
};
use crate::game::render::witch::witch_fx;
use crate::game::render::witch::{SIDE_1 as s1, SIDE_2 as s2};
use crate::game::world::map::{draw_map_column, draw_map_row};

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
            let an = &raw mut the_actors.list[i as usize];
            (*an).visible = 0;
            let mut pass: u8 = 0;
            loop {
                let passmode = weapon_passmode(an, &mut pass);
                if let Drawn::OutOfRoom = draw_one(&mut frame, an, i, cycle, passmode) {
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
unsafe fn draw_one(frame: &mut Frame, an: *mut Shape, i: i32, cycle: i16, passmode: bool) -> Drawn {
    unsafe {
        clear_scenery_mask();

        let Some(mut sprite) = choose_image(an, i, passmode) else {
            return Drawn::Done;
        };
        sprite.xstart += (the_world.map_x & 15) as i16;
        sprite.ystart += (the_world.map_y & 31) as i16;
        sprite.ground += (the_world.map_y & 31) as i16;

        let seq = sprite_sheet(sprite.atype);
        let xsize = (seq.width << 4) as i8;
        let ysize = image_height(sprite.atype, sprite.inum, seq.height as i8);
        let cwide = xsize / 8;
        sprite.xstop = sprite.xstart.wrapping_add(xsize as i16).wrapping_sub(1);
        sprite.ystop = sprite.ystart.wrapping_add(ysize as i16).wrapping_sub(1);

        if !adjust_for_depth(an, i, cycle, passmode, &mut sprite) {
            return Drawn::Done;
        }
        if sprite.xstart > SCREEN_RIGHT
            || sprite.ystart > SCREEN_BOTTOM
            || sprite.xstop < 0
            || sprite.ystop < 0
        {
            return Drawn::Done;
        }

        let clip = clip_to_screen(&mut sprite);
        // word offsets
        let xbw = (clip.xstart >> 4) as i8;
        let xew = (clip.xstop >> 4) as i8;
        let mut blitwide = (xew as i16 - xbw as i16 + 1) as i8;
        let blithigh = (clip.ystop - clip.ystart + 1) as i8;
        if clip.xoff & 15 != 0 {
            blitwide = blitwide.wrapping_add(1);
        }

        if !set_up_blit(frame, &sprite, &clip, cwide, xbw, blitwide, blithigh) {
            return Drawn::OutOfRoom; // no more blits if no room
        }

        OwnBlitter();
        save_background((*current_sshape).backsave);

        if !drawn_in_front(an, i, sprite.atype, sprite.inum) {
            mask_against_scenery(an, i, sprite.ground, xbw, clip.ystart, clip.ystop, blitwide);
        }

        combine_masks();
        draw_shape();
        (*an).visible = 1;
        DisownBlitter();
        (*the_screen.drawing_page).obcount += 1;
        Drawn::Done
    }
}

/// Fills in the blitter's registers and the queue entry for this sprite: sizes, where the image,
/// its mask and the saved background are, and the shifts. Returns false when the save buffer is
/// full.
#[allow(clippy::too_many_arguments)] // the blit set-up of the original, kept as one call
unsafe fn set_up_blit(
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
        (*current_sshape).savesize = (blitwide as i32 * blithigh as i32 * 2) as i16;
        (*current_sshape).blitsize = (((blithigh as i32) << 6) + blitwide as i32) as i16;

        the_memory.shapedata = seq
            .location
            .offset(the_screen.planesize as isize * PLANES as isize * sprite.inum as isize);
        the_memory.mask_buffer = seq
            .maskloc
            .offset(the_screen.planesize as isize * sprite.inum as isize);
        if !assign_background_save(frame) {
            return false;
        }

        shadow_mask_offset = ((clip.ystart & 31) as i32 * blitwide as i32 * 2) as i16;
        sprite_mask_offset = ((clip.xoff >> 4) as i32 * 2 + cwide as i32 * clip.yoff as i32) as i16;
        (*current_sshape).coff = (xbw as i32 * 2 + clip.ystart as i32 * 40) as i16;
        sprite_x_shift = sprite.xstart & 15;
        if clip.xoff & 15 != 0 {
            (*current_sshape).coff -= 2;
            last_word_mask = 0;
        } else {
            last_word_mask = -1;
        }
        sprite_mask_modulo = (cwide as i32 - blitwide as i32 * 2) as i16;
        (*current_sshape).cmod = (40 - blitwide as i32 * 2) as i16;
        true
    }
}

/// Cuts out the parts of the sprite that scenery (trees, walls, roofs) should hide, one map tile at a
/// time, by marking them in the mask buffer.
#[allow(clippy::too_many_arguments)] // the tile loop of the original, kept as one call
unsafe fn mask_against_scenery(
    an: *const Shape,
    i: i32,
    ground: i16,
    xbw: i8,
    ystart1: i16,
    ystop1: i16,
    blitwide: i8,
) {
    unsafe {
        let ym1 = (ystart1 >> 5) as u8;
        let ym2 = ((ystop1 >> 5) as u8).wrapping_sub(ym1);
        for xm in 0..blitwide.max(0) as i32 {
            for ym in 0..=ym2 as i32 {
                let ystop = ground - (((ym + ym1 as i32) << 5) as i16);
                let mut cm = ((xm + xbw as i32) * 6 + ym + ym1 as i32) as i16;
                cm = (minimap[cm as usize] as i32 * 4) as i16;
                let mut k = (*the_memory.terrain_mem.offset(cm as isize + 1) & 15) as i16;
                if (*an).state == FALL {
                    if cm <= FALL_TILE_LIMIT {
                        continue;
                    }
                    k = 3;
                }
                let on_bridge = the_hero.sector == BRIDGE_SECTOR && i != RAFT_SLOT;
                match scenery_rule(k, xm, ym, ystop, on_bridge) {
                    SceneryRule::Skip => continue,
                    SceneryRule::UseFullMask => cm = TILE_FULL_MASK,
                    SceneryRule::UseTileMask => {}
                }
                add_scenery_mask(
                    xm,
                    ym,
                    blitwide as i32,
                    *the_memory.terrain_mem.offset(cm as isize) as i32,
                );
            }
        }
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
