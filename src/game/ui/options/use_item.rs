//! The Use menu: picking a weapon, the key menu, the turtle shell and the sunstone, and using a key.

use super::shop::FIRST_MENU_ENTRY;
use super::{ITEMS, KEYBASE, KEYS};
use crate::game::actors::interaction::summon_turtle;
use crate::game::data::state::Game;
use crate::game::data::state::Hero;
use crate::game::data::state::Video;
use crate::game::ui::items_menu::item_name;
use crate::game::ui::menus::go_to_menu;
use crate::game::ui::messages::{print, print_cont, print_wrapped};
use crate::game::ui::text::speak;
use crate::game::world::key_doors::open_door_near;
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
fn in_no_turtle_area(hero: &Hero) -> bool {
    let (x_low, x_high, y_low, y_high) = NO_TURTLE_AREA;

    hero.x < x_high && hero.x > x_low && hero.y < y_high && hero.y > y_low
}

/// Picks a weapon, the key menu, the turtle shell, or the sunstone.
pub(super) fn use_option(game: &mut Game, video: &mut Video, chosen_entry: i16) -> bool {
    if chosen_entry == KEY_ENTRY {
        if game.ui.menu_hit_enabled != 0 {
            go_to_menu(game, video, KEYS as i16);
        } else {
            print_wrapped(game, video, "% has no keys!");
        }

        return false;
    }

    if chosen_entry < WEAPONS {
        if game.ui.menu_hit_enabled != 0 {
            game.actors.list[0].weapon = (chosen_entry + 1) as i8;
        } else {
            print_wrapped(game, video, "% doesn't have one.");
        }
    }

    if chosen_entry == SHELL_ENTRY && game.ui.menu_hit_enabled != 0 {
        if in_no_turtle_area(&game.hero) {
            return true;
        }

        summon_turtle(game, video);
    }

    if chosen_entry == SUNSTONE_ENTRY && game.witch.on_screen != 0 {
        speak(game, video, SUNSTONE_SPEECH);
    }

    go_to_menu(game, video, ITEMS as i16);

    true
}

/// Tries a key on every door and chest around the hero.
pub(super) fn keys_option(game: &mut Game, video: &mut Video, chosen_entry: i16) -> bool {
    let key = (chosen_entry - FIRST_MENU_ENTRY) as usize;
    game.world.bumped_locked_door = 0;

    if game.hero.stuff()[key + KEYBASE] != 0 {
        let mut direction = 0;

        while direction < DIRECTIONS {
            let x = step_x(game.hero.x as i32, direction, DOOR_REACH) as u16;
            let y = step_y(game.hero.y as i32, direction, DOOR_REACH) as u16;

            if open_door_near(game, video, x, y, key as u32 + 1) != 0 {
                game.hero.stuff_mut()[key + KEYBASE] -= 1;
                break;
            }

            direction += 1;
        }

        if direction >= DIRECTIONS {
            print_wrapped(game, video, "% tried a ");
            print_cont(video, item_name(KEYBASE + key));
            print_cont(video, " but it didn't");
            print(video, "fit.");
        }
    }

    go_to_menu(game, video, ITEMS as i16);

    true
}
