//! The Use menu: picking a weapon, the key menu, the turtle shell and the sunstone, and using a key.

use super::shop::FIRST_MENU_ENTRY;
use super::{ITEMS, KEYBASE, KEYS};
use crate::game::actors::interaction::summon_turtle;
use crate::game::data::state::{the_actors, the_hero, the_ui, the_witch};
use crate::game::ui::items_menu::item_name;
use crate::game::ui::menus::go_to_menu;
use crate::game::ui::messages::{print, print_cont, print_wrapped};
use crate::game::ui::text::speak;
use crate::game::world::key_doors::{bumped_locked_door, open_door_near};
use crate::game::world::map::{step_x, step_y};

// Use menu entries.
const KEY_ENTRY: i16 = 7;
const SHELL_ENTRY: i16 = 6;
const SUNSTONE_ENTRY: i16 = 8;
const WEAPONS: i16 = 5;
const SUNSTONE_SPEECH: i32 = 60;

/// No summoning the turtle inside this box (the swamp it lives in is elsewhere).
const NO_TURTLE_AREA: (u16, u16, u16, u16) = (11194, 21373, 10205, 16208);

// Keys: the menu entries start at 5 and the nine directions to try a door in.
const DIRECTIONS: i32 = 9;
const DOOR_REACH: i32 = 16;

/// Whether the hero is in the box where the turtle cannot be called.
unsafe fn in_no_turtle_area() -> bool {
    unsafe {
        let (x_low, x_high, y_low, y_high) = NO_TURTLE_AREA;

        the_hero.x < x_high && the_hero.x > x_low && the_hero.y < y_high && the_hero.y > y_low
    }
}

/// Picks a weapon, the key menu, the turtle shell, or the sunstone.
pub(super) unsafe fn use_option(chosen_entry: i16) -> bool {
    unsafe {
        if chosen_entry == KEY_ENTRY {
            if the_ui.menu_hit_enabled != 0 {
                go_to_menu(KEYS as i16);
            } else {
                print_wrapped(c"% has no keys!".as_ptr());
            }

            return false;
        }

        if chosen_entry < WEAPONS {
            if the_ui.menu_hit_enabled != 0 {
                the_actors.list[0].weapon = (chosen_entry + 1) as i8;
            } else {
                print_wrapped(c"% doesn't have one.".as_ptr());
            }
        }

        if chosen_entry == SHELL_ENTRY && the_ui.menu_hit_enabled != 0 {
            if in_no_turtle_area() {
                return true;
            }

            summon_turtle();
        }

        if chosen_entry == SUNSTONE_ENTRY && the_witch.on_screen != 0 {
            speak(SUNSTONE_SPEECH);
        }

        go_to_menu(ITEMS as i16);

        true
    }
}

/// Tries a key on every door and chest around the hero.
pub(super) unsafe fn keys_option(chosen_entry: i16) -> bool {
    unsafe {
        let key = (chosen_entry - FIRST_MENU_ENTRY) as usize;
        bumped_locked_door = 0;

        if *the_hero.stuff.add(key + KEYBASE) != 0 {
            let mut direction = 0;

            while direction < DIRECTIONS {
                let x = step_x(the_hero.x as i32, direction, DOOR_REACH) as u16;
                let y = step_y(the_hero.y as i32, direction, DOOR_REACH) as u16;

                if open_door_near(x, y, key as u32 + 1) != 0 {
                    *the_hero.stuff.add(key + KEYBASE) -= 1;
                    break;
                }

                direction += 1;
            }

            if direction >= DIRECTIONS {
                print_wrapped(c"% tried a ".as_ptr());
                print_cont(item_name(KEYBASE + key));
                print_cont(c" but it didn't".as_ptr());
                print(c"fit.".as_ptr());
            }
        }

        go_to_menu(ITEMS as i16);

        true
    }
}
