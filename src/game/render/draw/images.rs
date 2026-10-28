//! Choosing which picture of the sprite sheet an actor is drawn with, and how high it is.

use super::weapons::weapon_image;
use super::{CARRIER, DEAD, DYING, ENEMY, FALL, OBJECTS, PHIL, RAFT, Sprite};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Actors;
use crate::game::data::state::Game;
use crate::game::data::state::Hero;
use crate::game::data::tables::{animation_frames, entry};
use crate::game::data::types::{SeqInfo, StateEntry};

/// Enemy race 8 (the spectre) and 4 (snakes) look different.
const RACE_SPECTRE: u8 = 8;
const RACE_SNAKE: u8 = 4;

/// A sprite is 32 pixels from the top of its box to the ground line.
const SPRITE_GROUND_OFFSET: i16 = 32;

/// A falling actor with fewer tactic steps than this still shows as an enemy figure.
const FALL_AS_ENEMY_STEPS: i8 = 16;

/// The first snake picture, and the pairing of left/right facing pictures.
const SNAKE_FIRST_IMAGE: i16 = 0x24;
const IMAGE_PAIR_BIT: i16 = 1;

/// The swan's actor file number (it is drawn as the raft when nobody rides).
const SWAN_FILE: i16 = 11;
const RIDING_SWAN: i16 = 11;
const RAFT_IMAGE: i16 = 1;

/// Objects whose picture is only 8 lines high: 0x1b, 8 to 12, 25, 26, 0x11 to 0x17, and any with
/// bit 7 set.
const SMALL_OBJECT_LINES: i8 = 8;
const SMALL_OBJECT_SINGLES: [i16; 3] = [0x1b, 25, 26];
const SMALL_OBJECT_RANGES: [std::ops::RangeInclusive<i16>; 2] = [8..=12, 0x11..=0x17];
const OBJECT_FLAG_SMALL: i16 = 128;

// Sinking: the hero on the swan sits 16 lines higher; shallow water (depth 2) hides 10 lines;
// beyond 29 the actor is under the surface and only bubbles show, drawn 27 lines down and 7 high.
const SWAN_LIFT: i16 = 16;
const SHALLOW_DEPTH: i16 = 2;
const SHALLOW_HIDDEN_LINES: i16 = 10;
const UNDER_SURFACE_DEPTH: i16 = 29;
const BUBBLES_DROP: i16 = 27;
const BUBBLES_HEIGHT: i16 = 7;

/// Sprite numbers (object images).
const OBJ_BURNING: i16 = 0x58;
const OBJ_SUNK: i16 = 97;

pub(super) fn state_entry(inum: i16) -> &'static StateEntry {
    entry(&animation_frames, inum as usize)
}

pub(super) fn byte_of(table: &[i8; 32], inum: i16) -> i16 {
    table.get(inum as usize).copied().unwrap_or(0) as i16
}

/// Where the images of one kind of sprite are (`seq_list` entry `atype`).
pub(super) fn sprite_sheet(game: &Game, atype: i16) -> &SeqInfo {
    static NO_SHEET: SeqInfo = SeqInfo::EMPTY;

    game.memory
        .seq_list
        .get(atype as usize)
        .unwrap_or(&NO_SHEET)
}

/// Picks the image for this pass: the weapon's (`passmode`) or the actor's own, and where its top
/// left corner is on the screen. `None` means nothing is drawn.
pub(super) fn choose_image(
    actors: &Actors,
    hero: &Hero,
    actor: ActorRef,
    i: i32,
    passmode: bool,
) -> Option<Sprite> {
    let inum = actors.actor(actor).index as i16;
    let xstart = actors.actor(actor).rel_x as i16;
    let ystart = actors.actor(actor).rel_y as i16;
    let ground = ystart.wrapping_add(SPRITE_GROUND_OFFSET);
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
        return Some(weapon_image(actors, actor, blank));
    }

    let (atype, inum) = body_image(actors, hero, actor, i, inum)?;

    Some(Sprite {
        atype,
        inum,
        ..blank
    })
}

