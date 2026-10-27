//! People standing about (set figures): their pictures, talking, and the witch watching the hero.

use super::{Ctx, DEAD, DYING, Pc, SETFIG_WITCH, STILL, TALKING};
use crate::game::actors::steering::steer_actor;
use crate::game::data::random::rand2;
use crate::game::data::state::{the_hero, the_witch};
use crate::game::data::tables::setfig_table;
use crate::game::world::objects::SetFigEntry;

impl Ctx {
    /// People standing about: images from the setfig table, talking, and the witch watching the hero.
    pub(super) unsafe fn setfig(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;
            self.terrain = ((*actor).race & 0x7f) as i16;
            let entry = &*(&raw const setfig_table)
                .cast::<SetFigEntry>()
                .add(self.terrain as usize);
            self.image = entry.image_base as i16;
            let witch = self.terrain == SETFIG_WITCH;

            if self.state == DYING {
                self.image += 2;

                if witch {
                    self.image += 2;
                }
            } else if self.state == DEAD {
                self.image += 3;

                if witch {
                    self.image += 2;
                }
            } else if witch {
                steer_actor(self.slot, the_hero.x as i32, the_hero.y as i32, 0);
                self.image = ((*actor).facing / 2) as i16;
                the_witch.on_screen = 1;
            } else if self.state == TALKING {
                self.image += rand2() as i16;
                (*actor).tactic = (*actor).tactic.wrapping_sub(1);

                if (*actor).tactic == 0 {
                    (*actor).state = STILL;
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
