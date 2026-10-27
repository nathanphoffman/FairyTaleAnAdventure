//! Choosing which picture of the sprite sheet an actor is drawn with, and how high it is.

use super::*;

/// Enemy race 8 (the spectre) and 4 (snakes) look different.
const RACE_SPECTRE: u8 = 8;
const RACE_SNAKE: u8 = 4;
/// Sprite numbers (object images).
const OBJ_BURNING: i16 = 0x58;
const OBJ_SUNK: i16 = 97;

pub(super) unsafe fn state_entry(inum: i16) -> &'static StateEntry {
    // Not range checked, like the C table lookup.
    unsafe {
        &*(&raw const animation_frames)
            .cast::<StateEntry>()
            .offset(inum as isize)
    }
}

pub(super) fn byte_of(table: &[i8; 32], inum: i16) -> i16 {
    table.get(inum as usize).copied().unwrap_or(0) as i16
}

/// Where the images of one kind of sprite are (`seq_list` entry `atype`).
pub(super) unsafe fn sprite_sheet(atype: i16) -> &'static SeqInfo {
    unsafe {
        &*(&raw const the_memory.seq_list)
            .cast::<SeqInfo>()
            .offset(atype as isize)
    }
}

/// Picks the image for this pass: the weapon's (`passmode`) or the actor's own, and where its top
/// left corner is on the screen. `None` means nothing is drawn.
pub(super) unsafe fn choose_image(an: *const Shape, i: i32, passmode: bool) -> Option<Sprite> {
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

/// The actor's own sprite sheet and image for this animation frame. `None` means nothing is drawn
/// (a burnt body, or an object with no image).
unsafe fn body_image(an: *const Shape, i: i32, mut inum: i16) -> Option<(i16, i16)> {
    unsafe {
        let mut atype = (*an).kind as i16;
        if (atype == ENEMY && (*an).race != RACE_SPECTRE) || atype == PHIL {
            if the_actors.fiery_death != 0 && (*an).environ > 0 {
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
        if atype == CARRIER && the_hero.riding == 0 && the_actors.loaded_actor_file == 11 {
            atype = RAFT;
            inum = 1;
        }
        Some((atype, inum))
    }
}

/// The height of an image in lines: the sheet's, except for some small objects.
pub(super) fn image_height(atype: i16, inum: i16, sheet_height: i8) -> i8 {
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
pub(super) unsafe fn adjust_for_depth(
    an: *const Shape,
    i: i32,
    cycle: i16,
    passmode: bool,
    sprite: &mut Sprite,
) -> bool {
    unsafe {
        let depth = (*an).environ as i16;
        if !passmode && sprite.atype != OBJECTS {
            if i == 0 && the_hero.riding == 11 {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn images_with_bit_seven_start_eight_lines_into_the_picture() {
        let mut sprite = sprite_at(OBJECTS, 0x85, 10, 10, 25, 17);
        let clip = clip_to_screen(&mut sprite);
        assert_eq!(sprite.inum, 5);
        assert_eq!(clip.yoff, 8);
    }
}
