//! Placing new monsters while the hero walks about (from the main loop in csrc/fmain.c).

use crate::game::actors::encounters::load_enemy_sprites;
use crate::game::actors::encounters::place_encounter;
use crate::game::actors::interaction::pick_encounter_spot;
use crate::game::data::random::{fta_rand, rand4, rand64};
use crate::game::data::state::{the_actors, the_encounter, the_world};
use crate::game::files::disk::{motor_off, read_finished};
use crate::game::files::loading::finish_sprite_load;
use crate::game::world::map::terrain_at;
use crate::game::world::places::find_place;

// Actor kinds and states (csrc/ftale.h).
const DEAD: i8 = 15;

const ENEMY_SLOT: i16 = 2;
const FIRST_ENEMY_ACTOR: i16 = 3;
const LAST_ENEMY_ACTOR: i16 = 7;
const DISK_REQUEST_ACTORS: i32 = 8;
const PLACE_TRIES: i32 = 10;

/// Waiting encounters are placed on every 16th tick, and a new random one is rolled every 32nd.
const PLACE_PERIOD_MASK: u16 = 15;
const ROLL_PERIOD_MASK: u16 = 31;

/// The danger of an encounter is the extent type plus this base (higher indoors); the roll to beat
/// is 0-63.
const DANGER_BASE_OUTDOORS: u16 = 2;
const DANGER_BASE_INDOORS: u16 = 5;

/// Encounter kinds chosen by rule: the dark knight extent turns a kind 2 into 4, the spider pit
/// always gives 6 and the final extent 2.
const KIND_DKNIGHT_SWAP: (u16, u16) = (2, 4);
const KIND_SPIDER_PIT: u16 = 6;
const KIND_FINAL: u16 = 2;

/// Extents whose type is a multiple of 4 never mix races.
const NO_MIX_MASK: u16 = 3;
const SPREAD: u16 = 63;
const PLAIN_TERRAIN: i32 = 0;
const WRAITH: u8 = 2;

/// Xtype values: above this no mixed groups; the limit for random encounters; special zones.
const NO_MIX_XTYPE: u16 = 49;
const NO_RANDOM_XTYPE: u16 = 50;
const SPIDER_PIT_XTYPE: u16 = 8;
const DKNIGHT_XTYPE: u16 = 7;
const FINAL_XTYPE: u16 = 49;
const INDOOR_REGION: u16 = 7;

/// Says where the hero is, finishes loading monsters, places waiting ones and now and then
/// starts a new random encounter.
pub fn spawn_encounters() {
    unsafe {
        find_place(2);

        if the_actors.loading == 1 && read_finished(DISK_REQUEST_ACTORS) != 0 {
            finish_sprite_load(ENEMY_SLOT);
            motor_off();
            the_actors.loading = 0;
            the_actors.active = FIRST_ENEMY_ACTOR;
        }

        if the_world.day_clock & PLACE_PERIOD_MASK == 0
            && the_encounter.number != 0
            && the_actors.loading == 0
        {
            place_waiting_encounters();
        }

        if the_world.day_clock & ROLL_PERIOD_MASK == 0
            && the_actors.on_screen == 0
            && the_actors.loading == 0
            && the_actors.active_carrier == 0
            && the_world.extent_type < NO_RANDOM_XTYPE
        {
            the_encounter.danger_level = (if the_world.region > INDOOR_REGION {
                DANGER_BASE_INDOORS + the_world.extent_type
            } else {
                DANGER_BASE_OUTDOORS + the_world.extent_type
            }) as u8;

            if rand64() <= the_encounter.danger_level as i32 {
                the_encounter.kind = rand4() as u16;

                if the_world.extent_type == DKNIGHT_XTYPE
                    && the_encounter.kind == KIND_DKNIGHT_SWAP.0
                {
                    the_encounter.kind = KIND_DKNIGHT_SWAP.1;
                }

                if the_world.extent_type == SPIDER_PIT_XTYPE {
                    the_encounter.kind = KIND_SPIDER_PIT;
                    the_encounter.mix_flag = 0;
                }

                if the_world.extent_type == FINAL_XTYPE {
                    the_encounter.kind = KIND_FINAL;
                    the_encounter.mix_flag = 0;
                }

                load_enemy_sprites();
            }
        }
    }
}

/// Tries to find a clear spot near the hero and puts the waiting encounter there, into free
/// actor slots first and then into dead ones.
unsafe fn place_waiting_encounters() {
    unsafe {
        the_encounter.mix_flag = fta_rand() as i16;

        if the_world.extent_type > NO_MIX_XTYPE {
            the_encounter.mix_flag = 0;
        }

        the_encounter.weapon_type = rand4() as i16;

        if the_world.extent_type & NO_MIX_MASK == 0 {
            the_encounter.mix_flag = 0;
        }

        for _ in 1..PLACE_TRIES {
            pick_encounter_spot();

            if terrain_at(the_encounter.x as i32, the_encounter.y as i32) != PLAIN_TERRAIN {
                continue;
            }

            while the_encounter.number != 0 && the_actors.active < LAST_ENEMY_ACTOR {
                if place_encounter(the_actors.active as u16, SPREAD) != 0 {
                    the_actors.active += 1;
                }

                the_encounter.number -= 1;
            }

            let mut i = FIRST_ENEMY_ACTOR as usize;

            while i < LAST_ENEMY_ACTOR as usize && the_encounter.number != 0 {
                let actor = &raw const the_actors.list[i];

                if (*actor).state == DEAD && ((*actor).visible == 0 || (*actor).race == WRAITH) {
                    place_encounter(i as u16, SPREAD);
                    the_encounter.number -= 1;
                }

                i += 1;
            }

            break;
        }
    }
}
