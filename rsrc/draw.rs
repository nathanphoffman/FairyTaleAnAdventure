//! Drawing the actors each frame (from the main loop in csrc/fmain.c): finishing the scrolled map edge
//! and the witch's beam (`repair_map_and_witch`), and drawing every actor and its weapon into the page
//! (`draw_sprites`): clipping to the screen, hiding parts behind scenery, and saving the background
//! under each sprite so it can be put back next frame.

use super::actors::{actor_distance, hit_actor};
use super::asm_blit::{
    SShape, add_scenery_mask, clear_memory, combine_masks, draw_shape, save_background,
};
use super::asm_map::{draw_map_column, draw_map_row};
use super::game_tables::animation_frames;
use super::globals::{
    actor_list, aoff, bmask_mem, bmod, boff, draw_order, drawing_page, drawn_actor_count,
    extent_type, fiery_death, hero_sector, hero_x, hero_y, loaded_actor_file, map_x, map_y,
    mask_buffer, planes, planesize, riding, seq_list, shapedata, shift, shp, terrain_mem,
    witch_angle, witch_direction, witch_on_screen, wmask,
};
use super::random::{rand2, rand4};
use super::tables::{bow_x, bow_y, minimap};
use super::types::{SeqInfo, Shape, StateEntry};
use super::witch::witch_fx;
use super::witch::{SIDE_1 as s1, SIDE_2 as s2};

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

// ---- repair_map_and_witch ----
/// The map strip to redraw after a scroll: the right edge (36) or the left (0), and the bottom row
/// (10) or the top (0).
const STRIP_RIGHT: i32 = 36;
const STRIP_LEFT: i32 = 0;
const ROW_BOTTOM: i32 = 10;
const ROW_TOP: i32 = 0;
/// The witch's beam hurts the hero within this distance.
const WITCH_REACH: i32 = 100;
const WITCH_SLOT: usize = 2;

/// Finishes the scrolled map edge (`dif_x`, `dif_y` = scroll step) and runs the witch's beam and her
/// damage.
pub fn repair_map_and_witch(dif_x: i16, dif_y: i16) {
    unsafe {
        if dif_x == 1 {
            draw_map_column(STRIP_RIGHT);
        } else if dif_x == -1 {
            draw_map_column(STRIP_LEFT);
        }
        if dif_y == 1 {
            draw_map_row(ROW_BOTTOM);
        } else if dif_y == -1 {
            draw_map_row(ROW_TOP);
        }

        let witch = &raw const actor_list[WITCH_SLOT];
        let page = drawing_page;
        (*page).witchx = ((*witch).abs_x as i32 - (map_x & 0xfff0) as i32) as i16;
        (*page).witchy = (((*witch).abs_y as i32 - 15) - (map_y & 0xffe0) as i32) as i16;
        if rand4() == 0 {
            witch_direction = if s1 > 0 { 1 } else { -1 };
        }
        witch_angle = witch_angle.wrapping_add(witch_direction as u8);
        (*page).witchdir = witch_angle as i16;
        (*page).wflag = witch_on_screen as i16;
        if witch_on_screen != 0 {
            witch_fx(page);
            if s1 > 0 && s2 < 0 && actor_distance(WITCH_SLOT as i32, 0) < WITCH_REACH {
                hit_actor(-1, 0, (*witch).facing as i32, (rand2() + 1) as i16);
            }
        }
    }
}

