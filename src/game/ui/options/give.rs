//! The Give menu: gold to beggars and a bone to the spectre.

use super::{BONE, ITEMS, PRQ_STATS, PRQ_VITALITY};
use crate::game::data::random::rand64;
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::ui::menus::go_to_menu;
use crate::game::ui::text::{queue_print, speak};
use crate::game::world::objects::leave_item;

// Give menu.
const GIVE_GOLD: i16 = 5;
const GIVE_BONE: i16 = 8;
const GOLD_GIVEN: i16 = 2;
const BEGGAR_RACE: u8 = 0x8d;
const SPECTRE_RACE: u8 = 0x8a;
const BEGGAR_SPEECH: i32 = 24;
const BEGGAR_THANKS: i32 = 50;
const SPECTRE_NOT_HERE: i32 = 21;
const SPECTRE_THANKS: i32 = 48;
const SPECTRE_BONE_DROP: i16 = 140;

/// Gives gold to a beggar, or the bone to the spectre.
pub(super) fn give_option(game: &mut Game, chosen_entry: i16) -> bool {
    if game.actors.nearest_person == 0 {
        return true;
    }

    let person = ActorRef::new(game.actors.nearest_person as usize);
    let race = game.actor(person).race;

    if chosen_entry == GIVE_GOLD && game.hero.wealth > GOLD_GIVEN {
        game.hero.wealth -= GOLD_GIVEN;

        if rand64(&mut game.rng) > game.hero.kindness as i32 {
            game.hero.kindness += 1;
        }

        queue_print(game, PRQ_STATS);
        queue_print(game, PRQ_VITALITY);

        if race == BEGGAR_RACE {
            speak(game, BEGGAR_SPEECH + game.actor(person).goal as i32);
        } else {
            speak(game, BEGGAR_THANKS);
        }
    } else if chosen_entry == GIVE_BONE && game.hero.stuff()[BONE] != 0 {
        if race != SPECTRE_RACE {
            speak(game, SPECTRE_NOT_HERE);
        } else {
            speak(game, SPECTRE_THANKS);
            game.hero.stuff_mut()[BONE] = 0;
            leave_item(game, game.actors.nearest_person, SPECTRE_BONE_DROP);
        }
    }

    go_to_menu(game, ITEMS as i16);

    true
}
