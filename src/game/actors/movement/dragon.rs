//! The dragon: how it moves and breathes fire.

use super::{
    Ctx, DEAD, DRAGON_BREATH_FACING, DYING, MAX_MISSILE, MISSILE_ROCK_TYPE, MISSILE_SPEED_FIREBALL,
    Pc, WEAPON_BOW,
};

use crate::game::data::object_tables::{bowshotx, bowshoty, gunshoty};
use crate::game::data::random::{rand2, rand4, rand256};
use crate::game::data::state::the_actors;
use crate::game::files::loading::play_effect;

impl Ctx {
    /// The dragon: it always shows image 0 (the C sets the dying and dead images and then overwrites
    /// them), and now and then breathes a fireball.
    pub(super) unsafe fn dragon(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;
            self.image = 0;

            if self.state == DYING {
                self.image = 3;
            } else if self.state == DEAD {
                self.image = 4;
            } else if rand4() == 0 {
                (*self.missile).speed = MISSILE_SPEED_FIREBALL;
                the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                self.image = (rand2() + 1) as i16;
                play_effect(5, 1800 + rand256());
                (*actor).facing = DRAGON_BREATH_FACING;
                (*self.missile).missile_type = MISSILE_ROCK_TYPE;

                return Pc::DragShoot;
            }

            self.image = 0;

            Pc::Tail
        }
    }

    /// Sets up the missile just started (also used by the dragon): where it starts, which way it
    /// goes, and who shot it.
    pub(super) unsafe fn drag_shoot(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;
            let ms = self.missile;
            let d = self.facing as usize;
            let dx = bowshotx.get(d).copied().unwrap_or(0) as i32;
            (*ms).abs_x = ((*actor).abs_x as i32 + dx) as u16;
            (*ms).abs_y = (*actor).abs_y;
            let dy = if (*actor).weapon == WEAPON_BOW {
                bowshoty.get(d).copied().unwrap_or(0)
            } else {
                gunshoty.get(d).copied().unwrap_or(0)
            };

            (*ms).abs_y = ((*ms).abs_y as i32 + dy as i32) as u16;
            (*ms).time_of_flight = 0;
            (*ms).direction = (*actor).facing;
            (*ms).archer = self.slot as i8;

            if the_actors.next_missile > MAX_MISSILE {
                the_actors.next_missile = 0;
            }

            the_actors.frustration = 0;

            Pc::Cpx
        }
    }
}
