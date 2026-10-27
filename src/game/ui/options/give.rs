//! The Give menu: gold to beggars and a bone to the spectre.

use super::*;

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
pub(super) unsafe fn give_option(chosen_entry: i16) -> bool {
    unsafe {
        if the_actors.nearest_person == 0 {
            return true;
        }
        let person = &raw mut the_actors.list[the_actors.nearest_person as usize];
        let race = (*person).race;
        if chosen_entry == GIVE_GOLD && the_hero.wealth > GOLD_GIVEN {
            the_hero.wealth -= GOLD_GIVEN;
            if rand64() > the_hero.kindness as i32 {
                the_hero.kindness += 1;
            }
            queue_print(PRQ_STATS);
            queue_print(PRQ_VITALITY);
            if race == BEGGAR_RACE {
                speak(BEGGAR_SPEECH + (*person).goal as i32);
            } else {
                speak(BEGGAR_THANKS);
            }
        } else if chosen_entry == GIVE_BONE && *the_hero.stuff.add(BONE) != 0 {
            if race != SPECTRE_RACE {
                speak(SPECTRE_NOT_HERE);
            } else {
                speak(SPECTRE_THANKS);
                *the_hero.stuff.add(BONE) = 0;
                leave_item(the_actors.nearest_person, SPECTRE_BONE_DROP);
            }
        }
        go_to_menu(ITEMS as i16);
        true
    }
}
