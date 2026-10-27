//! People standing about (set figures): their pictures, talking, and the witch watching the hero.

use super::*;

impl Ctx {
    /// People standing about: images from the setfig table, talking, and the witch watching the hero.
    pub(super) unsafe fn setfig(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.j = ((*an).race & 0x7f) as i16;
            let entry = &*(&raw const setfig_table)
                .cast::<SetFigEntry>()
                .add(self.j as usize);
            self.dex = entry.image_base as i16;
            let witch = self.j == SETFIG_WITCH;
            if self.s == DYING {
                self.dex += 2;
                if witch {
                    self.dex += 2;
                }
            } else if self.s == DEAD {
                self.dex += 3;
                if witch {
                    self.dex += 2;
                }
            } else if witch {
                steer_actor(self.i, the_hero.x as i32, the_hero.y as i32, 0);
                self.dex = ((*an).facing / 2) as i16;
                the_witch.on_screen = 1;
            } else if self.s == TALKING {
                self.dex += rand2() as i16;
                (*an).tactic = (*an).tactic.wrapping_sub(1);
                if (*an).tactic == 0 {
                    (*an).state = STILL;
                }
            } else {
                return Pc::Statc;
            }
            self.k = 0;
            self.j = 0;
            Pc::Tail
        }
    }
}
