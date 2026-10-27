//! What the people near the hero say when the hero comes close (from the main loop in
//! csrc/fmain.c).

use crate::game::data::object_tables::ob_list8;
use crate::game::data::state::the_actors;
use crate::game::ui::text::speak;

const BEGGAR: i16 = 0x8d;
const WITCH: i16 = 0x89;
const PRINCESS: i16 = 0x84;
const NECROMANCER: i16 = 9;
const DKNIGHT: i16 = 7;
const PRINCESS_FREED_SLOT: usize = 9;

/// The nearest person says something when they come near (once per person).
pub fn speak_nearby() {
    unsafe {
        let race = the_actors.list[the_actors.nearest_person as usize].race as i16;
        if the_actors.nearest_person != 0 && race != the_actors.last_person {
            match race {
                BEGGAR => speak(23),
                WITCH => speak(46),
                PRINCESS => {
                    if ob_list8[PRINCESS_FREED_SLOT].ob_stat != 0 {
                        speak(16);
                    }
                }
                NECROMANCER => speak(43),
                DKNIGHT => speak(41),
                _ => {}
            }
            the_actors.last_person = race;
        }
    }
}
