//! What the people near the hero say when the hero comes close (from the main loop in
//! csrc/fmain.c).

use crate::game::data::state::Game;
use crate::game::ui::text::speak;

const BEGGAR: i16 = 0x8d;
const WITCH: i16 = 0x89;
const PRINCESS: i16 = 0x84;
const NECROMANCER: i16 = 9;
const DKNIGHT: i16 = 7;
const PRINCESS_FREED_SLOT: usize = 9;

// Speech numbers of what each person says on coming near.
const SPEECH_BEGGAR: i32 = 23;
const SPEECH_WITCH: i32 = 46;
const SPEECH_PRINCESS: i32 = 16;
const SPEECH_NECROMANCER: i32 = 43;
const SPEECH_DKNIGHT: i32 = 41;

/// The nearest person says something when they come near (once per person).
pub fn speak_nearby(game: &mut Game) {
    let race = game.actors.list[game.actors.nearest_person as usize].race as i16;

    if game.actors.nearest_person != 0 && race != game.actors.last_person {
        match race {
            BEGGAR => speak(game, SPEECH_BEGGAR),
            WITCH => speak(game, SPEECH_WITCH),
            PRINCESS => {
                if game.objects.region[8][PRINCESS_FREED_SLOT].ob_stat != 0 {
                    speak(game, SPEECH_PRINCESS);
                }
            }

            NECROMANCER => speak(game, SPEECH_NECROMANCER),
            DKNIGHT => speak(game, SPEECH_DKNIGHT),
            _ => {}
        }

        game.actors.last_person = race;
    }
}