// ---- draw_sprites ----
/// Screen size in pixels the sprites are clipped to (the play field is 320 by 174).
const SCREEN_RIGHT: i16 = 319;
const SCREEN_BOTTOM: i16 = 173;
/// Bytes in a page row, and the first row of the spare bit planes used as quick background saves.
const ROW_BYTES: usize = 40;
const SPARE_PLANE_OFFSET: usize = 192 * ROW_BYTES;
const SPARE_SAVES: u8 = 5;
/// A saved background smaller than this many bytes (per plane) goes in a spare plane.
const SMALL_SAVE: i16 = 64;
const PLANES: i32 = 5;
/// The background-save buffer holds this many bytes; running out stops drawing for the frame.
const SAVE_LIMIT: i32 = 74 * 80;
/// The mask buffer (`bmask_mem`) is cleared before each sprite: 96 lines of 64 words, plus 5.
const MASK_CLEAR_SIZE: i32 = (96 << 6) + 5;
/// Terrain flag bits `terrain_mem[tile*4+1] & 15` says how a tile hides what is behind it.
const TILE_FULL_MASK: i16 = 64 * 4;
/// Fall animation: tiles at or below this offset do not hide the actor.
const FALL_TILE_LIMIT: i16 = 220 * 4;
/// The bridge sector, where actors other than the raft walk in front of the bridge.
const BRIDGE_SECTOR: i16 = 48;
const RAFT_SLOT: i32 = 1;
/// Races that are never hidden behind scenery.
const RACE_NO_MASK_A: u8 = 0x85;
const RACE_NO_MASK_B: u8 = 0x87;
/// Enemy race 8 (the spectre) and 4 (snakes) look different.
const RACE_SPECTRE: u8 = 8;
const RACE_SNAKE: u8 = 4;
/// The `extent_type` of the room where actors are drawn in front of everything.
const NO_MASK_XTYPE: u16 = 84;
/// Sprite numbers (object images).
const OBJ_BURNING: i16 = 0x58;
const OBJ_SUNK: i16 = 97;
const OBJ_BOW_UP: i16 = 30;
const OBJ_BOW_MID: i16 = 0x53;
const OBJ_BOW_DOWN: i16 = 0x51;
const OBJ_ARROW_BASE: i16 = 103;
const WEAPON_BOW: i8 = 4;
const WEAPON_FIREBALL: i8 = 5;
/// The first frame of the bow's own drawing table; below this frames come from `bow_x`/`bow_y`.
const BOW_FRAMES: i16 = 32;
const WEAPON_SPRITE_BASE: [(i8, i16); 4] = [(2, 32), (3, 48), (1, 64), (4, 0)];

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

unsafe fn state_entry(inum: i16) -> &'static StateEntry {
    // Not range checked, like the C table lookup.
    unsafe {
        &*(&raw const animation_frames)
            .cast::<StateEntry>()
            .offset(inum as isize)
    }
}

fn byte_of(table: &[i8; 32], inum: i16) -> i16 {
    table.get(inum as usize).copied().unwrap_or(0) as i16
}

