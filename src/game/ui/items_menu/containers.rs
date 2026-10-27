//! What is inside a chest, urn or sacks: nothing, an item, two things, three keys or three of one item.

use super::{ARROWBASE, GOLDBASE, KEYBASE, add_stuff, item_name};
use crate::game::data::random::{rand4, rand8};
use crate::game::data::state::the_hero;
use crate::game::ui::messages::{print, print_cont};

/// The container items start at this entry of `item_list`; a roll of 0 there means arrows (or 100
/// gold when it is the first of two things).
const FIRST_CONTAINER_ITEM: i32 = 8;
const ROLL_ARROWS_OR_GOLD: i32 = FIRST_CONTAINER_ITEM;

/// The two key types that fold onto an earlier one, and what they fold onto.
const ODD_KEYS: [(i32, i32); 2] = [(22, 16), (23, 20)];

/// How many of one item, or keys, a container with several holds.
const MANY: usize = 3;

/// The 100 gold pieces entry of `item_list`.
const HUNDRED_GOLD: i32 = GOLDBASE as i32 + 3;
const CONTAINER_GOLD: i16 = 100;

/// The item a container holds when it rolls `roll` (0-7) for one item: slot 8 means arrows.
fn container_item(roll: i32) -> usize {
    let i = roll + FIRST_CONTAINER_ITEM;

    if i == ROLL_ARROWS_OR_GOLD {
        ARROWBASE
    } else {
        i as usize
    }
}

/// A key found in a container: the two odd key types fold onto the first of their kind.
fn container_key(roll: i32) -> i32 {
    let key = roll + KEYBASE;

    ODD_KEYS
        .iter()
        .find(|(odd, _)| *odd == key)
        .map_or(key, |(_, folded)| *folded)
}

/// What was inside a container. Returns true (the container is always used up).
pub(super) unsafe fn container_contents() -> bool {
    unsafe {
        match roll_contents(rand4, rand8) {
            Contents::Nothing => print("nothing."),
            Contents::One(i) => {
                print("a ");
                print_cont(item_name(i));
                print_cont(".");
                add_stuff(i, 1);
            }

            Contents::Two { first, second } => take_two_things(first, second),
            Contents::ThreeKeys(keys) => {
                print("3 keys.");

                for key in keys {
                    add_stuff(key as usize, 1);
                }
            }

            Contents::ThreeOf(i) => {
                print("3 ");
                print_cont(item_name(i));
                print_cont("s.");
                add_stuff(i, MANY as u8);
            }
        }

        true
    }
}

/// Tells what two things were found and adds them (the first may be 100 gold pieces).
unsafe fn take_two_things(first: usize, second: usize) {
    unsafe {
        if first == HUNDRED_GOLD as usize {
            the_hero.wealth += CONTAINER_GOLD;
        } else {
            print_cont(" a");
        }

        print(item_name(first));
        print_cont(" and a ");
        print_cont(item_name(second));

        if first < GOLDBASE {
            add_stuff(first, 1);
        }

        add_stuff(second, 1);
    }
}

/// What a container holds: nothing, an item, two things (the first may be 100 gold pieces), three
/// keys, or three of one item.
#[derive(Debug, PartialEq, Eq)]
enum Contents {
    Nothing,
    One(usize),
    Two { first: usize, second: usize },
    ThreeKeys([i32; 3]),
    ThreeOf(usize),
}

/// Two different things: the first may be 100 gold pieces (a roll of 0), the second is rolled again
/// until it differs and may be arrows (a roll of 0).
fn roll_two_things(pick8: &mut impl FnMut() -> i32) -> Contents {
    let mut first = pick8() + FIRST_CONTAINER_ITEM;

    if first == ROLL_ARROWS_OR_GOLD {
        first = HUNDRED_GOLD;
    }

    let mut second;

    loop {
        second = pick8() + FIRST_CONTAINER_ITEM;

        if second != first {
            break;
        }
    }

    if second == ROLL_ARROWS_OR_GOLD {
        second = ARROWBASE as i32;
    }

    Contents::Two {
        first: first as usize,
        second: second as usize,
    }
}

/// Rolls the contents with the game's random draws (`pick4` 0-3, `pick8` 0-7), in the same order
/// as the original: the kind first, then the item rolls.
fn roll_contents(mut pick4: impl FnMut() -> i32, mut pick8: impl FnMut() -> i32) -> Contents {
    match pick4() {
        0 => Contents::Nothing,
        1 => Contents::One(container_item(pick8())),
        2 => roll_two_things(&mut pick8),
        _ => {
            let i = pick8() + FIRST_CONTAINER_ITEM;

            if i == ROLL_ARROWS_OR_GOLD {
                Contents::ThreeKeys([0; MANY].map(|_| container_key(pick8())))
            } else {
                Contents::ThreeOf(i as usize)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ARROWBASE;
    use super::*;

    #[test]
    fn container_contents_follow_the_rolls() {
        let rolls = |kind: i32, items: &[i32]| {
            let mut next = items.iter().copied();

            roll_contents(|| kind, || next.next().unwrap())
        };

        assert_eq!(rolls(0, &[]), Contents::Nothing);
        assert_eq!(rolls(1, &[0]), Contents::One(ARROWBASE));
        assert_eq!(rolls(1, &[3]), Contents::One(11));
    }

    #[test]
    fn two_things_are_never_the_same() {
        let rolls = |kind: i32, items: &[i32]| {
            let mut next = items.iter().copied();

            roll_contents(|| kind, || next.next().unwrap())
        };

        // a first item and a different second one
        assert_eq!(
            rolls(2, &[2, 5]),
            Contents::Two {
                first: 10,
                second: 13
            }
        );

        // 100 gold first can never match the second roll, so the arrows are accepted
        assert_eq!(
            rolls(2, &[0, 0]),
            Contents::Two {
                first: HUNDRED_GOLD as usize,
                second: ARROWBASE
            }
        );

        assert_eq!(
            rolls(2, &[0, 4]),
            Contents::Two {
                first: HUNDRED_GOLD as usize,
                second: 12
            }
        );

        assert_eq!(
            rolls(2, &[3, 3, 0]),
            Contents::Two {
                first: 11,
                second: ARROWBASE
            }
        );
    }

    #[test]
    fn three_of_one_thing_or_three_keys() {
        let rolls = |kind: i32, items: &[i32]| {
            let mut next = items.iter().copied();

            roll_contents(|| kind, || next.next().unwrap())
        };

        assert_eq!(rolls(3, &[4]), Contents::ThreeOf(12));
        assert_eq!(rolls(3, &[0, 0, 5, 6]), Contents::ThreeKeys([16, 21, 16]));
    }

    #[test]
    fn container_rolls() {
        assert_eq!(container_item(0), ARROWBASE);
        assert_eq!(container_item(1), 9);
        assert_eq!(container_item(7), 15);
        assert_eq!(container_key(0), 16);
        assert_eq!(container_key(5), 21);
        assert_eq!(container_key(6), 16);
        assert_eq!(container_key(7), 20);
    }
}
