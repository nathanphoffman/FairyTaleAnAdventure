//! Drawing the weapon an actor holds: which pass it is drawn in and which picture.

use super::images::{byte_of, state_entry};
use super::{DEAD, OBJECTS, SHOOT1, Sprite};
use crate::game::data::object_tables::{bow_x, bow_y};
use crate::game::data::state::the_world;
use crate::game::data::types::Shape;

const OBJ_BOW_UP: i16 = 30;
const OBJ_BOW_MID: i16 = 0x53;
const OBJ_BOW_DOWN: i16 = 0x51;
const OBJ_ARROW_BASE: i16 = 103;
const WEAPON_BOW: i8 = 4;

/// Weapon numbers from 1 to 7 are shown held; the others (the wand and missiles) are not.
const LAST_HELD_WEAPON: i8 = 7;

/// Bit 2 of the facing (and of the facing shifted by 2) says whether the weapon is drawn before or
/// after the body.
const FACING_LEFT_BIT: i32 = 4;
const FACING_PICK_OFFSET: i32 = 2;

/// Extents whose type is above this are peaceful, and the weapon is drawn even when resting.
const PEACEFUL_XTYPE: u16 = 80;

/// The bow's drawn image changes every 8 frames: bit 0 up, bit 1 mid, else down.
const BOW_PHASE_FRAMES: i16 = 8;

/// A fireball facing east (2) is drawn this many lines higher.
const FIREBALL_FACING_EAST: i8 = 2;
const FIREBALL_EAST_LIFT: i16 = 6;
const WEAPON_FIREBALL: i8 = 5;

/// The first frame of the bow's own drawing table; below this frames come from `bow_x`/`bow_y`.
const BOW_FRAMES: i16 = 32;
const WEAPON_SPRITE_BASE: [(i8, i16); 4] = [(2, 32), (3, 48), (1, 64), (4, 0)];

/// Says whether this pass draws the weapon (`true`) or the body. An actor with a weapon is drawn in
/// two passes, the weapon first or second depending on which way it faces.
pub(super) unsafe fn weapon_passmode(actor: *const Shape, pass: &mut u8) -> bool {
    unsafe {
        let mut passmode = false;
        let state = (*actor).state;

        if (*actor).weapon > 0
            && (*actor).weapon <= LAST_HELD_WEAPON
            && (!(DEAD..SHOOT1).contains(&state) || the_world.extent_type > PEACEFUL_XTYPE)
        {
            let facing = (*actor).facing as i32;
            // The other of the two passes (0 becomes 1, 1 becomes 0).
            let flipped = *pass ^ 1;
            // Masks with FACING_PICK_OFFSET, keeping only the bits that matter here.
            let mut pick = if ((facing - FACING_PICK_OFFSET) & FACING_LEFT_BIT) != 0 {
                flipped
            } else {
                *pass
            };

            if (*actor).weapon == WEAPON_BOW && state < SHOOT1 {
                // Masks with FACING_LEFT_BIT, keeping only the bits that matter here.
                pick = if (facing & FACING_LEFT_BIT) == 0 {
                    flipped
                } else {
                    *pass
                };
            }

            passmode = pick != 0;
            *pass += 1;
        }

        passmode
    }
}

/// The picture of the bow for an animation frame: it is drawn up, middle or down as the frames go.
fn bow_image(frame_number: i16) -> i16 {
    let bow_phase = frame_number / BOW_PHASE_FRAMES;

    // Masks off the low bits (a cheap remainder or a flag test).
    if bow_phase & 1 != 0 {
        OBJ_BOW_UP
    // Masks off the low bits (a cheap remainder or a flag test).
    } else if bow_phase & 2 != 0 {
        OBJ_BOW_MID
    } else {
        OBJ_BOW_DOWN
    }
}

/// The weapon's sprite: where it is held for this animation frame, and which image it is.
pub(super) unsafe fn weapon_image(actor: *const Shape, mut sprite: Sprite) -> Sprite {
    unsafe {
        let kind = (*actor).weapon as i16;
        let frame_number = sprite.inum;

        if kind == WEAPON_BOW as i16 && frame_number < BOW_FRAMES {
            sprite.xstart += byte_of(&bow_x, frame_number);
            sprite.ystart += byte_of(&bow_y, frame_number);
            sprite.inum = bow_image(frame_number);
        } else {
            let entry = state_entry(frame_number);
            sprite.xstart += entry.wpn_x as i16;
            sprite.ystart += entry.wpn_y as i16;

            if kind == WEAPON_FIREBALL as i16 {
                sprite.inum = (*actor).facing as i16 + OBJ_ARROW_BASE;

                if (*actor).facing == FIREBALL_FACING_EAST {
                    sprite.ystart -= FIREBALL_EAST_LIFT;
                }
            } else {
                sprite.inum = entry.wpn_no as i16 + hand_weapon_images(kind);
            }
        }

        sprite.atype = OBJECTS;

        sprite
    }
}

/// Hand weapons each have their own block of images; the first image of the block for `kind`.
fn hand_weapon_images(kind: i16) -> i16 {
    WEAPON_SPRITE_BASE
        .iter()
        .find(|(w, _)| *w as i16 == kind)
        .map_or(kind, |&(_, base)| base)
}
