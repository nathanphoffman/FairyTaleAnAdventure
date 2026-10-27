//! The dragon: how it moves and breathes fire.

use super::*;

impl Ctx {
    /// The dragon: it always shows image 0 (the C sets the dying and dead images and then overwrites
    /// them), and now and then breathes a fireball.
    pub(super) unsafe fn dragon(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.dex = 0;
            if self.s == DYING {
                self.dex = 3;
            } else if self.s == DEAD {
                self.dex = 4;
            } else if rand4() == 0 {
                (*self.ms).speed = MISSILE_SPEED_FIREBALL;
                the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                self.dex = (rand2() + 1) as i16;
                play_effect(5, 1800 + rand256());
                (*an).facing = DRAGON_BREATH_FACING;
                (*self.ms).missile_type = MISSILE_ROCK_TYPE;
                return Pc::DragShoot;
            }
            self.dex = 0;
            Pc::Tail
        }
    }

    /// Sets up the missile just started (also used by the dragon): where it starts, which way it
    /// goes, and who shot it.
    pub(super) unsafe fn drag_shoot(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let ms = self.ms;
            let d = self.d as usize;
            let dx = bowshotx.get(d).copied().unwrap_or(0) as i32;
            (*ms).abs_x = ((*an).abs_x as i32 + dx) as u16;
            (*ms).abs_y = (*an).abs_y;
            let dy = if (*an).weapon == WEAPON_BOW {
                bowshoty.get(d).copied().unwrap_or(0)
            } else {
                gunshoty.get(d).copied().unwrap_or(0)
            };
            (*ms).abs_y = ((*ms).abs_y as i32 + dy as i32) as u16;
            (*ms).time_of_flight = 0;
            (*ms).direction = (*an).facing;
            (*ms).archer = self.i as i8;
            if the_actors.next_missile > MAX_MISSILE {
                the_actors.next_missile = 0;
            }
            the_actors.frustration = 0;
            Pc::Cpx
        }
    }
}
