//! People standing about (set figures): their pictures, talking, and the witch watching the hero.

use crate::game::data::state::Game;
use super::{
    Ctx, DEAD, DYING, PERSON_DEAD_OFFSET, PERSON_DYING_OFFSET, PERSON_MASK, Pc, SETFIG_WITCH,
    STILL, TALKING, WITCH_EXTRA_OFFSET,
};

use crate::game::actors::steering::steer_actor;
use crate::game::data::random::rand2;
use crate::game::data::state::{the_hero};
use crate::game::data::tables::setfig_table;
use crate::game::world::objects::SetFigEntry;

/// The witch's dying and dead frames come two further on than other people's.
fn witch_extra(witch: bool) -> i16 {
    if witch { WITCH_EXTRA_OFFSET } else { 0 }
}

impl Ctx {
    /// People standing about: images from the setfig table, talking, and the witch watching the hero.
    pub(super) unsafe fn setfig(&mut self, game: &mut Game) -> Pc {
        unsafe {
            let mut actor = self.actor;

            // Masks with PERSON_MASK, keeping only the bits that matter here.
            self.terrain = (actor.race & PERSON_MASK) as i16;
            let entry = &*(&raw const setfig_table)
                .cast::<SetFigEntry>()
                .add(self.terrain as usize);
            self.image = entry.image_base as i16;
            let witch = self.terrain == SETFIG_WITCH;

            if self.state == DYING {
                self.image += PERSON_DYING_OFFSET + witch_extra(witch);
            } else if self.state == DEAD {
                self.image += PERSON_DEAD_OFFSET + witch_extra(witch);
            } else if witch {
                steer_actor(self.slot, the_hero.x as i32, the_hero.y as i32, 0);
                self.image = (actor.facing / 2) as i16;
                game.witch.on_screen = 1;
            } else if self.state == TALKING {
                self.image += rand2() as i16;
                actor.tactic = actor.tactic.wrapping_sub(1);

                if actor.tactic == 0 {
                    actor.state = STILL;
                }
            } else {
                return Pc::Statc;
            }

            self.environ = 0;
            self.terrain = 0;

            Pc::Tail
        }
    }
}
