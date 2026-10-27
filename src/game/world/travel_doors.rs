//! Going through the doors between the outdoors and the buildings and caves (from the main loop in
//! csrc/fmain.c). Doors that need a key are in `key_doors`.

use crate::game::data::object_tables::page_colors;
use crate::game::data::state::{the_hero, the_world};
use crate::game::data::tables::door_list;
use crate::game::data::types::Door;
use crate::game::render::screen_fx::fade_page;
use crate::game::world::places::{find_place, transfer_hero};

const DOORCOUNT: i32 = 86;
const DESERT: i8 = 17;
const CAVE: i8 = 18;

/// Inventory slot of the gold statues, five are needed to enter the desert.
const STATBASE: usize = 25;
const STATUES_NEEDED: u8 = 5;
const REGION_INSIDE: u16 = 8;
const REGION_INSIDE_OTHER: u16 = 9;
const FIRST_INSIDE_REGION: u16 = 8;
const FADE_WHITE: i16 = 100;

/// Where to arrive on the other side of a door going in.
fn inside_spot(d: &Door) -> (u16, u16) {
    if d.kind == CAVE {
        (d.xc2.wrapping_add(24), d.yc2.wrapping_add(16))
    } else if d.kind & 1 != 0 {
        (d.xc2.wrapping_add(16), d.yc2)
    } else {
        (d.xc2.wrapping_sub(1), d.yc2.wrapping_add(16))
    }
}

/// Where to arrive going out.
fn outside_spot(d: &Door) -> (u16, u16) {
    if d.kind == CAVE {
        (d.xc1.wrapping_sub(4), d.yc1.wrapping_add(16))
    } else if d.kind & 1 != 0 {
        (d.xc1.wrapping_add(16), d.yc1.wrapping_add(34))
    } else {
        (d.xc1.wrapping_add(20), d.yc1.wrapping_add(16))
    }
}

/// The hero can go in when lined up with the door (horizontal doors by y, vertical ones by x), and
/// the desert door needs the gold statues.
fn may_enter(d: &Door, hero_x: u16, hero_y: u16, statues: u8) -> bool {
    let lined_up = if d.kind & 1 != 0 {
        hero_y & 0x10 == 0
    } else {
        hero_x & 15 <= 6
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

/// Whether a door's inside entrance is where the hero stands (`xtest`, `ytest`: the hero's position
/// rounded to a tile). Horizontal doors are also found one tile to the right.
fn is_exit_here(d: &Door, xtest: u16, ytest: u16) -> bool {
    let horizontal = d.kind & 1 != 0;

    d.yc2 == ytest && (d.xc2 == xtest || (d.xc2 as i32 == xtest as i32 - 16 && horizontal))
}

/// The hero can go out when lined up with the door the other way round to `may_enter`.
fn may_leave(d: &Door, hero_x: u16, hero_y: u16) -> bool {
    if d.kind & 1 != 0 {
        hero_y & 0x10 != 0
    } else {
        hero_x & 15 >= 2
    }
}

unsafe fn door_at(j: i32) -> &'static Door {
    unsafe { &*(&raw const door_list).cast::<Door>().add(j as usize) }
}

/// Is the hero standing at a door? Outdoors the doors are in sorted order and found by binary
/// search; indoors they are searched in turn. If so, the hero is moved to the other side.
/// Riding disables doors.
pub fn check_door() {
    unsafe {
        if the_hero.riding != 0 {
            return;
        }

        let xtest = the_hero.x & 0xfff0;
        let ytest = the_hero.y & 0xffe0;

        if the_world.region < FIRST_INSIDE_REGION {
            enter_door(xtest, ytest);
        } else {
            leave_door(xtest, ytest);
        }
    }
}

unsafe fn enter_door(xtest: u16, ytest: u16) {
    unsafe {
        let Some(index) = find_outdoor_door(xtest, ytest) else {
            return;
        };

        let d = door_at(index);
        let statues = *the_hero.stuff.add(STATBASE);

        if may_enter(d, the_hero.x, the_hero.y, statues) {
            let (x, y) = inside_spot(d);
            the_world.new_region = region_inside(d);
            transfer_hero(x, y, 0);
            find_place(2);
            fade_page(
                FADE_WHITE,
                FADE_WHITE,
                FADE_WHITE,
                1,
                (&raw const page_colors).cast(),
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
            } else if d.xc1 as i32 + 16 < xtest as i32 {
                low = j + 1;
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

/// Indoors: if the hero stands at the inside entrance of a door and is lined up with it, he is moved
/// to the outside.
unsafe fn leave_door(xtest: u16, ytest: u16) {
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
    fn doors_lead_to_two_indoor_regions() {
        assert_eq!(region_inside(&door(1, 1)), REGION_INSIDE);
        assert_eq!(region_inside(&door(1, 2)), REGION_INSIDE_OTHER);
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
