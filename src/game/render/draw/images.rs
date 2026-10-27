//! Choosing which picture of the sprite sheet an actor is drawn with, and how high it is.

use super::weapons::weapon_image;
use super::{CARRIER, DEAD, DYING, ENEMY, FALL, OBJECTS, PHIL, RAFT, Sprite};
use crate::game::data::state::{the_actors, the_hero, the_memory};
use crate::game::data::tables::animation_frames;
use crate::game::data::types::{SeqInfo, Shape, StateEntry};

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
pub(super) unsafe fn choose_image(actor: *const Shape, i: i32, passmode: bool) -> Option<Sprite> {
    unsafe {
        let inum = (*actor).index as i16;
        let xstart = (*actor).rel_x as i16;
        let ystart = (*actor).rel_y as i16;
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
            return Some(weapon_image(actor, blank));
        }

        let (atype, inum) = body_image(actor, i, inum)?;

        Some(Sprite {
            atype,
            inum,
            ..blank
        })
    }
}

/// The actor's own sprite sheet and image for this animation frame. `None` means nothing is drawn
/// (a burnt body, or an object with no image).
unsafe fn body_image(actor: *const Shape, i: i32, mut inum: i16) -> Option<(i16, i16)> {
    unsafe {
        let mut atype = (*actor).kind as i16;

        if (atype == ENEMY && (*actor).race != RACE_SPECTRE) || atype == PHIL {
            if the_actors.fiery_death != 0 && (*actor).environ > 0 {
                if (*actor).state == DEAD {
                    return None;
                } else if (*actor).state == DYING {
                    // a body burning in the lava: nothing more to work out
                    return Some((OBJECTS, OBJ_BURNING));
                }
            } else if (*actor).state == FALL {
                atype = if (*actor).tactic < 16 { ENEMY } else { OBJECTS };
            } else {
                inum = state_entry(inum).figure as i16;
            }

            if (*actor).race == RACE_SNAKE {
                if (*actor).state < DYING {
                    inum += 0x24;
                }
            } else if i > 0 {
                if (*actor).race & 1 != 0 {
                    inum |= 1;
                } else {
                    inum &= 0xfffeu16 as i16;
                }
            }
        }

        if atype == OBJECTS && (*actor).race == 0 {
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
    actor: *const Shape,
    i: i32,
    cycle: i16,
    passmode: bool,
    sprite: &mut Sprite,
) -> bool {
    unsafe {
        let depth = (*actor).environ as i16;

        if !passmode && sprite.atype != OBJECTS {
            sink_body(actor, i, cycle, depth, sprite)
        } else {
            sink_weapon_or_object(depth, sprite)
        }
    }
}

/// A body standing in water: the raft and the swan lift it, shallow water hides the legs, and deep
/// water shows only bubbles. Returns false if there is nothing to draw (a dead body under water).
unsafe fn sink_body(
    actor: *const Shape,
    i: i32,
    cycle: i16,
    depth: i16,
    sprite: &mut Sprite,
) -> bool {
    unsafe {
        if i == 0 && the_hero.riding == 11 {
            sprite.ystop -= 16;
        } else if depth == 2 {
            sprite.ystop -= 10;
        } else if depth > 29 {
            if (*actor).state == DEAD {
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

        true
    }
}

/// A weapon or object in water goes down with the water; under the surface it is not drawn at all.
fn sink_weapon_or_object(depth: i16, sprite: &mut Sprite) -> bool {
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

    true
}

#[cfg(test)]
mod tests {
    use super::super::ENEMY;
    use super::super::OBJECTS;
    use super::super::blit_setup::clip_to_screen;
    use super::super::sprite_at;
    use super::*;
    use crate::game::data::object_tables::bow_x;
    use crate::game::data::object_tables::bow_y;

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
