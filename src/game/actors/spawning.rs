//! Placing new monsters while the hero walks about (from the main loop in csrc/fmain.c).

use crate::game::actors::encounters::load_enemy_sprites;
use crate::game::actors::encounters::place_encounter;
use crate::game::actors::interaction::pick_encounter_spot;
use crate::game::data::random::{fta_rand, rand4, rand64};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Actors;
use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::data::state::World;
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
pub fn spawn_encounters(game: &mut Game, video: &mut Video) {
    find_place(game, video, 2);

    if game.actors.loading == 1 && read_finished(DISK_REQUEST_ACTORS) != 0 {
        finish_sprite_load(video, ENEMY_SLOT);
        motor_off();
        game.actors.loading = 0;
        game.actors.active = FIRST_ENEMY_ACTOR;
    }

    if game.world.day_clock & PLACE_PERIOD_MASK == 0
        && game.encounter.number != 0
        && game.actors.loading == 0
    {
        place_waiting_encounters(game);
    }

    if random_encounter_due(&game.actors, &game.world) {
        roll_random_encounter(game, video);
    }
}

/// A new random encounter is rolled now and then, when nothing else is going on.
fn random_encounter_due(actors: &Actors, world: &World) -> bool {
    world.day_clock & ROLL_PERIOD_MASK == 0
        && actors.on_screen == 0
        && actors.loading == 0
        && actors.active_carrier == 0
        && world.extent_type < NO_RANDOM_XTYPE
}

/// Rolls the danger of the area against a random number; if it is beaten, picks the kind of the
/// encounter (a few extents decide it by rule) and starts loading the monsters' sprites.
fn roll_random_encounter(game: &mut Game, video: &mut Video) {
    game.encounter.danger_level = (if game.world.region > INDOOR_REGION {
        DANGER_BASE_INDOORS + game.world.extent_type
    } else {
        DANGER_BASE_OUTDOORS + game.world.extent_type
    }) as u8;

    if rand64(&mut game.rng) <= game.encounter.danger_level as i32 {
        game.encounter.kind = rand4(&mut game.rng) as u16;

        if game.world.extent_type == DKNIGHT_XTYPE && game.encounter.kind == KIND_DKNIGHT_SWAP.0 {
            game.encounter.kind = KIND_DKNIGHT_SWAP.1;
        }

        if game.world.extent_type == SPIDER_PIT_XTYPE {
            game.encounter.kind = KIND_SPIDER_PIT;
            game.encounter.mix_flag = 0;
        }

        if game.world.extent_type == FINAL_XTYPE {
            game.encounter.kind = KIND_FINAL;
            game.encounter.mix_flag = 0;
        }

        load_enemy_sprites(game, video);
    }
}

/// Tries to find a clear spot near the hero and puts the waiting encounter there, into free
/// actor slots first and then into dead ones.
fn place_waiting_encounters(game: &mut Game) {
    game.encounter.mix_flag = fta_rand(&mut game.rng) as i16;

    if game.world.extent_type > NO_MIX_XTYPE {
        game.encounter.mix_flag = 0;
    }

    game.encounter.weapon_type = rand4(&mut game.rng) as i16;

    if game.world.extent_type & NO_MIX_MASK == 0 {
        game.encounter.mix_flag = 0;
    }

    for _ in 1..PLACE_TRIES {
        pick_encounter_spot(game);

        if terrain_at(game, game.encounter.x as i32, game.encounter.y as i32) != PLAIN_TERRAIN {
            continue;
        }

        place_in_free_slots(game);
        place_in_dead_slots(game);

        break;
    }
}

/// Puts the waiting monsters into the actor slots that are still free.
fn place_in_free_slots(game: &mut Game) {
    while game.encounter.number != 0 && game.actors.active < LAST_ENEMY_ACTOR {
        if place_encounter(game, game.actors.active as u16, SPREAD) != 0 {
            game.actors.active += 1;
        }

        game.encounter.number -= 1;
    }
}

/// Puts the monsters still waiting in place of dead ones that are no longer on screen (a wraith
/// can be replaced even while its body is seen).
fn place_in_dead_slots(game: &mut Game) {
    let mut i = FIRST_ENEMY_ACTOR as usize;

    while i < LAST_ENEMY_ACTOR as usize && game.encounter.number != 0 {
        let actor = ActorRef::new(i);

        if game.actor(actor).state == DEAD
            && (game.actor(actor).visible == 0 || game.actor(actor).race == WRAITH)
        {
            place_encounter(game, i as u16, SPREAD);
            game.encounter.number -= 1;
        }

        i += 1;
    }
}
