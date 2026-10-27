//! Shooting arrows and fireballs: starting a missile when an actor lets go.

use super::*;

impl Ctx {
    /// Aiming and firing a bow or a fireball. No shooting under water or in peaceful places.
    pub(super) unsafe fn shooting(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let ms = self.ms;
            let d8 = self.d + 8;
            if self.k > ENVIRON_SINKING || the_world.extent_type > NO_SHOOT_XTYPE {
                return Pc::Cpx;
            }
            if self.s == SHOOT3 {
                if (*an).weapon == WEAPON_FIREBALL {
                    self.dex = diroff(d8);
                    return Pc::Cpx;
                }
                self.dex = diroff(d8) + 11;
                (*an).state = STILL;
                if self.i == 0 {
                    if *the_hero.stuff.add(STUFF_ARROWS) == 0 {
                        return Pc::Cpx;
                    }
                    *the_hero.stuff.add(STUFF_ARROWS) -= 1;
                }
                (*ms).speed = MISSILE_SPEED_ARROW;
                the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                play_effect(4, 400 + rand256());
            } else if self.s == SHOOT1 {
                self.dex = diroff(d8) + if (*an).kind == ENEMY { 11 } else { 10 };
                if (*an).weapon == WEAPON_FIREBALL {
                    (*ms).speed = MISSILE_SPEED_FIREBALL;
                    the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                    (*an).state = SHOOT3 as i8;
                    self.dex = diroff(d8);
                    play_effect(5, 1800 + rand256());
                } else if self.i == 0 && *the_hero.stuff.add(STUFF_ARROWS) == 0 {
                    print(c"No Arrows!".as_ptr());
                    return Pc::Cpx;
                } else {
                    (*ms).speed = 0;
                }
            }
            (*ms).missile_type = (*an).weapon - 3;
            Pc::DragShoot
        }
    }
}