/// Draws every actor, and its weapon, into the page being drawn, and saves the background under each
/// (`backsave`) so it can be undone next frame. `cycle` is the frame counter.
pub fn draw_sprites(cycle: i16) {
    unsafe {
        let page = drawing_page;
        (*page).obcount = 0;
        (*page).saveused = 0;
        let mut frame = Frame {
            crack: 0,
            backalloc: (*page).backsave,
        };
        for j in 0..drawn_actor_count {
            let i = draw_order[j as usize] as i32;
            let an = &raw mut actor_list[i as usize];
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

/// Says whether this pass draws the weapon (`true`) or the body. An actor with a weapon is drawn in
/// two passes, the weapon first or second depending on which way it faces.
unsafe fn weapon_passmode(an: *const Shape, pass: &mut u8) -> bool {
    unsafe {
        let mut passmode = false;
        let state = (*an).state;
        if (*an).weapon > 0
            && (*an).weapon < 8
            && (state < DEAD || state >= SHOOT1 || extent_type > 80)
        {
            let facing = (*an).facing as i32;
            let flipped = *pass ^ 1;
            let mut pick = if ((facing - 2) & 4) != 0 {
                flipped
            } else {
                *pass
            };
            if (*an).weapon == WEAPON_BOW && state < SHOOT1 {
                pick = if (facing & 4) == 0 { flipped } else { *pass };
            }
            passmode = pick != 0;
            *pass += 1;
        }
        passmode
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

/// The part of a sprite that is on the screen.
struct Clip {
    xstart: i16,
    ystart: i16,
    xstop: i16,
    ystop: i16,
    /// How many pixels of the left edge, and lines of the top, are cut off.
    xoff: i8,
    yoff: i8,
}

/// Draws one actor (or its weapon) for the current pass.
unsafe fn draw_one(frame: &mut Frame, an: *mut Shape, i: i32, cycle: i16, passmode: bool) -> Drawn {
    unsafe {
        clear_scenery_mask();

        let Some(mut sprite) = choose_image(an, i, passmode) else {
            return Drawn::Done;
        };
        sprite.xstart += (map_x & 15) as i16;
        sprite.ystart += (map_y & 31) as i16;
        sprite.ground += (map_y & 31) as i16;

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
        save_background((*shp).backsave);

        if !drawn_in_front(an, i, sprite.atype, sprite.inum) {
            mask_against_scenery(an, i, sprite.ground, xbw, clip.ystart, clip.ystop, blitwide);
        }

        combine_masks();
        draw_shape();
        (*an).visible = 1;
        DisownBlitter();
        (*drawing_page).obcount += 1;
        Drawn::Done
    }
}

/// Cuts the sprite to the screen. Also notes in the image number whether the sprite is drawn from
/// the middle of its picture (bit 7).
fn clip_to_screen(sprite: &mut Sprite) -> Clip {
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

/// Fills in the blitter's registers and the queue entry for this sprite: sizes, where the image,
/// its mask and the saved background are, and the shifts. Returns false when the save buffer is
/// full.
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
        shp = (*drawing_page)
            .shape_queue
            .cast::<SShape>()
            .offset((*drawing_page).obcount as isize);
        planesize = seq.bytes;
        (*shp).savesize = (blitwide as i32 * blithigh as i32 * 2) as i16;
        (*shp).blitsize = (((blithigh as i32) << 6) + blitwide as i32) as i16;

        shapedata = seq
            .location
            .offset(planesize as isize * PLANES as isize * sprite.inum as isize);
        mask_buffer = seq
            .maskloc
            .offset(planesize as isize * sprite.inum as isize);
        if !assign_background_save(frame) {
            return false;
        }

        aoff = ((clip.ystart & 31) as i32 * blitwide as i32 * 2) as i16;
        boff = ((clip.xoff >> 4) as i32 * 2 + cwide as i32 * clip.yoff as i32) as i16;
        (*shp).coff = (xbw as i32 * 2 + clip.ystart as i32 * 40) as i16;
        shift = sprite.xstart & 15;
        if clip.xoff & 15 != 0 {
            (*shp).coff -= 2;
            wmask = 0;
        } else {
            wmask = -1;
        }
        bmod = (cwide as i32 - blitwide as i32 * 2) as i16;
        (*shp).cmod = (40 - blitwide as i32 * 2) as i16;
        true
    }
}

/// Where the images of one kind of sprite are (`seq_list` entry `atype`).
unsafe fn sprite_sheet(atype: i16) -> &'static SeqInfo {
    unsafe {
        &*(&raw const seq_list)
            .cast::<SeqInfo>()
            .offset(atype as isize)
    }
}

/// Zeroes the scenery mask buffer, ready for the next sprite.
unsafe fn clear_scenery_mask() {
    unsafe {
        OwnBlitter();
        WaitBlit();
        clear_memory(bmask_mem, MASK_CLEAR_SIZE);
        DisownBlitter();
    }
}

/// Picks the image for this pass: the weapon's (`passmode`) or the actor's own, and where its top
/// left corner is on the screen. `None` means nothing is drawn.
unsafe fn choose_image(an: *const Shape, i: i32, passmode: bool) -> Option<Sprite> {
    unsafe {
        let inum = (*an).index as i16;
        let xstart = (*an).rel_x as i16;
        let ystart = (*an).rel_y as i16;
        let ground = ystart.wrapping_add(32);
        let blank = Sprite {
            atype: OBJECTS,
            inum,
            xstart,
            ystart,
            xstop: 0,
            ystop: 0,
            ground,
        };
        if passmode {
            return Some(weapon_image(an, blank));
        }
        let (atype, inum) = body_image(an, i, inum)?;
        Some(Sprite {
            atype,
            inum,
            ..blank
        })
    }
}

/// The weapon's sprite: where it is held for this animation frame, and which image it is.
unsafe fn weapon_image(an: *const Shape, mut sprite: Sprite) -> Sprite {
    unsafe {
        let mut kind = (*an).weapon as i16;
        let frame_number = sprite.inum;
        let is_bow_frame = kind == WEAPON_BOW as i16 && frame_number < BOW_FRAMES;
        if is_bow_frame {
            sprite.xstart += byte_of(&bow_x, frame_number);
            sprite.ystart += byte_of(&bow_y, frame_number);
        } else {
            let entry = state_entry(frame_number);
            sprite.xstart += entry.wpn_x as i16;
            sprite.ystart += entry.wpn_y as i16;
        }
        if is_bow_frame {
            let bow_phase = frame_number / 8;
            sprite.inum = if bow_phase & 1 != 0 {
                OBJ_BOW_UP
            } else if bow_phase & 2 != 0 {
                OBJ_BOW_MID
            } else {
                OBJ_BOW_DOWN
            };
        } else if kind == WEAPON_FIREBALL as i16 {
            sprite.inum = (*an).facing as i16 + OBJ_ARROW_BASE;
            if (*an).facing == 2 {
                sprite.ystart -= 6;
            }
        } else {
            // hand weapons: each kind has its own block of images
            if let Some(&(_, base)) = WEAPON_SPRITE_BASE.iter().find(|(w, _)| *w as i16 == kind) {
                kind = base;
            }
            sprite.inum = state_entry(frame_number).wpn_no as i16 + kind;
        }
        sprite.atype = OBJECTS;
        sprite
    }
}

/// The actor's own sprite sheet and image for this animation frame. `None` means nothing is drawn
/// (a burnt body, or an object with no image).
unsafe fn body_image(an: *const Shape, i: i32, mut inum: i16) -> Option<(i16, i16)> {
    unsafe {
        let mut atype = (*an).kind as i16;
        if (atype == ENEMY && (*an).race != RACE_SPECTRE) || atype == PHIL {
            if fiery_death != 0 && (*an).environ > 0 {
                if (*an).state == DEAD {
                    return None;
                } else if (*an).state == DYING {
                    // a body burning in the lava: nothing more to work out
                    return Some((OBJECTS, OBJ_BURNING));
                }
            } else if (*an).state == FALL {
                atype = if (*an).tactic < 16 { ENEMY } else { OBJECTS };
            } else {
                inum = state_entry(inum).figure as i16;
            }
            if (*an).race == RACE_SNAKE {
                if (*an).state < DYING {
                    inum += 0x24;
                }
            } else if i > 0 {
                if (*an).race & 1 != 0 {
                    inum |= 1;
                } else {
                    inum &= 0xfffeu16 as i16;
                }
            }
        }
        if atype == OBJECTS && (*an).race == 0 {
            return None;
        }
        if atype == CARRIER && riding == 0 && loaded_actor_file == 11 {
            atype = RAFT;
            inum = 1;
        }
        Some((atype, inum))
    }
}

/// The height of an image in lines: the sheet's, except for some small objects.
fn image_height(atype: i16, inum: i16, sheet_height: i8) -> i8 {
    let small_object = inum == 0x1b
        || (8..=12).contains(&inum)
        || inum == 25
        || inum == 26
        || (inum > 0x10 && inum < 0x18)
        || (inum & 128) != 0;
    if atype == OBJECTS && small_object {
        8
    } else {
        sheet_height
    }
}

/// Sinks the sprite into water, shortens it when the hero rides the swan, and so on. Returns false
/// if nothing is left to draw.
unsafe fn adjust_for_depth(
    an: *const Shape,
    i: i32,
    cycle: i16,
    passmode: bool,
    sprite: &mut Sprite,
) -> bool {
    unsafe {
        let depth = (*an).environ as i16;
        if !passmode && sprite.atype != OBJECTS {
            if i == 0 && riding == 11 {
                sprite.ystop -= 16;
            } else if depth == 2 {
                sprite.ystop -= 10;
            } else if depth > 29 {
                if (*an).state == DEAD {
                    return false;
                }
                // under the surface: only the bubbles show
                sprite.ystart += 27;
                sprite.ystop = sprite.ystart + 7;
                sprite.atype = OBJECTS;
                sprite.inum = OBJ_SUNK + ((cycle as i32 + i) & 1) as i16;
            } else if depth > 2 {
                sprite.ystart += depth;
            }
        } else {
            if depth > 29 {
                return false; // no weapon
            }
            if depth > 2 {
                sprite.ystart += depth;
                sprite.ystop += depth;
                if sprite.ystop > sprite.ground {
                    sprite.ystop = sprite.ground;
                }
                if sprite.ystart >= sprite.ground {
                    return false;
                }
                sprite.ground += depth;
            }
        }
        true
    }
}

/// Reserves room to save the background under the sprite: a spare bit plane while they last (for
/// small sprites), otherwise the page's save buffer. Returns false when the buffer is full.
unsafe fn assign_background_save(frame: &mut Frame) -> bool {
    unsafe {
        let page = drawing_page;
        if (*shp).savesize < SMALL_SAVE && frame.crack < SPARE_SAVES {
            (*shp).backsave = (*planes.add(frame.crack as usize)).add(SPARE_PLANE_OFFSET);
            frame.crack += 1;
            return true;
        }
        (*shp).backsave = frame.backalloc;
        let bytes = PLANES * (*shp).savesize as i32;
        frame.backalloc = frame.backalloc.offset(bytes as isize);
        (*page).saveused += bytes;
        (*page).saveused < SAVE_LIMIT
    }
}

/// True for sprites drawn in front of everything: carriers, the hero when riding the swan or in
/// certain places, arrows and some special races.
unsafe fn drawn_in_front(an: *const Shape, i: i32, atype: i16, inum: i16) -> bool {
    unsafe {
        if atype == CARRIER
            || (i == 0
                && (riding == 11
                    || extent_type == NO_MASK_XTYPE
                    || (hero_x > 22833 && hero_x < 26428 && hero_y > 26425 && hero_y < 26527)))
        {
            return true;
        }
        if atype == OBJECTS && inum > 99 && inum < 102 {
            return true;
        }
        (*an).race == RACE_NO_MASK_A || (*an).race == RACE_NO_MASK_B
    }
}

/// Cuts out the parts of the sprite that scenery (trees, walls, roofs) should hide, one map tile at a
/// time, by marking them in the mask buffer.
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
                let mut k = (*terrain_mem.offset(cm as isize + 1) & 15) as i16;
                if (*an).state == FALL {
                    if cm <= FALL_TILE_LIMIT {
                        continue;
                    }
                    k = 3;
                }
                match k {
                    0 => continue,
                    1 => {
                        if xm == 0 {
                            continue;
                        }
                    }
                    2 => {
                        if ystop > 35 {
                            continue;
                        }
                    }
                    3 => {
                        if hero_sector == BRIDGE_SECTOR && i != RAFT_SLOT {
                            continue;
                        }
                    }
                    4 => {
                        if xm == 0 || ystop > 35 {
                            continue;
                        }
                    }
                    5 => {
                        if xm == 0 && ystop > 35 {
                            continue;
                        }
                    }
                    6 => {
                        if ym != 0 {
                            cm = TILE_FULL_MASK; // full if above
                        }
                    }
                    7 => {
                        if ystop > 20 {
                            continue;
                        }
                    }
                    _ => {}
                }
                add_scenery_mask(
                    xm,
                    ym,
                    blitwide as i32,
                    *terrain_mem.offset(cm as isize) as i32,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn small_objects_are_eight_lines_high() {
        for small in [0x1b, 8, 12, 25, 26, 0x11, 0x17, 0x80] {
            assert_eq!(image_height(OBJECTS, small, 32), 8, "image {small}");
        }
    }

    #[test]
    fn other_images_keep_the_sheet_height() {
        for big in [0, 7, 13, 24, 28, 0x10, 0x18] {
            assert_eq!(image_height(OBJECTS, big, 32), 32, "image {big}");
        }
        // only objects have small images
        assert_eq!(image_height(ENEMY, 0x1b, 32), 32);
    }

    #[test]
    fn table_lookups_outside_the_table_give_zero() {
        assert_eq!(byte_of(&bow_x, 0), bow_x[0] as i16);
        assert_eq!(byte_of(&bow_y, 31), bow_y[31] as i16);
        assert_eq!(byte_of(&bow_x, 32), 0);
        assert_eq!(byte_of(&bow_x, -1), 0);
    }

    #[test]
    fn a_sprite_inside_the_screen_is_not_cut() {
        let mut sprite = sprite_at(PHIL, 5, 100, 50, 131, 81);
        let clip = clip_to_screen(&mut sprite);
        assert_eq!((clip.xstart, clip.ystart, clip.xstop, clip.ystop), (100, 50, 131, 81));
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

    #[test]
    fn images_with_bit_seven_start_eight_lines_into_the_picture() {
        let mut sprite = sprite_at(OBJECTS, 0x85, 10, 10, 25, 17);
        let clip = clip_to_screen(&mut sprite);
        assert_eq!(sprite.inum, 5);
        assert_eq!(clip.yoff, 8);
    }
}