/// The actor's own sprite sheet and image for this animation frame. `None` means nothing is drawn
/// (a burnt body, or an object with no image).
fn body_image(
    actors: &Actors,
    hero: &Hero,
    actor: ActorRef,
    i: i32,
    mut inum: i16,
) -> Option<(i16, i16)> {
    let mut atype = actors.actor(actor).kind as i16;

    if (atype == ENEMY && actors.actor(actor).race != RACE_SPECTRE) || atype == PHIL {
        if actors.fiery_death != 0 && actors.actor(actor).environ > 0 {
            if actors.actor(actor).state == DEAD {
                return None;
            } else if actors.actor(actor).state == DYING {
                // a body burning in the lava: nothing more to work out
                return Some((OBJECTS, OBJ_BURNING));
            }
        } else if actors.actor(actor).state == FALL {
            atype = if actors.actor(actor).tactic < FALL_AS_ENEMY_STEPS {
                ENEMY
            } else {
                OBJECTS
            };
        } else {
            inum = state_entry(inum).figure as i16;
        }

        inum = pick_of_pair(actors, actor, i, inum);
    }

    if atype == OBJECTS && actors.actor(actor).race == 0 {
        return None;
    }

    if atype == CARRIER && hero.riding == 0 && actors.loaded_actor_file == SWAN_FILE {
        atype = RAFT;
        inum = RAFT_IMAGE;
    }

    Some((atype, inum))
}

/// Snakes use their own pictures; other enemies have two pictures per figure, picked by the race
/// (the hero, actor 0, has just the one).
fn pick_of_pair(actors: &Actors, actor: ActorRef, i: i32, inum: i16) -> i16 {
    if actors.actor(actor).race == RACE_SNAKE {
        if actors.actor(actor).state < DYING {
            return inum + SNAKE_FIRST_IMAGE;
        }
    } else if i > 0 {
        if actors.actor(actor).race & 1 != 0 {
            return inum | IMAGE_PAIR_BIT;
        }

        return inum & !IMAGE_PAIR_BIT;
    }

    inum
}

/// The height of an image in lines: the sheet's, except for some small objects.
pub(super) fn image_height(atype: i16, inum: i16, sheet_height: i8) -> i8 {
    let small_object = SMALL_OBJECT_SINGLES.contains(&inum)
        || SMALL_OBJECT_RANGES
            .iter()
            .any(|range| range.contains(&inum))
        || (inum & OBJECT_FLAG_SMALL) != 0;

    if atype == OBJECTS && small_object {
        SMALL_OBJECT_LINES
    } else {
        sheet_height
    }
}

/// Sinks the sprite into water, shortens it when the hero rides the swan, and so on. Returns false
/// if nothing is left to draw.
#[allow(clippy::too_many_arguments)] // the sprite being drawn, plus the actors and the hero
pub(super) fn adjust_for_depth(
    actors: &Actors,
    hero: &Hero,
    actor: ActorRef,
    i: i32,
    cycle: i16,
    passmode: bool,
    sprite: &mut Sprite,
) -> bool {
    let depth = actors.actor(actor).environ as i16;

    if !passmode && sprite.atype != OBJECTS {
        sink_body(actors, hero, actor, i, cycle, depth, sprite)
    } else {
        sink_item(depth, sprite)
    }
}

/// A body standing in water: the raft and the swan lift it, shallow water hides the legs, and deep
/// water shows only bubbles. Returns false if there is nothing to draw (a dead body under water).
#[allow(clippy::too_many_arguments)] // the sprite being drawn, plus the actors and the hero
fn sink_body(
    actors: &Actors,
    hero: &Hero,
    actor: ActorRef,
    i: i32,
    cycle: i16,
    depth: i16,
    sprite: &mut Sprite,
) -> bool {
    if i == 0 && hero.riding == RIDING_SWAN {
        sprite.ystop -= SWAN_LIFT;
    } else if depth == SHALLOW_DEPTH {
        sprite.ystop -= SHALLOW_HIDDEN_LINES;
    } else if depth > UNDER_SURFACE_DEPTH {
        if actors.actor(actor).state == DEAD {
            return false;
        }

        // under the surface: only the bubbles show
        sprite.ystart += BUBBLES_DROP;
        sprite.ystop = sprite.ystart + BUBBLES_HEIGHT;
        sprite.atype = OBJECTS;

        sprite.inum = OBJ_SUNK + ((cycle as i32 + i) & 1) as i16;
    } else if depth > SHALLOW_DEPTH {
        sprite.ystart += depth;
    }

    true
}

/// A weapon or object in water goes down with the water; under the surface it is not drawn at all.
fn sink_item(depth: i16, sprite: &mut Sprite) -> bool {
    if depth > UNDER_SURFACE_DEPTH {
        return false; // no weapon
    }

    if depth > SHALLOW_DEPTH {
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
