//! What is inside a chest, urn or sacks: nothing, an item, two things, three keys or three of one item.

use super::*;

/// The 100 gold pieces entry of `item_list`.
const HUNDRED_GOLD: i32 = GOLDBASE as i32 + 3;
const CONTAINER_GOLD: i16 = 100;

/// The item a container holds when it rolls `roll` (0-7) for one item: slot 8 means arrows.
fn container_item(roll: i32) -> usize {
    let i = roll as usize + 8;
    if i == 8 { ARROWBASE } else { i }
}

/// A key found in a container: the two odd key types fold onto the first of their kind.
fn container_key(roll: i32) -> i32 {
    match roll + KEYBASE {
        22 => 16,
        23 => 20,
        key => key,
    }
}

/// What was inside a container. Returns true (the container is always used up).
pub(super) unsafe fn container_contents() -> bool {
    unsafe {
        match roll_contents(rand4, rand8) {
            Contents::Nothing => print(c"nothing.".as_ptr()),
            Contents::One(i) => {
                print(c"a ".as_ptr());
                print_cont(item_name(i));
                print_cont(c".".as_ptr());
                add_stuff(i, 1);
            }
            Contents::Two { first, second } => {
                if first == HUNDRED_GOLD as usize {
                    the_hero.wealth += CONTAINER_GOLD;
                } else {
                    print_cont(c" a".as_ptr());
                }
                print(item_name(first));
                print_cont(c" and a ".as_ptr());
                print_cont(item_name(second));
                if first < GOLDBASE {
                    add_stuff(first, 1);
                }
                add_stuff(second, 1);
            }
            Contents::ThreeKeys(keys) => {
                print(c"3 keys.".as_ptr());
                for key in keys {
                    add_stuff(key as usize, 1);
                }
            }
            Contents::ThreeOf(i) => {
                print(c"3 ".as_ptr());
                print_cont(item_name(i));
                print_cont(c"s.".as_ptr());
                add_stuff(i, 3);
            }
        }
        true
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

/// Rolls the contents with the game's random draws (`pick4` 0-3, `pick8` 0-7), in the same order
/// as the original: the kind first, then the item rolls.
fn roll_contents(mut pick4: impl FnMut() -> i32, mut pick8: impl FnMut() -> i32) -> Contents {
    match pick4() {
        0 => Contents::Nothing,
        1 => Contents::One(container_item(pick8())),
        2 => {
            let mut first = pick8() + 8;
            if first == 8 {
                first = HUNDRED_GOLD;
            }
            let mut second;
            loop {
                second = pick8() + 8;
                if second != first {
                    break;
                }
            }
            if second == 8 {
                second = ARROWBASE as i32;
            }
            Contents::Two {
                first: first as usize,
                second: second as usize,
            }
        }
        _ => {
            let i = pick8() + 8;
            if i == 8 {
                Contents::ThreeKeys([0; 3].map(|_| container_key(pick8())))
            } else {
                Contents::ThreeOf(i as usize)
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
        // two things: a first item and a different second one
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
        // three of one thing, or three keys
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
