//! Drawing the actors each frame (from the main loop in csrc/fmain.c): finishing the scrolled map edge
//! and the witch's beam (`repair_map_and_witch`), and drawing every actor and its weapon into the page
//! (`draw_sprites`): clipping to the screen, hiding parts behind scenery, and saving the background
//! under each sprite so it can be put back next frame.

use super::actors::{calc_dist, dohit};
use super::asm_blit::{SShape, clear_blit, mask_blit, maskit, save_blit, shape_blit};
use super::asm_map::{row_draw, strip_draw};
use super::random::{rand2, rand4};
use super::tables::{bow_x, bow_y, minimap};
use super::types::{FPage, SeqInfo, Shape, StateEntry};
use super::witch::witch_fx;

unsafe extern "C" {
    static mut anim_list: [Shape; 20];
    static anim_index: [u8; 20];
    static anix2: i16;
    static mut fp_drawing: *mut FPage;
    static mut seq_list: [SeqInfo; 7];
    static statelist: [StateEntry; 0];
    static planes: *mut *mut u8;
    static bmask_mem: *mut u8;
    static terra_mem: *const u8;
    static mut shp: *mut SShape;
    static mut shapedata: *mut u8;
    static mut mask_buffer: *mut u8;
    static mut shift: i16;
    static mut aoff: i16;
    static mut boff: i16;
    static mut bmod: i16;
    static mut planesize: i16;
    static mut wmask: i16;
    static map_x: u16;
    static map_y: u16;
    static xtype: u16;
    static riding: i16;
    static actor_file: i16;
    static fiery_death: i8;
    static hero_x: u16;
    static hero_y: u16;
    static hero_sector: i16;
    static witchflag: i8;
    static mut wdir: i8;
    static mut witchindex: u8;
    static s1: i16;
    static s2: i16;

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
#[unsafe(no_mangle)]
pub extern "C" fn repair_map_and_witch(dif_x: i16, dif_y: i16) {
    unsafe {
        if dif_x == 1 {
            strip_draw(STRIP_RIGHT);
        } else if dif_x == -1 {
            strip_draw(STRIP_LEFT);
        }
        if dif_y == 1 {
            row_draw(ROW_BOTTOM);
        } else if dif_y == -1 {
            row_draw(ROW_TOP);
        }

        let witch = &raw const anim_list[WITCH_SLOT];
        let page = fp_drawing;
        (*page).witchx = ((*witch).abs_x as i32 - (map_x & 0xfff0) as i32) as i16;
        (*page).witchy = (((*witch).abs_y as i32 - 15) - (map_y & 0xffe0) as i32) as i16;
        if rand4() == 0 {
            wdir = if s1 > 0 { 1 } else { -1 };
        }
        witchindex = witchindex.wrapping_add(wdir as u8);
        (*page).witchdir = witchindex as i16;
        (*page).wflag = witchflag as i16;
        if witchflag != 0 {
            witch_fx(page);
            if s1 > 0 && s2 < 0 && calc_dist(WITCH_SLOT as i32, 0) < WITCH_REACH {
                dohit(-1, 0, (*witch).facing as i32, (rand2() + 1) as i16);
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
/// Terrain flag bits `terra_mem[tile*4+1] & 15` says how a tile hides what is behind it.
const TILE_FULL_MASK: i16 = 64 * 4;
/// Fall animation: tiles at or below this offset do not hide the actor.
const FALL_TILE_LIMIT: i16 = 220 * 4;
/// The bridge sector, where actors other than the raft walk in front of the bridge.
const BRIDGE_SECTOR: i16 = 48;
const RAFT_SLOT: i64 = 1;
/// Races that are never hidden behind scenery.
const RACE_NO_MASK_A: u8 = 0x85;
const RACE_NO_MASK_B: u8 = 0x87;
/// Enemy race 8 (the spectre) and 4 (snakes) look different.
const RACE_SPECTRE: u8 = 8;
const RACE_SNAKE: u8 = 4;
/// The `xtype` of the room where actors are drawn in front of everything.
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
    unsafe { &*(&raw const statelist).cast::<StateEntry>().offset(inum as isize) }
}

fn byte_of(table: &[i8; 32], inum: i16) -> i16 {
    table.get(inum as usize).copied().unwrap_or(0) as i16
}

/// Draws every actor, and its weapon, into the page being drawn, and saves the background under each
/// (`backsave`) so it can be undone next frame. `cycle` is the frame counter.
#[unsafe(no_mangle)]
pub extern "C" fn draw_sprites(cycle: i16) {
    unsafe {
        let page = fp_drawing;
        (*page).obcount = 0;
        (*page).saveused = 0;
        let mut frame = Frame { crack: 0, backalloc: (*page).backsave };
        for j in 0..anix2 {
            let i = anim_index[j as usize] as i64;
            let an = &raw mut anim_list[i as usize];
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
        if (*an).weapon > 0 && (*an).weapon < 8 && (state < DEAD || state >= SHOOT1 || xtype > 80) {
            let facing = (*an).facing as i32;
            let flipped = *pass ^ 1;
            let mut pick = if ((facing - 2) & 4) != 0 { flipped } else { *pass };
            if (*an).weapon == WEAPON_BOW && state < SHOOT1 {
                pick = if (facing & 4) == 0 { flipped } else { *pass };
            }
            passmode = pick != 0;
            *pass += 1;
        }
        passmode
    }
}

/// Draws one actor (or its weapon) for the current pass.
unsafe fn draw_one(frame: &mut Frame, an: *mut Shape, i: i64, cycle: i16, passmode: bool) -> Drawn {
    unsafe {
        let page = fp_drawing;
        let mut inum = (*an).index as i16;
        let mut xstart = (*an).rel_x as i16;
        let mut ystart = (*an).rel_y as i16;
        let mut ground = ystart.wrapping_add(32);
        let mut atype: i16;

        OwnBlitter();
        WaitBlit();
        clear_blit(bmask_mem, MASK_CLEAR_SIZE);
        DisownBlitter();

        if passmode {
            let mut k = (*an).weapon as i16;
            if k == WEAPON_BOW as i16 && inum < BOW_FRAMES {
                xstart += byte_of(&bow_x, inum);
                ystart += byte_of(&bow_y, inum);
            } else {
                let entry = state_entry(inum);
                xstart += entry.wpn_x as i16;
                ystart += entry.wpn_y as i16;
            }
            if k == WEAPON_BOW as i16 && inum < BOW_FRAMES {
                inum /= 8;
                inum = if inum & 1 != 0 {
                    OBJ_BOW_UP
                } else if inum & 2 != 0 {
                    OBJ_BOW_MID
                } else {
                    OBJ_BOW_DOWN
                };
            } else if k == WEAPON_FIREBALL as i16 {
                inum = (*an).facing as i16 + OBJ_ARROW_BASE;
                if (*an).facing == 2 {
                    ystart -= 6;
                }
            } else {
                // hand weapons: each kind has its own block of images
                if let Some(&(_, base)) = WEAPON_SPRITE_BASE.iter().find(|(w, _)| *w as i16 == k) {
                    k = base;
                }
                inum = state_entry(inum).wpn_no as i16 + k;
            }
            atype = OBJECTS;
        } else {
            atype = (*an).kind as i16;
            let mut burning = false;
            if (atype == ENEMY && (*an).race != RACE_SPECTRE) || atype == PHIL {
                if fiery_death != 0 && (*an).environ > 0 {
                    if (*an).state == DEAD {
                        return Drawn::Done;
                    } else if (*an).state == DYING {
                        atype = OBJECTS;
                        inum = OBJ_BURNING;
                        burning = true;
                    }
                } else if (*an).state == FALL {
                    atype = if (*an).tactic < 16 { ENEMY } else { OBJECTS };
                } else {
                    inum = state_entry(inum).figure as i16;
                }
                if !burning {
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
            }
            if !burning {
                if atype == OBJECTS && (*an).race == 0 {
                    return Drawn::Done;
                }
                if atype == CARRIER && riding == 0 && actor_file == 11 {
                    atype = RAFT;
                    inum = 1;
                }
            }
        }

        xstart += (map_x & 15) as i16;
        ystart += (map_y & 31) as i16;
        ground += (map_y & 31) as i16;

        let seq = &*(&raw const seq_list).cast::<SeqInfo>().offset(atype as isize);
        let xsize = (seq.width << 4) as i8;
        let mut ysize = seq.height as i8;

        if atype == OBJECTS {
            let n = inum;
            if n == 0x1b || (8..=12).contains(&n) || n == 25 || n == 26 || (n > 0x10 && n < 0x18) || (n & 128) != 0 {
                ysize = 8;
            }
        }

        let cwide = xsize / 8;
        let xstop = xstart.wrapping_add(xsize as i16).wrapping_sub(1);
        let mut ystop = ystart.wrapping_add(ysize as i16).wrapping_sub(1);

        if !passmode && atype != OBJECTS {
            if i == 0 && riding == 11 {
                ystop -= 16;
            } else if (*an).environ == 2 {
                ystop -= 10;
            } else if (*an).environ > 29 {
                if (*an).state == DEAD {
                    return Drawn::Done;
                }
                ystart += 27;
                ystop = ystart + 7;
                atype = OBJECTS;
                inum = OBJ_SUNK + ((cycle as i64 + i) & 1) as i16;
            } else if (*an).environ > 2 {
                ystart += (*an).environ as i16;
            }
        } else {
            if (*an).environ > 29 {
                return Drawn::Done; // no weapon
            }
            if (*an).environ > 2 {
                ystart += (*an).environ as i16;
                ystop += (*an).environ as i16;
                if ystop > ground {
                    ystop = ground;
                }
                if ystart >= ground {
                    return Drawn::Done;
                }
                ground += (*an).environ as i16;
            }
        }

        if xstart > SCREEN_RIGHT || ystart > SCREEN_BOTTOM || xstop < 0 || ystop < 0 {
            return Drawn::Done;
        }

        // clip the shape to the screen
        let xstart1 = xstart.max(0);
        let ystart1 = ystart.max(0);
        let xstop1 = xstop.min(SCREEN_RIGHT);
        let ystop1 = ystop.min(SCREEN_BOTTOM);
        let xoff = (xstart1 - xstart) as i8;
        let mut yoff = (ystart1 - ystart) as i8;

        if atype == OBJECTS && (inum & 128) != 0 {
            inum &= 0x7f;
            yoff = yoff.wrapping_add(8);
        }

        // word offsets
        let xbw = (xstart1 >> 4) as i8;
        let xew = (xstop1 >> 4) as i8;
        let mut blitwide = (xew as i16 - xbw as i16 + 1) as i8;
        let blithigh = (ystop1 - ystart1 + 1) as i8;
        if xoff & 15 != 0 {
            blitwide = blitwide.wrapping_add(1);
        }

        shp = (*page).shape_queue.cast::<SShape>().offset((*page).obcount as isize);
        planesize = seq.bytes;
        (*shp).savesize = (blitwide as i32 * blithigh as i32 * 2) as i16;
        (*shp).blitsize = (((blithigh as i32) << 6) + blitwide as i32) as i16;

        shapedata = seq.location.offset(planesize as isize * PLANES as isize * inum as isize);
        mask_buffer = seq.maskloc.offset(planesize as isize * inum as isize);
        if (*shp).savesize < SMALL_SAVE && frame.crack < SPARE_SAVES {
            (*shp).backsave = (*planes.add(frame.crack as usize)).add(SPARE_PLANE_OFFSET);
            frame.crack += 1;
        } else {
            (*shp).backsave = frame.backalloc;
            let bytes = PLANES * (*shp).savesize as i32;
            frame.backalloc = frame.backalloc.offset(bytes as isize);
            (*page).saveused += bytes;
            if (*page).saveused >= SAVE_LIMIT {
                return Drawn::OutOfRoom; // no more blits if no room
            }
        }

        aoff = ((ystart1 & 31) as i32 * blitwide as i32 * 2) as i16;
        boff = ((xoff >> 4) as i32 * 2 + cwide as i32 * yoff as i32) as i16;
        (*shp).coff = (xbw as i32 * 2 + ystart1 as i32 * 40) as i16;
        shift = xstart & 15;
        if xoff & 15 != 0 {
            (*shp).coff -= 2;
            wmask = 0;
        } else {
            wmask = -1;
        }
        bmod = (cwide as i32 - blitwide as i32 * 2) as i16;
        (*shp).cmod = (40 - blitwide as i32 * 2) as i16;

        OwnBlitter();
        save_blit((*shp).backsave);

        if !drawn_in_front(an, i, atype, inum) {
            mask_against_scenery(an, i, ground, xbw, ystart1, ystop1, blitwide);
        }

        mask_blit();
        shape_blit();
        (*an).visible = 1;
        DisownBlitter();
        (*page).obcount += 1;
        Drawn::Done
    }
}

/// True for sprites drawn in front of everything: carriers, the hero when riding the swan or in
/// certain places, arrows and some special races.
unsafe fn drawn_in_front(an: *const Shape, i: i64, atype: i16, inum: i16) -> bool {
    unsafe {
        if atype == CARRIER
            || (i == 0
                && (riding == 11
                    || xtype == NO_MASK_XTYPE
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
unsafe fn mask_against_scenery(an: *const Shape, i: i64, ground: i16, xbw: i8, ystart1: i16, ystop1: i16, blitwide: i8) {
    unsafe {
        let ym1 = (ystart1 >> 5) as u8;
        let ym2 = ((ystop1 >> 5) as u8).wrapping_sub(ym1);
        for xm in 0..blitwide.max(0) as i32 {
            for ym in 0..=ym2 as i32 {
                let ystop = ground - (((ym + ym1 as i32) << 5) as i16);
                let mut cm = ((xm + xbw as i32) * 6 + ym + ym1 as i32) as i16;
                cm = (minimap[cm as usize] as i32 * 4) as i16;
                let mut k = (*terra_mem.offset(cm as isize + 1) & 15) as i16;
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
                maskit(xm, ym, blitwide as i32, *terra_mem.offset(cm as isize) as i32);
            }
        }
    }
}
