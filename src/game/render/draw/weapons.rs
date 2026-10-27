//! Drawing the weapon an actor holds: which pass it is drawn in and which picture.

use super::*;

const OBJ_BOW_UP: i16 = 30;
const OBJ_BOW_MID: i16 = 0x53;
const OBJ_BOW_DOWN: i16 = 0x51;
const OBJ_ARROW_BASE: i16 = 103;
const WEAPON_BOW: i8 = 4;
const WEAPON_FIREBALL: i8 = 5;
/// The first frame of the bow's own drawing table; below this frames come from `bow_x`/`bow_y`.
const BOW_FRAMES: i16 = 32;
const WEAPON_SPRITE_BASE: [(i8, i16); 4] = [(2, 32), (3, 48), (1, 64), (4, 0)];

/// Says whether this pass draws the weapon (`true`) or the body. An actor with a weapon is drawn in
/// two passes, the weapon first or second depending on which way it faces.
pub(super) unsafe fn weapon_passmode(an: *const Shape, pass: &mut u8) -> bool {
    unsafe {
        let mut passmode = false;
        let state = (*an).state;
        if (*an).weapon > 0
            && (*an).weapon < 8
            && (!(DEAD..SHOOT1).contains(&state) || the_world.extent_type > 80)
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

/// The weapon's sprite: where it is held for this animation frame, and which image it is.
pub(super) unsafe fn weapon_image(an: *const Shape, mut sprite: Sprite) -> Sprite {
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
