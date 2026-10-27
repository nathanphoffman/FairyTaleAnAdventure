//! Going out through a door from inside: finding the door the hero stands in and where to arrive.

use super::entering::door_at;
use super::{CAVE, DOORCOUNT, LOWER_HALF_BIT, TILE_COLUMN_MASK, TILE_WIDTH};
use crate::game::data::state::the_hero;
use crate::game::data::types::Door;
use crate::game::world::places::{find_place, transfer_hero};

const CAVE_OUT: (u16, u16) = (4, 16);
const HORIZONTAL_OUT: (u16, u16) = (16, 34);
const VERTICAL_OUT: (u16, u16) = (20, 16);
const OUT_COLUMN_LIMIT: u16 = 2;

/// Where to arrive going out.
fn outside_spot(d: &Door) -> (u16, u16) {
    if d.kind == CAVE {
        (
            d.xc1.wrapping_sub(CAVE_OUT.0),
            d.yc1.wrapping_add(CAVE_OUT.1),
        )
    // Masks off the low bits (a cheap remainder or a flag test).
    } else if d.kind & 1 != 0 {
        (
            d.xc1.wrapping_add(HORIZONTAL_OUT.0),
            d.yc1.wrapping_add(HORIZONTAL_OUT.1),
        )
    } else {
        (
            d.xc1.wrapping_add(VERTICAL_OUT.0),
            d.yc1.wrapping_add(VERTICAL_OUT.1),
        )
    }
}

/// Whether a door's inside entrance is where the hero stands (`xtest`, `ytest`: the hero's position
/// rounded to a tile). Horizontal doors are also found one tile to the right.
fn is_exit_here(d: &Door, xtest: u16, ytest: u16) -> bool {
    // Masks off the low bits (a cheap remainder or a flag test).
    let horizontal = d.kind & 1 != 0;

    d.yc2 == ytest && (d.xc2 == xtest || (d.xc2 as i32 == xtest as i32 - TILE_WIDTH && horizontal))
}

/// The hero can go out when lined up with the door the other way round to `may_enter`.
fn may_leave(d: &Door, hero_x: u16, hero_y: u16) -> bool {
    // Masks off the low bits (a cheap remainder or a flag test).
    if d.kind & 1 != 0 {
        // Masks with LOWER_HALF_BIT, keeping only the bits that matter here.
        hero_y & LOWER_HALF_BIT != 0
    } else {
        // Masks with TILE_COLUMN_MASK, keeping only the bits that matter here.
        hero_x & TILE_COLUMN_MASK >= OUT_COLUMN_LIMIT
    }
}

/// Indoors: if the hero stands at the inside entrance of a door and is lined up with it, he is moved
/// to the outside.
pub(super) unsafe fn leave_door(xtest: u16, ytest: u16) {
    unsafe {
        for j in 0..DOORCOUNT {
            let d = door_at(j);

            if is_exit_here(d, xtest, ytest) {
                if may_leave(d, the_hero.x, the_hero.y) {
                    let (x, y) = outside_spot(d);
                    transfer_hero(x, y, 1);
                    find_place(0);
                }

                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::CAVE;
    use super::super::entering::inside_spot;
    use super::*;
    use crate::game::data::types::Door;

    fn door(kind: i8, secs: i8) -> Door {
        Door {
            xc1: 100,
            yc1: 200,
            xc2: 300,
            yc2: 400,
            kind,
            secs,
        }
    }

    #[test]
    fn leaving_is_the_mirror_of_entering() {
        assert!(may_leave(&door(1, 1), 0, 0x10));
        assert!(!may_leave(&door(1, 1), 0, 0));
        assert!(may_leave(&door(2, 1), 2, 0));
        assert!(!may_leave(&door(2, 1), 1, 0));
    }

    #[test]
    fn exits_are_found_by_tile() {
        assert!(is_exit_here(&door(2, 1), 300, 400));
        assert!(!is_exit_here(&door(2, 1), 316, 400));
        assert!(is_exit_here(&door(1, 1), 316, 400)); // horizontal: also one tile right
        assert!(!is_exit_here(&door(1, 1), 300, 416));
    }

    #[test]
    fn arrival_spots_depend_on_the_door_kind() {
        assert_eq!(inside_spot(&door(CAVE, 1)), (324, 416));
        assert_eq!(inside_spot(&door(1, 1)), (316, 400));
        assert_eq!(inside_spot(&door(2, 1)), (299, 416));
        assert_eq!(outside_spot(&door(CAVE, 1)), (96, 216));
        assert_eq!(outside_spot(&door(1, 1)), (116, 234));
        assert_eq!(outside_spot(&door(2, 1)), (120, 216));
    }
}
