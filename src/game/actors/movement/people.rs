//! People standing about (set figures): their pictures, talking, and the witch watching the hero.

use super::{
    Ctx, DEAD, DYING, PERSON_DEAD_OFFSET, PERSON_DYING_OFFSET, PERSON_MASK, Pc, SETFIG_WITCH,
    STILL, TALKING, WITCH_EXTRA_OFFSET,
};
use crate::game::data::state::Game;

use crate::game::actors::steering::steer_actor;
use crate::game::data::random::rand2;
use crate::game::data::tables::setfig_table;

/// The witch's dying and dead frames come two further on than other people's.
fn witch_extra(witch: bool) -> i16 {
    if witch { WITCH_EXTRA_OFFSET } else { 0 }
}

impl Ctx {
    /// People standing about: images from the setfig table, talking, and the witch watching the hero.
    pub(super) fn setfig(&mut self, game: &mut Game) -> Pc {
        let actor = self.actor;

        // Masks with PERSON_MASK, keeping only the bits that matter here.
        self.terrain = (game.actors.list[actor.slot()].race & PERSON_MASK) as i16;
        let entry = &setfig_table[self.terrain as usize];
        self.image = entry.image_base as i16;
        let witch = self.terrain == SETFIG_WITCH;

        if self.state == DYING {
            self.image += PERSON_DYING_OFFSET + witch_extra(witch);
        } else if self.state == DEAD {
            self.image += PERSON_DEAD_OFFSET + witch_extra(witch);
        } else if witch {
            steer_actor(game, self.slot, game.hero.x as i32, game.hero.y as i32, 0);
            self.image = (game.actors.list[actor.slot()].facing / 2) as i16;
            game.witch.on_screen = 1;
        } else if self.state == TALKING {
            self.image += rand2(&mut game.rng) as i16;
            game.actors.list[actor.slot()].tactic =
                game.actors.list[actor.slot()].tactic.wrapping_sub(1);

            if game.actors.list[actor.slot()].tactic == 0 {
                game.actors.list[actor.slot()].state = STILL;
            }
        } else {
            return Pc::Statc;
        }

        self.environ = 0;
        self.terrain = 0;

        Pc::Tail
    }
}
