//! The debug keys: cheat items and moving the hero around the map.

use super::{
    CARRIER_BIRD, CARRIER_OFFSET, CHEAT_CARRIER, CHEAT_EAST, CHEAT_NORTH, CHEAT_PRQ_2, CHEAT_PRQ_3,
    CHEAT_RESCUE, CHEAT_SOUTH, CHEAT_STEP_X, CHEAT_STEP_Y, CHEAT_STUFF, CHEAT_TIME,
    CHEAT_TIME_STEP, CHEAT_WEST, GOLDBASE, RAFT_STONE_SLOT, VIAL_SLOT,
};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::data::state::player;

use crate::game::actors::encounters::load_carrier;
use crate::game::actors::interaction::{center_extent_on, rescue_princess};
use crate::game::data::random::rnd;
use crate::game::data::state::{the_actors, the_hero, the_world};
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::text::queue_print;

/// Debug keys that give the hero a carrier or items. Returns true if the key was one.
pub(super) unsafe fn cheat_items(game: &mut Game, key: i8) -> bool {
    unsafe {
        if game.ui.cheat_enabled == 0 {
            return false;
        }

        if key == CHEAT_CARRIER {
            if the_actors.active_carrier == CARRIER_BIRD {
                player().stuff_mut()[RAFT_STONE_SLOT] = 1;
            }

            center_extent_on(
                0,
                (the_hero.x as i16).wrapping_add(CARRIER_OFFSET),
                (the_hero.y as i16).wrapping_add(CARRIER_OFFSET),
            );

            load_carrier(game, CARRIER_BIRD);

            true
        } else if key == CHEAT_STUFF {
            let slot = rnd(GOLDBASE) as usize;
            player().stuff_mut()[slot] = (player().stuff()[slot]).wrapping_add(CHEAT_STUFF_COUNT);
            update_menu_availability();

            player().stuff_mut()[VIAL_SLOT] = 0;
            true
        } else {
            false
        }
    }
}

/// The stuff cheat adds this many of a random item; the print requests show debug information.
const CHEAT_STUFF_COUNT: u8 = 3;
const PRQ_DEBUG_COORDS: i32 = 2;
const PRQ_DEBUG_SECTOR: i32 = 3;

/// Debug keys that rescue, redraw or move the hero about. Returns true if the key was one.
pub(super) unsafe fn cheat_move(game: &mut Game, key: i8) -> bool {
    unsafe {
        if game.ui.cheat_enabled == 0 {
            return false;
        }

        let mut hero = ActorRef::new(0);

        match key {
            CHEAT_RESCUE => rescue_princess(game),
            CHEAT_PRQ_2 => queue_print(PRQ_DEBUG_COORDS),
            CHEAT_PRQ_3 => queue_print(PRQ_DEBUG_SECTOR),
            CHEAT_TIME => the_world.day_clock = the_world.day_clock.wrapping_add(CHEAT_TIME_STEP),
            CHEAT_NORTH => {
                hero.abs_y = hero.abs_y.wrapping_sub(CHEAT_STEP_Y);
                the_world.map_y = the_world.map_y.wrapping_sub(CHEAT_STEP_Y);
            }

            CHEAT_SOUTH => {
                hero.abs_y = hero.abs_y.wrapping_add(CHEAT_STEP_Y);
                the_world.map_y = the_world.map_y.wrapping_add(CHEAT_STEP_Y);
            }

            CHEAT_EAST => {
                hero.abs_x = hero.abs_x.wrapping_add(CHEAT_STEP_X);
                the_world.map_x = the_world.map_x.wrapping_add(CHEAT_STEP_X);
            }

            CHEAT_WEST => {
                hero.abs_x = hero.abs_x.wrapping_sub(CHEAT_STEP_X);
                the_world.map_x = the_world.map_x.wrapping_sub(CHEAT_STEP_X);
            }

            _ => return false,
        }

        true
    }
}
