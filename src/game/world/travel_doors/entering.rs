//! Going in through a door from outdoors: finding the door and where to arrive.

use super::{CAVE, DOORCOUNT, LOWER_HALF_BIT, TILE_COLUMN_MASK, TILE_WIDTH};
use crate::game::data::object_tables::page_colors;
use crate::game::data::state::Game;
use crate::game::data::state::the_world;
use crate::game::data::tables::door_list;
use crate::game::data::types::Door;
use crate::game::render::screen_fx::fade_page;
use crate::game::world::places::{find_place, transfer_hero};

const DESERT: i8 = 17;

/// Inventory slot of the gold statues, five are needed to enter the desert.
const STATBASE: usize = 25;
const STATUES_NEEDED: u8 = 5;
const REGION_INSIDE: u16 = 8;
const REGION_INSIDE_OTHER: u16 = 9;
const FADE_WHITE: i16 = 100;

// Arrival offsets from a door's corner (pixels): in through a cave door, a horizontal door and a
// vertical door, then out through each.
const CAVE_IN: (u16, u16) = (24, 16);
const HORIZONTAL_IN: (u16, u16) = (16, 0);
const VERTICAL_IN: (u16, u16) = (1, 16);
const IN_COLUMN_LIMIT: u16 = 6;

/// Where to arrive on the other side of a door going in.
pub(super) fn inside_spot(d: &Door) -> (u16, u16) {
    if d.kind == CAVE {
        (d.xc2.wrapping_add(CAVE_IN.0), d.yc2.wrapping_add(CAVE_IN.1))
    // Masks off the low bits (a cheap remainder or a flag test).
    } else if d.kind & 1 != 0 {
        (
            d.xc2.wrapping_add(HORIZONTAL_IN.0),
            d.yc2.wrapping_add(HORIZONTAL_IN.1),
        )
    } else {
        (
            d.xc2.wrapping_sub(VERTICAL_IN.0),
            d.yc2.wrapping_add(VERTICAL_IN.1),
        )
    }
}

/// The hero can go in when lined up with the door (horizontal doors by y, vertical ones by x), and
/// the desert door needs the gold statues.
pub(super) fn may_enter(d: &Door, hero_x: u16, hero_y: u16, statues: u8) -> bool {
    // Masks off the low bits (a cheap remainder or a flag test).
    let lined_up = if d.kind & 1 != 0 {
        // Masks with LOWER_HALF_BIT, keeping only the bits that matter here.
        hero_y & LOWER_HALF_BIT == 0
    } else {
        // Masks with TILE_COLUMN_MASK, keeping only the bits that matter here.
        hero_x & TILE_COLUMN_MASK <= IN_COLUMN_LIMIT
    };

    lined_up && (d.kind != DESERT || statues >= STATUES_NEEDED)
}

/// Which indoor region a door leads to.
fn region_inside(d: &Door) -> u16 {
    if d.secs == 1 {
        REGION_INSIDE
    } else {
        REGION_INSIDE_OTHER
    }
}

pub(super) unsafe fn door_at(j: i32) -> &'static Door {
    unsafe { &*(&raw const door_list).cast::<Door>().add(j as usize) }
}

pub(super) unsafe fn enter_door(game: &mut Game, xtest: u16, ytest: u16) {
    unsafe {
        let Some(index) = find_outdoor_door(xtest, ytest) else {
            return;
        };

        let d = door_at(index);
        let statues = game.hero.stuff()[STATBASE];

        if may_enter(d, game.hero.x, game.hero.y, statues) {
            let (x, y) = inside_spot(d);
            the_world.new_region = region_inside(d);
            transfer_hero(game, x, y, 0);
            find_place(game, 2);
            fade_page(
                game,
                FADE_WHITE,
                FADE_WHITE,
                FADE_WHITE,
                1,
                &*(&raw const page_colors),
            );
        }
    }
}

/// Binary search of the (sorted) outdoor doors for the one whose entrance covers the point.
/// A door matches within 16 pixels to the right of its x; vertical doors need the exact x.
unsafe fn find_outdoor_door(xtest: u16, ytest: u16) -> Option<i32> {
    unsafe {
        let mut low = 0;
        let mut high = DOORCOUNT - 1;

        while high >= low {
            let j = (high + low) / 2;
            let d = door_at(j);

            if d.xc1 > xtest {
                high = j - 1;
            } else if d.xc1 as i32 + TILE_WIDTH < xtest as i32 {
                low = j + 1;

            // Masks off the low bits (a cheap remainder or a flag test).
            } else if d.xc1 < xtest && d.kind & 1 == 0 {
                low = j + 1; // vertical door: its x must match exactly
            } else if d.yc1 > ytest {
                high = j - 1;
            } else if d.yc1 < ytest {
                low = j + 1;
            } else {
                return Some(j);
            }

            if low >= DOORCOUNT || high < 0 {
                break;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn entering_needs_the_hero_lined_up() {
        // horizontal doors (odd kind) go by y, vertical ones by x
        assert!(may_enter(&door(1, 1), 0, 0x00, 0));
        assert!(!may_enter(&door(1, 1), 0, 0x10, 0));
        assert!(may_enter(&door(2, 1), 6, 0, 0));
        assert!(!may_enter(&door(2, 1), 7, 0, 0));
    }

    #[test]
    fn the_desert_door_needs_five_statues() {
        assert!(!may_enter(&door(DESERT, 1), 0, 0, STATUES_NEEDED - 1));
        assert!(may_enter(&door(DESERT, 1), 0, 0, STATUES_NEEDED));
        assert!(may_enter(&door(2, 1), 0, 0, 0));
    }

    #[test]
    fn doors_lead_to_two_indoor_regions() {
        assert_eq!(region_inside(&door(1, 1)), REGION_INSIDE);
        assert_eq!(region_inside(&door(1, 2)), REGION_INSIDE_OTHER);
    }

    /// Doors whose entrance the binary search cannot reach, because a neighbouring door's x range
    /// (16 pixels) overlaps theirs. This is how the original data behaves; they are listed so a
    /// change to the search is noticed.
    const UNREACHABLE: &[i32] = &[38, 67];

    #[test]
    fn every_other_door_is_found_at_its_own_entrance() {
        unsafe {
            for j in (0..DOORCOUNT).filter(|j| !UNREACHABLE.contains(j)) {
                let d = door_at(j);
                let found = find_outdoor_door(d.xc1, d.yc1)
                    .unwrap_or_else(|| panic!("door {j} at {},{} not found", d.xc1, d.yc1));
                let f = door_at(found);
                assert_eq!((f.xc1, f.yc1), (d.xc1, d.yc1));
            }

            for j in UNREACHABLE {
                let d = door_at(*j);
                assert_eq!(find_outdoor_door(d.xc1, d.yc1), None);
            }
        }
    }

    #[test]
    fn empty_ground_has_no_door() {
        unsafe {
            assert_eq!(find_outdoor_door(1, 1), None);
        }
    }
}
